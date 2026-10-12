#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[tokio::main]
async fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 700.0])
            .with_min_inner_size([640.0, 420.0])
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "Tick Studio",
        native_options,
        Box::new(|cc| Ok(Box::new(tick_studio::App::new(cc)))),
    )
}
