use crate::common::Point;

use super::{ExplorerModule, ExplorerPoint};

impl ExplorerModule {
    pub(super) fn render_right_panel(&mut self, ui: &mut egui::Ui) {
        let right_panel_frame =
            egui::Frame::side_top_panel(ui.style()).outer_margin(egui::Margin {
                left: 12,
                ..Default::default()
            });

        egui::Panel::right("explorer_aside_right")
            .frame(right_panel_frame)
            .resizable(false)
            .show(ui, |ui| {
                ui.label("Format");
                egui::ComboBox::new("format_select", "")
                    .selected_text(self.format.get_name())
                    .show_ui(ui, |ui| {
                        for format in crate::common::formats::ALL_FORMATS {
                            if ui
                                .selectable_label(
                                    self.format.get_name() == format.get_name(),
                                    format.get_name(),
                                )
                                .clicked()
                            {
                                self.format = *format;
                            }
                        }
                    });

                let (can_import, can_export) = self.format.capabilities();

                ui.add_enabled_ui(can_import, |ui| {
                    if ui.button("Load points").clicked()
                        && let Some(path) = rfd::FileDialog::new()
                            .add_filter(self.format.get_name(), &self.format.get_extensions())
                            .pick_file()
                    {
                        let format = self.format;
                        match std::fs::read(&path)
                            .map_err(|e| format!("Failed to read file: {e}"))
                            .and_then(|buffer| format.from_buffer(&buffer))
                        {
                            Ok(points) => {
                                self.points = points
                                    .into_iter()
                                    .map(|inner| ExplorerPoint {
                                        inner,
                                        selected: false,
                                    })
                                    .collect();
                                self.fit_view = true;
                                self.file_error = None;
                            }
                            Err(e) => self.file_error = Some(e),
                        }
                    }
                });

                ui.add_enabled_ui(!self.points.is_empty() && can_export, |ui| {
                    if ui.button("Save points").clicked()
                        && let Some(path) = rfd::FileDialog::new()
                            .add_filter(self.format.get_name(), &self.format.get_extensions())
                            .set_file_name(format!(
                                "points.{}",
                                self.format.get_extensions().first().unwrap_or(&"json")
                            ))
                            .save_file()
                    {
                        let plain: Vec<Point> =
                            self.points.iter().map(|p| p.inner.clone()).collect();
                        match self
                            .format
                            .to_buffer(&plain)
                            .and_then(|buffer| {
                                std::fs::write(&path, buffer)
                                    .map_err(|e| format!("Failed to write file: {e}"))
                            }) {
                            Ok(()) => self.file_error = None,
                            Err(e) => self.file_error = Some(e),
                        }
                    }
                });
                if let Some(err) = self.file_error.clone() {
                    ui.colored_label(egui::Color32::RED, err);
                }
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Min line length (in points)");
                    ui.add(
                        egui::DragValue::new(&mut self.auto_cleanup_settings.min_line_length)
                            .range(2..=50),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Tolerance (seconds)");
                    ui.add(
                        egui::DragValue::new(&mut self.auto_cleanup_settings.tolerance)
                            .range(0..=50),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Max gap (seconds)");
                    ui.add(
                        egui::DragValue::new(&mut self.auto_cleanup_settings.max_gap)
                            .range(0..=10000),
                    );
                });

                if ui.button("Auto Cleanup").clicked() {
                    self.points = super::cleanup::cleanup_data(
                        self.points.clone(),
                        &self.auto_cleanup_settings,
                    );
                }
            });
    }
}
