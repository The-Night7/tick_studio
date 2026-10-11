use crate::modules::{
    explorer::ExplorerModule, scanner::ScanningModule, manual::ManualModule, youtube::YoutubeModule,
};

pub struct GlobalState {}

pub trait AppModule {
    fn ui(&mut self, ui: &mut egui::Ui, global: &mut GlobalState);
    fn logic(&mut self, _global: &mut GlobalState) {}
    fn name(&self) -> &str;
}

pub struct App {
    state: GlobalState,
    modules: Vec<Box<dyn AppModule>>,
    current_tab: usize,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        egui_material_icons::initialize(&cc.egui_ctx);
        Self::default()
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            state: GlobalState {},
            modules: vec![
                Box::new(ManualModule::default()),
                Box::new(YoutubeModule::default()),
                Box::new(ScanningModule::default()),
                Box::new(ExplorerModule::default()),
            ],
            current_tab: 0,
        }
    }
}

impl eframe::App for App {
    fn logic(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(module) = self.modules.get_mut(self.current_tab) {
            module.logic(&mut self.state);
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let dark_panel_frame =
            egui::Frame::side_top_panel(ui.style()).fill(egui::Color32::from_rgb(20, 20, 24));

        egui::Panel::top("top_panel")
            .frame(dark_panel_frame)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    for (index, module) in self.modules.iter().enumerate() {
                        if ui
                            .selectable_label(self.current_tab == index, module.name())
                            .clicked()
                        {
                            self.current_tab = index;
                        }
                    }
                });
            });

        egui::CentralPanel::default().show(ui, |ui| {
            if let Some(module) = self.modules.get_mut(self.current_tab) {
                module.ui(ui, &mut self.state);
            }
        });
    }
}
