use image::GenericImageView;

use crate::common::{Jolt, Offset};

fn index_to_offset(index: usize) -> Offset {
    match index {
        0 => Offset::Up,
        1 => Offset::Neutral,
        2 => Offset::Down,
        _ => panic!("Invalid index for Offset"),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoodState {
    Empty,
    Half,
    Full,
}

impl FoodState {
    pub fn from_index(idx: usize) -> Self {
        match idx {
            0 => Self::Empty,
            1 => Self::Half,
            2 => Self::Full,
            _ => unreachable!("Invalid food state index"),
        }
    }
}

pub fn process_img(
    img: &image::RgbImage,
    foods: &[image::RgbImage; 3],
    alpha: &[f64],
    points: &[[(u32, u32); 3]; 10],
    food_w: u32,
    food_h: u32,
) -> Result<Jolt, String> {
    let mut offsets = Vec::with_capacity(10);
    let mut food_amount: Option<u32> = None;

    for (i, point_set) in points.iter().enumerate() {
        let mut min_diff = f64::MAX;
        let mut best_off = 0;
        let mut best_food = 0;

        for (off, &(rel_x, rel_y)) in point_set.iter().enumerate() {
            if rel_x.saturating_add(food_w) > img.width()
                || rel_y.saturating_add(food_h) > img.height()
            {
                return Err(format!(
                    "Sample point ({rel_x}, {rel_y}) with template {food_w}x{food_h} exceeds frame {}x{}",
                    img.width(),
                    img.height()
                ));
            }
            let view = img.view(rel_x, rel_y, food_w, food_h);

            for (food_idx, food_rgb) in foods.iter().enumerate() {
                let diff_sum: f64 = view
                    .pixels()
                    .zip(food_rgb.pixels())
                    .zip(alpha)
                    .map(|(((_, _, p1), p2), a)| {
                        ((p1[0] as f64 - p2[0] as f64).abs()
                            + (p1[1] as f64 - p2[1] as f64).abs()
                            + (p1[2] as f64 - p2[2] as f64).abs())
                            * a
                    })
                    .sum();

                if diff_sum < min_diff {
                    min_diff = diff_sum;
                    best_off = off;
                    best_food = food_idx;
                }
            }
        }

        let offset = index_to_offset(best_off);
        let food_state = FoodState::from_index(best_food);

        if food_amount.is_none() {
            match food_state {
                FoodState::Full => food_amount = Some((10 - i as u32) * 2),
                FoodState::Half => food_amount = Some((10 - i as u32) * 2 - 1),
                FoodState::Empty => {}
            }
        }

        offsets.push(offset);
    }

    match food_amount {
        Some(amount) => Ok(Jolt {
            food_amount: amount as i32,
            pattern: offsets.try_into().unwrap(),
        }),
        None => Err("Could not determine food amount from image".into()),
    }
}
