use egui::{Color32, ImageSource, RichText};

use crate::common::{
    Offset,
    ui::{DOWN_RED, UP_GREEN},
};

pub trait OffsetUi {
    fn spacing(&self) -> (f32, f32);
    fn ui(&mut self, texture: ImageSource<'static>, ui: &mut egui::Ui);
}

impl OffsetUi for Offset {
    fn spacing(&self) -> (f32, f32) {
        match self {
            Offset::Up => (0.0, 10.0),
            Offset::Down => (10.0, 0.0),
            Offset::Neutral => (5.0, 5.0),
        }
    }

    fn ui(&mut self, texture: ImageSource<'static>, ui: &mut egui::Ui) {
        ui.allocate_ui_with_layout(
            egui::vec2(27.0, ui.available_height()),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                let up_button = if *self == Offset::Up {
                    egui::Button::new(
                        RichText::new(egui_material_icons::icons::ICON_ARROW_UPWARD)
                            .color(Color32::BLACK),
                    )
                    .fill(UP_GREEN)
                } else {
                    egui::Button::new(egui_material_icons::icons::ICON_ARROW_UPWARD)
                };

                let down_button = if *self == Offset::Down {
                    egui::Button::new(
                        RichText::new(egui_material_icons::icons::ICON_ARROW_DOWNWARD)
                            .color(Color32::BLACK),
                    )
                    .fill(DOWN_RED)
                } else {
                    egui::Button::new(egui_material_icons::icons::ICON_ARROW_DOWNWARD)
                };

                if ui.add(up_button).clicked() {
                    self.up();
                }

                ui.add_space(self.spacing().0);

                ui.add(
                    egui::Image::new(texture)
                        .texture_options(egui::TextureOptions::NEAREST)
                        .fit_to_exact_size(egui::vec2(27.0, 27.0))
                        .show_loading_spinner(false),
                );

                ui.add_space(self.spacing().1);

                if ui.add(down_button).clicked() {
                    self.down();
                }
            },
        );
    }
}
