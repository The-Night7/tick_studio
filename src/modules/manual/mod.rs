use egui::{ImageSource, RichText};
mod offset;

use crate::{
    app::AppModule,
    common::{Jolt, Offset},
    cracker::Cracker,
    modules::manual::offset::OffsetUi,
};

const FULL_FOOD_TEXTURE: ImageSource = egui::include_image!("../../../assets/food.png");
const HALF_FOOD_TEXTURE: ImageSource = egui::include_image!("../../../assets/food_half.png");
const EMPTY_FOOD_TEXTURE: ImageSource = egui::include_image!("../../../assets/food_empty.png");

pub struct ManualModule {
    offsets: [Offset; 10],
    food_amount: i32,
    max_hours: i32,
    low_health: bool,
    last_result: Vec<i32>,
    history: Vec<Vec<i32>>,
}

impl Default for ManualModule {
    fn default() -> Self {
        Self {
            offsets: [Offset::Neutral; 10],
            food_amount: 20,
            max_hours: 8,
            low_health: false,
            last_result: Vec::new(),
            history: Vec::new(),
        }
    }
}

fn tick_to_time(tick: i32) -> String {
    let total_seconds = tick / 20;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

impl ManualModule {
    pub fn render_aside(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let total = self.offsets.len();
            for (index, jolt) in self.offsets.iter_mut().enumerate() {
                let food_slot = (total - 1 - index) as i32;

                let texture = if self.food_amount >= (food_slot * 2 + 2) {
                    FULL_FOOD_TEXTURE
                } else if self.food_amount >= (food_slot * 2 + 1) {
                    HALF_FOOD_TEXTURE
                } else {
                    EMPTY_FOOD_TEXTURE
                };

                jolt.ui(texture, ui);
            }
        });
        ui.separator();

        ui.horizontal_wrapped(|ui| {
            ui.horizontal(|ui| {
                ui.label("Food Amount:");
                ui.add(
                    egui::DragValue::new(&mut self.food_amount)
                        .range(0..=20)
                        .speed(-0.1),
                );
            });

            ui.horizontal(|ui| {
                ui.label("Max Hours:");
                ui.add(egui::DragValue::new(&mut self.max_hours).range(0..=100));
            });

            ui.horizontal(|ui| {
                ui.label("Low Health:");
                ui.checkbox(&mut self.low_health, "");
            });
        });

        ui.separator();

        ui.horizontal(|ui| {
            let button_width = (ui.available_width() - ui.spacing().item_spacing.x) / 2.0;
            if ui
                .add_sized(
                    egui::vec2(button_width, 32.0),
                    egui::Button::new(RichText::new("Crack").size(20.0)),
                )
                .clicked()
            {
                let cracker = Cracker {
                    low_health: self.low_health,
                    max_tick: self.max_hours * 60 * 60 * 20,
                    matches: 100,
                    ..Default::default()
                };
                let jolt = Jolt {
                    food_amount: self.food_amount,
                    pattern: self.offsets,
                };
                let res = cracker.crack_ticks(&jolt);
                self.last_result = res.clone();
                if !res.is_empty() {
                    self.history.push(res);
                }
            };

            if ui
                .add_sized(
                    egui::vec2(button_width, 32.0),
                    egui::Button::new(RichText::new("Reset").size(20.0)),
                )
                .clicked()
            {
                *self = Self {
                    history: self.history.clone(),
                    ..Default::default()
                }
            }
        });
    }

    fn render_result(&self, result: i32, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(tick_to_time(result));
            ui.label(format!("({} ticks)", result));
        });
    }

    pub fn render_results(&self, ui: &mut egui::Ui) {
        if self.last_result.is_empty() {
            ui.label("No results");
            return;
        }

        for (index, result) in self.last_result.iter().enumerate() {
            ui.horizontal_wrapped(|ui| {
                ui.label(format!("{}.", index + 1));
                self.render_result(*result, ui);
            });
            if index != self.last_result.len() - 1 {
                ui.separator();
            }
        }
    }

    pub fn render_history(&mut self, ui: &mut egui::Ui) {
        if self.history.is_empty() {
            ui.label("No history");
            return;
        }

        for (index, result) in self.history.iter().enumerate().rev() {
            for tick in result {
                self.render_result(*tick, ui);
            }
            if index != 0 {
                ui.separator();
            }
        }
    }
}

impl AppModule for ManualModule {
    fn ui(&mut self, ui: &mut egui::Ui, _global: &mut crate::app::GlobalState) {
        egui::Panel::left("simple_aside")
            .resizable(false)
            .show(ui, |ui| {
                self.render_aside(ui);
            });

        egui::Panel::right("simple_history")
            .resizable(false)
            .show(ui, |ui| {
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.add_space(8.0);
                    if ui.button("Clear History").clicked() {
                        self.history.clear();
                    }
                    ui.add_space(8.0);
                    ui.separator();

                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        ui.heading("History");
                        ui.add_space(10.0);
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            self.render_history(ui);
                        });
                    });
                });
            });

        egui::CentralPanel::default().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Results");
                ui.add_space(10.0);
                self.render_results(ui);
            });
        });
    }

    fn name(&self) -> &str {
        "Manual"
    }
}
