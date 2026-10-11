use egui::{Align, Color32, Layout, RichText};
use egui_material_icons::icons::{ICON_CHECK, ICON_CLOSE};

use crate::common::{
    tick_to_time,
    ui::{DOWN_RED, card, kv_row, pattern_strip, shortcut_row},
};

use super::{ExplorerModule, ExplorerPoint};

impl ExplorerModule {
    pub(super) fn render_left_panel(&mut self, ui: &mut egui::Ui) {
        let left_panel_frame = egui::Frame::side_top_panel(ui.style()).outer_margin(egui::Margin {
            right: 12,
            ..Default::default()
        });

        egui::Panel::left("explorer_aside_left")
            .frame(left_panel_frame)
            .resizable(false)
            .min_size(250.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Selection");
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let n = self.points.iter().filter(|p| p.selected).count();
                        ui.label(format!("{n} selected"));
                    });
                });
                ui.separator();

                let selected: Vec<usize> = self
                    .points
                    .iter()
                    .enumerate()
                    .filter_map(|(i, p)| p.selected.then_some(i))
                    .collect();
                match selected.len() {
                    0 => {
                        ui.label("Nothing selected");
                    }
                    1 => Self::render_point_card(ui, &self.points[selected[0]]),
                    2 => Self::render_diff_card(
                        ui,
                        &self.points[selected[0]],
                        &self.points[selected[1]],
                    ),
                    n => {
                        ui.label(format!("{} points selected", n));
                    }
                }
                ui.separator();
                Self::render_binds(ui);
                ui.separator();
                ui.label(format!("{} points total", self.points.len()));
            });
    }

    fn render_binds(ui: &mut egui::Ui) {
        shortcut_row(ui, "ctrl", "Pan and zoom");
        shortcut_row(ui, "drag", "Box select");
        shortcut_row(ui, "shift", "Toggle point");
        shortcut_row(ui, "F", "Fit view");
        shortcut_row(ui, "del", "Delete selected");
    }

    fn render_point_card(ui: &mut egui::Ui, point: &ExplorerPoint) {
        card(ui, |ui| {
            let time = tick_to_time(point.time)
                .unwrap()
                .strftime("%H:%M:%S")
                .to_string();
            let mc_time = tick_to_time(point.tick)
                .unwrap()
                .strftime("%H:%M:%S")
                .to_string();
            kv_row(ui, "Video time", &time, true);
            kv_row(ui, "Mc time", &mc_time, true);
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label("Food");
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("× {}", point.jolt.food_amount))
                            .monospace()
                            .strong(),
                    );
                });
            });
            ui.horizontal(|ui| {
                ui.label("Low health");
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let (icon, color) = if point.low_health {
                        (ICON_CHECK, DOWN_RED)
                    } else {
                        (ICON_CLOSE, Color32::from_gray(140))
                    };
                    ui.label(RichText::new(icon).color(color).strong());
                });
            });
            ui.add_space(4.0);
            ui.label("Jolt Pattern");
            pattern_strip(ui, &point.jolt.pattern);
        });
    }

    fn render_diff_card(ui: &mut egui::Ui, point_a: &ExplorerPoint, point_b: &ExplorerPoint) {
        card(ui, |ui| {
            let time_a = tick_to_time(point_a.time)
                .unwrap()
                .strftime("%H:%M:%S")
                .to_string();
            let time_b = tick_to_time(point_b.time)
                .unwrap()
                .strftime("%H:%M:%S")
                .to_string();
            let mc_time_a = tick_to_time(point_a.tick)
                .unwrap()
                .strftime("%H:%M:%S")
                .to_string();
            let mc_time_b = tick_to_time(point_b.tick)
                .unwrap()
                .strftime("%H:%M:%S")
                .to_string();
            kv_row(ui, "Video time", &format!("{time_a} - {time_b}"), false);
            kv_row(ui, "Mc time", &format!("{mc_time_a} - {mc_time_b}"), false);
            let time_diff = point_b.time - point_a.time;
            let tick_diff = point_b.tick - point_a.tick;
            kv_row(
                ui,
                "Video time diff",
                &format!("{} seconds", time_diff / 20),
                false,
            );
            kv_row(
                ui,
                "Mc time diff",
                &format!("{} seconds", tick_diff / 20),
                false,
            );

            let missing_time = (point_b.tick - point_a.tick) - (point_b.time - point_a.time);
            if missing_time > 0 {
                kv_row(
                    ui,
                    "Missing time",
                    &format!("{} seconds", missing_time / 20),
                    false,
                );
            }

            ui.add_space(4.0);
            ui.add_space(4.0);
            ui.label("Jolt patterns");
            pattern_strip(ui, &point_a.jolt.pattern);
            pattern_strip(ui, &point_b.jolt.pattern);
        });
    }
}
