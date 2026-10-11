use crate::{
    app::AppModule,
    common::formats::{ModernFormat, PointFormat},
};
use egui::{Pos2, Vec2};
mod cleanup;
mod grid;
mod left_panel;
mod point;
mod right_panel;
mod viewport;
use point::ExplorerPoint;

impl ExplorerPoint {
    fn world_pos(&self) -> Pos2 {
        Pos2::new(self.time as f32, -self.tick as f32)
    }
}

pub struct ExplorerModule {
    pan: Vec2,
    zoom: f32,
    points: Vec<ExplorerPoint>,
    selection_box: Option<(Pos2, Pos2)>,
    auto_cleanup_settings: cleanup::AutoCleanupSettings,
    fit_view: bool,
    format: &'static dyn PointFormat,
    file_error: Option<String>,
}

impl Default for ExplorerModule {
    fn default() -> Self {
        Self {
            pan: Vec2::ZERO,
            zoom: 1.0,
            points: Vec::new(),
            selection_box: None,
            auto_cleanup_settings: cleanup::AutoCleanupSettings::default(),
            fit_view: true,
            format: &ModernFormat,
            file_error: None,
        }
    }
}

impl AppModule for ExplorerModule {
    fn ui(&mut self, ui: &mut egui::Ui, _global: &mut crate::app::GlobalState) {
        self.render_right_panel(ui);
        self.render_left_panel(ui);

        egui::CentralPanel::default().show(ui, |ui| {
            self.explorer_viewer(ui);
        });
    }

    fn name(&self) -> &str {
        "Explorer"
    }
}
