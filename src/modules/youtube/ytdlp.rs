use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;

use jiff::{SignedDuration, fmt::friendly::SpanPrinter};
use parking_lot::Mutex;
use serde::Deserialize;
use serde_cursor::Cursor;
use tokio::io::{AsyncBufReadExt, BufReader};
use ytd_rs::YtDlp;

use crate::common::CancelToken;

#[derive(Clone)]
pub struct VideoInfo {
    pub title: String,
    pub duration: String,
    pub thumbnail_url: String,
    pub formats: Vec<YtFormatInfo>,
}

#[derive(Deserialize, Clone)]
pub struct YtFormatInfo {
    pub format_id: String,
    pub resolution: String,
    pub vcodec: String,
    pub video_ext: String,
    pub fps: Option<f32>,
}

pub async fn yt_video_info(video_url: String) -> Result<VideoInfo, String> {
    let infos = YtDlp::new(video_url)
        .get_info()
        .await
        .map_err(|e| format!("Failed to get video info: {}", e))?;

    let first_info = infos
        .into_iter()
        .next()
        .ok_or_else(|| "No video info found".to_string())?;

    let duration_raw = first_info
        .duration
        .ok_or_else(|| "Duration not available".to_string())?;

    let duration = SignedDuration::from_secs_f64(duration_raw);

    let formats: Vec<YtFormatInfo> = serde_json::from_value::<Cursor!(formats)>(first_info.extra)
        .map_err(|e| format!("Failed to parse formats: {}", e))?
        .0;

    let formats = formats
        .into_iter()
        .filter(|format| {
            format.vcodec != "none" && format.video_ext != "none" && format.fps.is_some()
        })
        .collect();

    Ok(VideoInfo {
        title: first_info.title,
        thumbnail_url: first_info
            .thumbnail
            .ok_or_else(|| "Thumbnail URL not available".to_string())?,
        duration: SpanPrinter::new().duration_to_string(&duration),
        formats,
    })
}

#[derive(Clone)]
pub enum DownloadProgress {
    Started,
    Progress(f32),
    Finished,
    Stopped,
    #[expect(dead_code, reason = "error handle todo")]
    Error(String),
}

fn set_progress(state: &Arc<Mutex<DownloadProgress>>, progress: DownloadProgress) {
    *state.lock() = progress;
}

pub async fn download_video(
    video_url: String,
    format_id: String,
    output_path: PathBuf,
    state: Arc<Mutex<DownloadProgress>>,
    cancel: CancelToken,
) {
    if cancel.is_cancelled() {
        set_progress(&state, DownloadProgress::Stopped);
        return;
    }
    set_progress(&state, DownloadProgress::Started);

    if let Err(e) = tokio::fs::create_dir_all(&output_path).await {
        set_progress(
            &state,
            DownloadProgress::Error(format!("Failed to create output dir: {e}")),
        );
        return;
    }

    let mut child = match tokio::process::Command::new("yt-dlp")
        .current_dir(&output_path)
        .arg("--newline")
        .arg("--format")
        .arg(&format_id)
        .arg("--retries")
        .arg("infinite")
        .arg("--fragment-retries")
        .arg("infinite")
        .arg("--progress-template")
        .arg("[prog]%(progress._percent)s")
        .arg(&video_url)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(e) => {
            set_progress(
                &state,
                DownloadProgress::Error(format!("Failed to start download: {e}")),
            );
            return;
        }
    };

    let stdout = match child.stdout.take() {
        Some(stdout) => stdout,
        None => {
            let _ = child.start_kill();
            let _ = child.wait().await;
            set_progress(
                &state,
                DownloadProgress::Error("Failed to capture yt-dlp stdout".to_string()),
            );
            return;
        }
    };
    let mut lines = BufReader::new(stdout).lines();

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                let _ = child.start_kill();
                let _ = child.wait().await;
                set_progress(&state, DownloadProgress::Stopped);
                return;
            }
            line = lines.next_line() => {
                match line {
                    Ok(Some(line)) => {
                        if let Some(val) = parse_progress(&line) {
                            set_progress(&state, DownloadProgress::Progress(val));
                        }
                    }
                    Ok(None) => break,
                    Err(e) => {
                        let _ = child.start_kill();
                        let _ = child.wait().await;
                        set_progress(
                            &state,
                            DownloadProgress::Error(format!("Failed to read progress: {e}")),
                        );
                        return;
                    }
                }
            }
        }
    }

    if cancel.is_cancelled() {
        let _ = child.start_kill();
        let _ = child.wait().await;
        set_progress(&state, DownloadProgress::Stopped);
        return;
    }

    match child.wait().await {
        Ok(status) if status.success() => {
            set_progress(&state, DownloadProgress::Finished);
        }
        Ok(status) => {
            set_progress(
                &state,
                DownloadProgress::Error(format!(
                    "Download failed with exit code: {}",
                    status.code().map_or("unknown".to_string(), |c| c.to_string())
                )),
            );
        }
        Err(e) => {
            set_progress(
                &state,
                DownloadProgress::Error(format!("Failed to wait for process: {e}")),
            );
        }
    }
}

fn parse_progress(line: &str) -> Option<f32> {
    let clean_line = line.trim_matches(|c| c == '\r' || c == '\n').trim();

    let prog_index = clean_line.find("[prog]")?;
    let raw_val = &clean_line[prog_index + 6..];

    raw_val.trim().trim_end_matches('%').parse::<f32>().ok()
}
