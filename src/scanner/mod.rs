use std::sync::Arc;

use parking_lot::Mutex;
use tokio::io::AsyncReadExt;

use crate::{
    common::{CancelToken, Jolt, Point},
    cracker::Cracker,
    scanner::data::CrackingSettings,
};

pub mod data;
mod ffmpeg;
mod food;
mod points;
mod process;
mod utils;
use utils::{check_jolt, frame_to_tick};

#[derive(Debug, Clone, Default)]
pub struct ScannerState {
    pub progress: f32,
    pub last_detection: Option<(Point, egui::ColorImage)>,
    pub status: ScannerStatus,
    pub jolts: Vec<Point>,
}

#[derive(Debug, Clone, Default)]
pub enum ScannerStatus {
    #[default]
    Scanning,
    Done,
    Stopped,
    Error(String),
}

pub async fn scan_video(
    settings: &CrackingSettings,
    state: Arc<Mutex<ScannerState>>,
    cancel: CancelToken,
) {
    if let Err(e) = scan_video_inner(settings, &state, &cancel).await {
        let mut guard = state.lock();
        if cancel.is_cancelled() {
            guard.status = ScannerStatus::Stopped;
        } else if !matches!(guard.status, ScannerStatus::Error(_)) {
            guard.status = ScannerStatus::Error(e);
        }
    }
}

async fn scan_video_inner(
    settings: &CrackingSettings,
    state: &Arc<Mutex<ScannerState>>,
    cancel: &CancelToken,
) -> Result<(), String> {
    let bbox = settings.rect;
    if bbox.2 == 0 || bbox.3 == 0 {
        return Err("Hunger bbox width and height must be non-zero".to_string());
    }
    let scale = bbox.2 as f64 / 81.0;

    const PREVIEW_WIDTH: u32 = 500;
    let preview_h = (bbox.3 * PREVIEW_WIDTH / bbox.2.max(1)).max(1);

    let food_w = (9.0 * scale).round() as u32;
    let food_h = (9.0 * scale).round() as u32;
    if food_w == 0 || food_h == 0 {
        return Err(format!(
            "Hunger bbox width {} too small to scale food templates",
            bbox.2
        ));
    }

    let (need_w, need_h) = points::required_extents(scale, food_w, food_h);
    if bbox.2.saturating_add(1) < need_w || bbox.3.saturating_add(1) < need_h {
        return Err(format!(
            "Hunger bbox {}x{} too small for its width: need at least {}x{} (height should be ~width*11/81 ~= {:.0})",
            bbox.2,
            bbox.3,
            need_w,
            need_h,
            11.0 * scale
        ));
    }

    if cancel.is_cancelled() {
        state.lock().status = ScannerStatus::Stopped;
        return Ok(());
    }

    let (foods, alpha) = food::prepare_food_images(settings)?;

    let points = points::prepare_points(scale, bbox.2, bbox.3, food_w, food_h);

    let probe = ffmpeg::ffmpeg_probe(settings).await?;
    let fps = probe.fps;

    if cancel.is_cancelled() {
        state.lock().status = ScannerStatus::Stopped;
        return Ok(());
    }

    let mut child = ffmpeg::spawn_ffmpeg(settings)?;

    let mut stdout = match child.stdout.take() {
        Some(stdout) => stdout,
        None => {
            let _ = child.start_kill();
            let _ = child.wait().await;
            return Err("Failed to capture ffmpeg stdout".to_string());
        }
    };

    let mut buffer = vec![0; bbox.2 as usize * bbox.3 as usize * 3];
    let mut frame_count: i32 = -1;

    let cracker = Cracker::default();
    let mut last_jolt: Option<Jolt> = None;
    let mut error: Option<String> = None;

    loop {
        frame_count += 1;
        tokio::select! {
            _ = cancel.cancelled() => break,
            result = stdout.read_exact(&mut buffer) => {
                match result {
                    Ok(_) => {
                        let Some(img) =
                            image::ImageBuffer::<image::Rgb<u8>, _>::from_raw(bbox.2, bbox.3, buffer)
                        else {
                            error = Some("Failed to create image buffer".to_string());
                            break;
                        };

                        let jolt = process::process_img(&img, &foods, &alpha, &points, food_w, food_h)
                            .ok()
                            .filter(check_jolt);

                        let preview = jolt.as_ref().map(|_| {
                            image::imageops::resize(
                                &img,
                                PREVIEW_WIDTH,
                                preview_h,
                                image::imageops::FilterType::Nearest,
                            )
                        });

                        buffer = img.into_raw();

                        let progress =
                            (frame_count as f32 / probe.frame_count as f32).clamp(0.0, 1.0);

                        let (Some(jolt), Some(preview)) = (jolt, preview) else {
                            state.lock().progress = progress;
                            continue;
                        };

                        if last_jolt.as_ref() == Some(&jolt) {
                            state.lock().progress = progress;
                            continue;
                        }

                        last_jolt = Some(jolt);

                        let time = frame_to_tick(frame_count, fps);
                        let tick = match cracker.crack_ticks(&jolt).first() {
                            Some(tick) => *tick,
                            None => {
                                continue;
                            }
                        };
                        let img = egui::ColorImage::from_rgb(
                            [PREVIEW_WIDTH as usize, preview_h as usize],
                            &preview.into_raw(),
                        );
                        let point = Point { tick, time, jolt,low_health: false };
                        let mut guard = state.lock();
                        guard.progress = progress;
                        guard.last_detection = Some((point.clone(), img));
                        guard.jolts.push(point);
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                    Err(e) => {
                        error = Some(format!("Error reading ffmpeg output: {e}"));
                        break;
                    }
                }
            }
        }
    }

    let _ = child.start_kill();
    let _ = child.wait().await;

    if cancel.is_cancelled() {
        state.lock().status = ScannerStatus::Stopped;
        return Ok(());
    }
    if let Some(e) = error {
        return Err(e);
    }

    let mut guard = state.lock();
    guard.progress = 1.0;
    guard.status = ScannerStatus::Done;
    Ok(())
}
