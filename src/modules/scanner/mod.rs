use crate::{
    app::AppModule,
    common::Task,
    scanner::{
        ScannerState, ScannerStatus,
        data::{CrackingSettings, VideoTime},
    },
};

mod scan;

#[derive(Default)]
pub struct ScanningModule {
    settings: CrackingSettings,
    scan: Option<Task<ScannerState>>,
    texture: Option<egui::TextureHandle>,
    last_shown_tick: Option<i32>,
    center_content_h: f32,
    export_error: Option<String>,
}

impl AppModule for ScanningModule {
    fn ui(&mut self, ui: &mut egui::Ui, _global: &mut crate::app::GlobalState) {
        let scanning = self.is_scanning();
        if scanning {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(100));
        }
        ui.horizontal(|ui| {
            if ui.button("Open Video").clicked()
                && let Some(path) = rfd::FileDialog::new()
                    .add_filter("Video Files", &["mp4", "mkv", "avi", "mov"])
                    .pick_file()
            {
                self.settings.video_path = Some(path);
            }

            if let Some(video) = self.settings.video_path.as_ref() {
                ui.add(
                    egui::Label::new(video.file_name().unwrap_or_default().to_string_lossy())
                        .truncate(),
                );
            } else {
                ui.label("No video opened");
            }
        });
        ui.separator();
        egui::Panel::left("cracking_aside_left")
            .resizable(false)
            .min_size(300.0)
            .show(ui, |ui| self.redner_sidebar_left(ui));

        egui::Panel::right("cracking_aside_right")
            .resizable(false)
            .min_size(200.0)
            .show(ui, |ui| self.render_sidebar_right(ui));

        egui::CentralPanel::default().show(ui, |ui| {
            self.render_center(ui);
        });
    }
    fn name(&self) -> &str {
        "Scanner"
    }
}

fn centered_content<R>(
    content_h: &mut f32,
    ui: &mut egui::Ui,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    if *content_h > 0.0 {
        ui.add_space(((ui.available_height() - *content_h) / 2.0).max(0.0));
    }
    let resp = ui.vertical_centered(add_contents);
    let h = resp.response.rect.height();
    if (h - *content_h).abs() > 0.5 {
        *content_h = h;
        ui.ctx().request_repaint();
    }
    resp.inner
}

impl ScanningModule {
    fn render_center(&mut self, ui: &mut egui::Ui) {
        let Some(task) = self.scan.as_ref() else {
            centered_content(&mut self.center_content_h, ui, |ui| {
                ui.label("No scanning in progress");
            });
            return;
        };
        let state = task.state().lock();

        if let Some((point, img)) = &state.last_detection
            && self.last_shown_tick != Some(point.tick)
        {
            match &mut self.texture {
                Some(tex) => tex.set(img.clone(), egui::TextureOptions::NEAREST),
                None => {
                    self.texture = Some(ui.ctx().load_texture(
                        "scan-frame",
                        img.clone(),
                        egui::TextureOptions::NEAREST,
                    ));
                }
            }
            self.last_shown_tick = Some(point.tick);
        }

        let texture = &self.texture;
        centered_content(&mut self.center_content_h, ui, |ui| {
            if let ScannerStatus::Error(err) = &state.status {
                ui.colored_label(egui::Color32::RED, format!("Error: {err}"));
                return;
            }
            if let Some(tex) = texture {
                ui.add(egui::Image::new(tex).corner_radius(10.0));
            }
            ui.add_space(10.0);
            ui.add(egui::ProgressBar::new(state.progress).show_percentage());
            ui.add_space(10.0);
            match state.status {
                ScannerStatus::Done => {
                    ui.label("Scanning completed.");
                }
                ScannerStatus::Stopped => {
                    ui.label("Scanning stopped.");
                }
                _ => {}
            }
        });
    }

    fn render_sidebar_right(&mut self, ui: &mut egui::Ui) {
        let Some(task) = self.scan.as_ref() else {
            ui.label("No scanning in progress");
            return;
        };
        let total = task.state().lock().jolts.len();
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
            ui.add_space(8.0);
            let jolts = self
                .scan
                .as_ref()
                .map(|task| task.state().lock().jolts.clone())
                .unwrap_or_default();
            if ui
                .add_enabled_ui(!jolts.is_empty(), |ui| {
                    ui.add_sized(
                        [ui.available_width(), 32.0],
                        egui::Button::new("Export JSON"),
                    )
                })
                .inner
                .clicked()
            {
                match serde_json::to_string_pretty(&jolts) {
                    Ok(json) => {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("JSON", &["json"])
                            .set_file_name("points.json")
                            .save_file()
                        {
                            match std::fs::write(&path, json) {
                                Ok(()) => self.export_error = None,
                                Err(e) => {
                                    self.export_error = Some(format!("Export failed: {e}"));
                                }
                            }
                        }
                    }
                    Err(e) => {
                        self.export_error = Some(format!("Serialization failed: {e}"));
                    }
                }
            }
            ui.add_space(8.0);
            if let Some(err) = self.export_error.clone() {
                ui.colored_label(egui::Color32::RED, err);
            }
            ui.separator();

            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.heading("Scan results");
                ui.label(format!("Total scanned jolts: {total}"));
                if total > 0 {
                    let state = task.state().lock();
                    egui::ScrollArea::vertical()
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            for point in &state.jolts {
                                ui.horizontal(|ui| {
                                    ui.label(format!("Tick: {}", point.tick));
                                    ui.label(format!("Jolt: {}", point.jolt));
                                });
                            }
                        });
                }
            });
        });
    }

    fn redner_sidebar_left(&mut self, ui: &mut egui::Ui) {
        let start = &mut self.settings.start_time;
        ui.checkbox(&mut start.enabled, "Use start time");
        Self::time_editors(ui, start, start.enabled);
        let end = &mut self.settings.end_time;
        ui.checkbox(&mut end.enabled, "Use end time");
        Self::time_editors(ui, end, end.enabled);
        ui.checkbox(&mut self.settings.cuda, "Use CUDA Hardware Acceleration");
        ui.add_space(10.0);
        ui.label("Hunger bbox");
        ui.horizontal(|ui| {
            ui.label("X:");
            ui.add(egui::DragValue::new(&mut self.settings.rect.0).speed(1));
            ui.label("Y:");
            ui.add(egui::DragValue::new(&mut self.settings.rect.1).speed(1));
        });
        ui.horizontal(|ui| {
            ui.label("Width:");
            ui.add(egui::DragValue::new(&mut self.settings.rect.2).speed(1));
            ui.label("Height:");
            ui.add(
                egui::DragValue::new(&mut self.settings.rect.3)
                    .speed(1)
                    .range(0..=10000),
            );
        });

        egui::Panel::bottom("cracking_start_stop").show(ui, |ui| {
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                let scanning = self.is_scanning();
                let can_start = self.settings.video_path.is_some() && !scanning;
                let button_width = (ui.available_width() - ui.spacing().item_spacing.x) / 2.0;
                if ui
                    .add_enabled_ui(can_start, |ui| {
                        ui.add_sized(
                            [button_width, 32.0],
                            egui::Button::new(egui::RichText::new("Start Scanning").size(15.0)),
                        )
                    })
                    .inner
                    .clicked()
                {
                    self.start_scanning();
                }
                if ui
                    .add_enabled_ui(scanning, |ui| {
                        ui.add_sized(
                            [button_width, 32.0],
                            egui::Button::new(egui::RichText::new("Stop").size(15.0)),
                        )
                    })
                    .inner
                    .clicked()
                {
                    self.stop_scanning();
                }
            });
        });
    }

    fn time_editors(ui: &mut egui::Ui, time: &mut VideoTime, enabled: bool) {
        ui.horizontal(|ui| {
            ui.add_enabled(
                enabled,
                egui::DragValue::new(&mut time.hours).speed(1).range(0..=99),
            );
            ui.label(":");
            ui.add_enabled(
                enabled,
                egui::DragValue::new(&mut time.minutes)
                    .speed(1)
                    .range(0..=59),
            );
            ui.label(":");
            ui.add_enabled(
                enabled,
                egui::DragValue::new(&mut time.seconds)
                    .speed(1)
                    .range(0..=59),
            );
        });
    }
}
