use egui::{Align, Color32, Layout, RichText};
use egui_material_icons::icons::{
    ICON_ARROW_DOWNWARD, ICON_ARROW_UPWARD, ICON_CONTENT_COPY, ICON_HORIZONTAL_RULE,
};

use super::Offset;

pub const UP_GREEN: Color32 = Color32::from_rgb(130, 220, 130);
pub const DOWN_RED: Color32 = Color32::from_rgb(255, 130, 130);

pub fn card(ui: &mut egui::Ui, content: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(Color32::from_rgb(28, 28, 34))
        .stroke(egui::Stroke::new(1.0, Color32::from_gray(65)))
        .corner_radius(8.0)
        .inner_margin(10.0)
        .show(ui, content);
}

pub fn kv_row(ui: &mut egui::Ui, key: &str, value: &str, copy_button: bool) {
    ui.horizontal(|ui| {
        ui.label(key);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if copy_button
                && ui
                    .button(ICON_CONTENT_COPY)
                    .on_hover_text("Copy to clipboard")
                    .clicked()
            {
                ui.ctx().copy_text(value.to_owned());
            }
            ui.label(value);
        });
    });
}

pub fn shortcut_row(ui: &mut egui::Ui, key: &str, action: &str) {
    ui.horizontal(|ui| {
        egui::Frame::new()
            .fill(Color32::from_gray(45))
            .corner_radius(4.0)
            .inner_margin(egui::vec2(6.0, 2.0))
            .show(ui, |ui| {
                ui.label(RichText::new(key).small().strong().monospace());
            });
        ui.label(RichText::new(action).weak());
    });
}

pub fn pattern_strip(ui: &mut egui::Ui, pattern: &[Offset; 10]) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        for offset in pattern {
            let (icon, color) = match offset {
                Offset::Up => (ICON_ARROW_UPWARD, UP_GREEN),
                Offset::Down => (ICON_ARROW_DOWNWARD, DOWN_RED),
                Offset::Neutral => (ICON_HORIZONTAL_RULE, Color32::from_gray(140)),
            };
            ui.label(RichText::new(icon).size(16.0).color(color));
        }
    });
}
