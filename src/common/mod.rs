pub mod ui;

mod task;
pub use task::{CancelToken, Task};
pub mod formats;

use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Offset {
    Up,
    Down,
    #[default]
    Neutral,
}

impl Offset {
    pub fn up(&mut self) {
        *self = match self {
            Offset::Up => Offset::Up,
            Offset::Down => Offset::Neutral,
            Offset::Neutral => Offset::Up,
        }
    }

    pub fn down(&mut self) {
        *self = match self {
            Offset::Up => Offset::Neutral,
            Offset::Down => Offset::Down,
            Offset::Neutral => Offset::Down,
        }
    }
}

impl From<Offset> for i32 {
    fn from(val: Offset) -> Self {
        match val {
            Offset::Up => -1,
            Offset::Down => 1,
            Offset::Neutral => 0,
        }
    }
}
impl Serialize for Offset {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_i32(i32::from(*self))
    }
}

impl<'de> Deserialize<'de> for Offset {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match i32::deserialize(deserializer)? {
            -1 => Ok(Offset::Up),
            0 => Ok(Offset::Neutral),
            1 => Ok(Offset::Down),
            other => Err(serde::de::Error::custom(format!(
                "invalid offset {other}, expected -1 (up), 0 (neutral), or 1 (down)"
            ))),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Jolt {
    pub food_amount: i32,
    pub pattern: [Offset; 10],
}

impl Display for Jolt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pattern_str: String = self
            .pattern
            .iter()
            .map(|offset| match offset {
                Offset::Up => '+',
                Offset::Down => '-',
                Offset::Neutral => '0',
            })
            .collect();
        write!(f, "{} {}", self.food_amount, pattern_str)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Point {
    pub tick: i32,
    pub time: i32,
    pub jolt: Jolt,
    #[serde(default)]
    pub low_health: bool,
}

pub fn tick_to_time(tick: i32) -> Result<jiff::Timestamp, String> {
    jiff::Timestamp::from_second(tick as i64 / 20)
        .map_err(|e| format!("Failed to convert tick to time: {}", e))
}
