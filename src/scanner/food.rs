use crate::scanner::data::CrackingSettings;

pub fn prepare_food_images(
    settings: &CrackingSettings,
) -> Result<([image::RgbImage; 3], Vec<f64>), String> {
    let food_img = image::load_from_memory(include_bytes!("../../assets/food.png"))
        .map_err(|e| format!("Failed to load food image: {}", e))?;
    let food_half_img = image::load_from_memory(include_bytes!("../../assets/food_half.png"))
        .map_err(|e| format!("Failed to load food half image: {}", e))?;
    let food_empty_img = image::load_from_memory(include_bytes!("../../assets/food_empty.png"))
        .map_err(|e| format!("Failed to load food empty image: {}", e))?;

    let bbox = settings.rect;
    let scale = bbox.2 as f64 / 81.0;

    let food_w = (9.0f64 * scale).round() as u32;
    let food_h = (9.0f64 * scale).round() as u32;

    if food_w == 0 || food_h == 0 {
        return Err(format!(
            "Hunger bbox width {} too small to scale food templates",
            bbox.2
        ));
    }

    let foods_raw = [&food_empty_img, &food_half_img, &food_img];

    let resized_foods: Vec<image::DynamicImage> = foods_raw
        .iter()
        .map(|&f| f.resize_exact(food_w, food_h, image::imageops::FilterType::Nearest))
        .collect();

    let alpha: Vec<f64> = resized_foods[0]
        .to_rgba8()
        .pixels()
        .zip(resized_foods[1].to_rgba8().pixels())
        .zip(resized_foods[2].to_rgba8().pixels())
        .map(|((p0, p1), p2)| {
            let a0 = p0[3] as f64;
            let a1 = p1[3] as f64;
            let a2 = p2[3] as f64;
            a0.max(a1).max(a2) / 255.0
        })
        .collect();

    let foods: [image::RgbImage; 3] = [
        resized_foods[0].to_rgb8(),
        resized_foods[1].to_rgb8(),
        resized_foods[2].to_rgb8(),
    ];

    Ok((foods, alpha))
}
