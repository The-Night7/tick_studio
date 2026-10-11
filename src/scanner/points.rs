pub fn required_extents(scale: f64, food_w: u32, food_h: u32) -> (u32, u32) {
    let max_x = (9.0 * 8.0 * scale).round() as u32;
    let max_y = (2.0 * scale).round() as u32;
    (max_x + food_w, max_y + food_h)
}

pub fn prepare_points(
    scale: f64,
    bbox_w: u32,
    bbox_h: u32,
    food_w: u32,
    food_h: u32,
) -> [[(u32, u32); 3]; 10] {
    let max_x = bbox_w.saturating_sub(food_w);
    let max_y = bbox_h.saturating_sub(food_h);
    let mut points = [[(0u32, 0u32); 3]; 10];
    for p in 0..10 {
        let screen_x = ((p as f64 * 8.0 * scale).round() as u32).min(max_x);

        for off in 0..3 {
            let screen_y = ((off as f64 * scale).round() as u32).min(max_y);

            points[p as usize][off as usize] = (screen_x, screen_y);
        }
    }

    points
}
