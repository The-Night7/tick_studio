use egui::{Color32, Pos2, emath::TSTransform};

pub fn draw_grid(
    painter: &egui::Painter,
    screen_rect: egui::Rect,
    to_screen: TSTransform,
    zoom: f32,
) {
    let from_screen = to_screen.inverse();

    let min_world = from_screen * screen_rect.min;
    let max_world = from_screen * screen_rect.max;

    const MIN_PIXEL_SPACING: f32 = 200.0;
    let raw_grid_size = MIN_PIXEL_SPACING / zoom;
    let grid_size = 10.0f32.powf(raw_grid_size.log10().floor());

    let line_color = Color32::from_gray(45);
    let stroke = (1.0, line_color);

    let start_x = (min_world.x.min(max_world.x) / grid_size).floor() as i64;
    let end_x = (min_world.x.max(max_world.x) / grid_size).ceil() as i64;

    let start_y = (min_world.y.min(max_world.y) / grid_size).floor() as i64;
    let end_y = (min_world.y.max(max_world.y) / grid_size).ceil() as i64;

    for x_idx in start_x..=end_x {
        let world_x = x_idx as f32 * grid_size;
        let top = to_screen * Pos2::new(world_x, min_world.y);
        let bottom = to_screen * Pos2::new(world_x, max_world.y);
        painter.line_segment([top, bottom], stroke);
    }

    for y_idx in start_y..=end_y {
        let world_y = y_idx as f32 * grid_size;
        let left = to_screen * Pos2::new(min_world.x, world_y);
        let right = to_screen * Pos2::new(max_world.x, world_y);
        painter.line_segment([left, right], stroke);
    }
}
