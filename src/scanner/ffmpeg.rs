use tokio::process::{Child, Command};

use crate::scanner::data::CrackingSettings;

pub fn spawn_ffmpeg(settings: &CrackingSettings) -> Result<Child, String> {
    let rect = settings.rect;

    let crop_filter = format!(
        "format=rgb24,crop={}:{}:{}:{}",
        rect.2, rect.3, rect.0, rect.1
    );

    let mut ffmpeg_args = Vec::new();

    if settings.cuda {
        ffmpeg_args.push("-hwaccel".to_string());
        ffmpeg_args.push("cuda".to_string());
    }

    if let Some(start_time) = settings.start_time.get() {
        ffmpeg_args.push("-ss".to_string());
        ffmpeg_args.push(start_time);
    }

    if let Some(end_time) = settings.end_time.get() {
        ffmpeg_args.push("-to".to_string());
        ffmpeg_args.push(end_time);
    }

    ffmpeg_args.extend([
        "-i".to_string(),
        settings
            .video_path
            .clone()
            .ok_or("Video path not set")?
            .to_string_lossy()
            .to_string(),
        "-vf".to_string(),
        crop_filter,
        "-f".to_string(),
        "rawvideo".to_string(),
        "-pix_fmt".to_string(),
        "rgb24".to_string(),
        "pipe:1".to_string(),
    ]);

    let child = Command::new("ffmpeg")
        .args(&ffmpeg_args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to spawn ffmpeg: {}", e))?;

    Ok(child)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Probe {
    pub frame_count: i32,
    pub fps: f64,
}

pub async fn ffmpeg_probe(settings: &CrackingSettings) -> Result<Probe, String> {
    let video_path = settings.video_path.as_ref().ok_or("Video path not set")?;

    let args = [
        "-v",
        "error",
        "-select_streams",
        "v:0",
        "-show_entries",
        "stream=avg_frame_rate,r_frame_rate,nb_frames,duration:format=duration",
        "-of",
        "json",
    ];

    let output = Command::new("ffprobe")
        .args(args)
        .arg(video_path)
        .output()
        .await
        .map_err(|e| format!("Failed to spawn ffprobe: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "ffprobe failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse ffprobe output: {}", e))?;

    let stream = parsed
        .get("streams")
        .and_then(|streams| streams.get(0))
        .ok_or("ffprobe: no video stream found")?;

    let fps = ["avg_frame_rate", "r_frame_rate"]
        .into_iter()
        .find_map(|key| {
            stream
                .get(key)
                .and_then(|v| v.as_str())
                .and_then(parse_rate)
        })
        .ok_or("ffprobe: no usable frame rate")?;

    let frame_count = stream
        .get("nb_frames")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<i64>().ok())
        .filter(|&n| n > 0 && n <= i32::MAX as i64)
        .map(|n| n as i32)
        .or_else(|| {
            let duration = stream
                .get("duration")
                .and_then(parse_json_f64)
                .or_else(|| {
                    parsed
                        .get("format")
                        .and_then(|format| format.get("duration"))
                        .and_then(parse_json_f64)
                })?;
            let count = (duration * fps).round();
            (count.is_finite() && count > 0.0 && count <= i32::MAX as f64).then_some(count as i32)
        })
        .ok_or("ffprobe: no frame count (nb_frames=N/A and no usable duration)")?;

    Ok(Probe { frame_count, fps })
}

fn parse_rate(s: &str) -> Option<f64> {
    let (num, den) = s.split_once('/')?;
    let num: f64 = num.trim().parse().ok()?;
    let den: f64 = den.trim().parse().ok()?;
    (num.is_finite() && den.is_finite() && num > 0.0 && den > 0.0).then_some(num / den)
}

fn parse_json_f64(value: &serde_json::Value) -> Option<f64> {
    match value {
        serde_json::Value::String(s) => {
            let f: f64 = s.trim().parse().ok()?;
            f.is_finite().then_some(f)
        }
        serde_json::Value::Number(n) => n.as_f64().filter(|f| f.is_finite()),
        _ => None,
    }
}