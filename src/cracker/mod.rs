mod rand;
use rand::JavaRandom;
use serde::{Deserialize, Serialize};

use crate::common::Jolt;

fn check_tick(tick: i32, pattern: &[i32; 10], low_health: bool) -> bool {
    let mut rand = JavaRandom::with_seed(tick.wrapping_mul(312871) as u64);
    if low_health {
        rand.skip_n(10);
    }
    for &expected in pattern.iter().rev() {
        let off = rand.next_int_n(3) - 1;
        if off != expected {
            return false;
        }
    }
    true
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cracker {
    pub low_health: bool,
    pub min_tick: i32,
    pub max_tick: i32,
    pub matches: usize,
}

impl Default for Cracker {
    fn default() -> Self {
        Self {
            low_health: false,
            min_tick: 0,
            max_tick: i32::MAX,
            matches: 1,
        }
    }
}

impl Cracker {
    pub fn crack_ticks(&self, jolt: &Jolt) -> Vec<i32> {
        let pattern = jolt.pattern.map(|offset| offset.into());
        let step = jolt.food_amount * 3 + 1;

        if step <= 0 {
            return Vec::new();
        }

        let rem = self.min_tick.rem_euclid(step);
        let start_tick = if rem == 0 {
            self.min_tick
        } else {
            match self.min_tick.checked_add(step - rem) {
                Some(aligned) => aligned,
                None => return Vec::new(),
            }
        };

        (start_tick..=self.max_tick)
            .step_by(step as usize)
            .filter(|&tick| check_tick(tick, &pattern, self.low_health))
            .take(self.matches)
            .collect()
    }
}
