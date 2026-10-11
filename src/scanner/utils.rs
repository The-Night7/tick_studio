use crate::common::{Jolt, Offset};

pub fn check_jolt(jolt: &Jolt) -> bool {
    if jolt.pattern.iter().all(|f| *f == Offset::Neutral) {
        return false;
    }

    let mut up_count = 0;
    let mut neutral_count = 0;
    let mut down_count = 0;

    for offset in &jolt.pattern {
        match offset {
            Offset::Up => up_count += 1,
            Offset::Neutral => neutral_count += 1,
            Offset::Down => down_count += 1,
        }
    }

    let most_common_count = up_count.max(neutral_count).max(down_count);

    most_common_count < 6
}

pub fn frame_to_tick(frame_count: i32, fps: f64) -> i32 {
    let seconds = frame_count as f64 / fps;
    (seconds * 20.0).round() as i32
}
