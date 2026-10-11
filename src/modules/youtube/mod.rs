mod download;
mod formats;
mod ytdlp;

use crate::app::{AppModule, GlobalState};
use download::ActiveDownload;
use parking_lot::Mutex;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Default)]
pub struct YoutubeModule {
    url: String,
    folder: Option<PathBuf>,
    info: Arc<Mutex<Option<ytdlp::VideoInfo>>>,
    selected_format: Option<usize>,
    download: Option<ActiveDownload>,
}

impl AppModule for YoutubeModule {
    fn ui(&mut self, ui: &mut egui::Ui, _global: &mut GlobalState) {
        self.render_sidebar(ui);
        self.render_central_panel(ui);
    }

    fn name(&self) -> &str {
        "YouTube"
    }
}

impl YoutubeModule {
    fn render_sidebar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::left("youtube_aside")
            .resizable(false)
            .show(ui, |ui| {
                self.render_url_input(ui);

                let info = self.info.lock();
                if let Some(video_info) = &*info {
                    self.render_video_preview(ui, video_info);
                }
            });
    }

    fn render_url_input(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            egui::TextEdit::singleline(&mut self.url)
                .hint_text("Enter video URL")
                .show(ui);
            if ui.button("Get Video").clicked() {
                let url = self.url.clone();
                let info_arc = Arc::clone(&self.info);
                let ctx = ui.ctx().clone();
                tokio::spawn(async move {
                    match ytdlp::yt_video_info(url).await {
                        Ok(video_info) => {
                            *info_arc.lock() = Some(video_info);
                            ctx.request_repaint();
                        }
                        Err(err) => {
                            eprintln!("Error fetching video info: {}", err);
                        }
                    }
                });
            }
        });
    }

    fn render_video_preview(&self, ui: &mut egui::Ui, video_info: &ytdlp::VideoInfo) {
        ui.separator();
        ui.heading(&video_info.title);
        ui.add(egui::Image::new(&video_info.thumbnail_url).corner_radius(10.0));
        ui.label(format!("Duration: {}", video_info.duration));
    }

    fn render_central_panel(&mut self, ui: &mut egui::Ui) {
        let Some(video_info) = self.info.lock().clone() else {
            return;
        };

        egui::CentralPanel::default().show(ui, |ui| {
            self.render_folder_picker(ui);
            ui.add_space(10.0);

            let download_requested = self.render_download_controls(ui);
            let format_to_download = self
                .selected_format
                .and_then(|index| video_info.formats.get(index))
                .map(|format| format.format_id.clone());

            ui.add_space(10.0);
            formats::render_formats_table(&mut self.selected_format, ui, &video_info);

            if download_requested
                && let Some(format_id) = format_to_download
                && let Some(folder) = self.folder.clone()
            {
                self.start_download(format_id, folder);
            }
        });

        if self.download.as_ref().is_some_and(|d| d.is_running()) {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(50));
        }
    }

    fn render_folder_picker(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("📁 Open Folder").clicked()
                && let Some(path) = rfd::FileDialog::new().pick_folder()
            {
                self.folder = Some(path);
            }

            if let Some(folder) = &self.folder {
                ui.label(format!("Selected Folder: {}", folder.display()));
            } else {
                ui.label("No folder selected");
            }
        });
    }

    fn render_download_controls(&self, ui: &mut egui::Ui) -> bool {
        ui.horizontal(|ui| {
            let current = self.download.as_ref().map(|d| d.progress());

            let downloading = matches!(
                current,
                Some(ytdlp::DownloadProgress::Started) | Some(ytdlp::DownloadProgress::Progress(_))
            );
            let button_enabled =
                !downloading && self.selected_format.is_some() && self.folder.is_some();
            let start_clicked = ui
                .add_enabled(
                    button_enabled,
                    egui::Button::new(egui::RichText::new("Download").size(18.0)),
                )
                .clicked();

            if ui.add_enabled(downloading, egui::Button::new("Stop")).clicked()
                && let Some(download) = &self.download
            {
                download.stop();
            }

            let progress = match current {
                Some(ytdlp::DownloadProgress::Progress(p)) => p / 100.0,
                Some(ytdlp::DownloadProgress::Started) => 0.0,
                Some(ytdlp::DownloadProgress::Finished) => 1.0,
                _ => 0.0,
            };
            let progress_bar = egui::ProgressBar::new(progress);
            let progress_bar = if downloading {
                progress_bar.show_percentage()
            } else {
                progress_bar
            };
            ui.add(progress_bar);

            start_clicked
        })
        .inner
    }

    fn start_download(&mut self, format_id: String, folder: PathBuf) {
        if self.url.is_empty() {
            return;
        }
        if self.download.as_ref().is_some_and(|d| d.is_running()) {
            return;
        }

        self.download = Some(ActiveDownload::spawn(self.url.clone(), format_id, folder));
    }
}
