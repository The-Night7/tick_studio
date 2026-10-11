#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[tokio::main]
async fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 600.0])
            .with_min_inner_size([1100.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Tick Studio",
        native_options,
        Box::new(|cc| Ok(Box::new(tick_studio::App::new(cc)))),
    )
}
