use std::collections::HashSet;

use crate::{cracker::Cracker, modules::explorer::ExplorerPoint};

pub struct AutoCleanupSettings {
    pub min_line_length: i32,
    pub tolerance: i32,
    pub max_gap: i32,
}

impl Default for AutoCleanupSettings {
    fn default() -> Self {
        Self {
            min_line_length: 2,
            tolerance: 0,
            max_gap: 300,
        }
    }
}

struct Line {
    height: i32,
    points: Vec<usize>,
}

impl Line {
    pub fn new(point: &ExplorerPoint, point_index: usize) -> Self {
        Self {
            height: point.tick - point.time,
            points: vec![point_index],
        }
    }

    fn is_on_line(&self, point: &ExplorerPoint, tolerance: i32) -> bool {
        let height = point.tick - point.time;
        let err = (height - self.height).abs();
        err <= tolerance
    }
}
pub fn cleanup_data(
    points: Vec<ExplorerPoint>,
    settings: &AutoCleanupSettings,
) -> Vec<ExplorerPoint> {
    println!("Starting cleanup with {} points", points.len());

    let mut points = points;
    points.sort_by_key(|p| (p.time, p.tick));
    points.dedup_by_key(|p| (p.time, p.tick));

    let recrackers = vec![
        Cracker {
            low_health: true,
            matches: 3,
            ..Default::default()
        },
        Cracker {
            matches: 3,
            ..Default::default()
        },
    ];

    let mut all_points = vec![];
    all_points.extend(points.clone());
    for recracker in recrackers {
        for point in points.iter() {
            let ticks = recracker.crack_ticks(&point.jolt);
            for tick in ticks {
                let mut new_point = point.clone();
                new_point.tick = tick;
                new_point.low_health |= recracker.low_health;
                all_points.push(new_point);
            }
        }
    }

    all_points.sort_by_key(|p| (p.time, p.tick));
    all_points.dedup_by_key(|p| (p.time, p.tick));

    let mut points_out = Vec::<ExplorerPoint>::new();
    let mut lines = Vec::<Line>::new();
    let mut explored: HashSet<usize> = HashSet::new();

    for (index, point) in all_points.iter().enumerate() {
        if explored.contains(&index) {
            continue;
        }
        let mut new_line = Line::new(point, index);
        for (index2, point2) in all_points.iter().skip(index + 1).enumerate() {
            if explored.contains(&(index2 + index + 1)) {
                continue;
            }
            if new_line.is_on_line(point2, settings.tolerance) {
                let last_time = all_points[*new_line.points.last().unwrap()].time;
                if point2.time - last_time > settings.max_gap * 20 {
                    break;
                }
                new_line.points.push(index2 + index + 1);
            }
        }
        if new_line.points.len() >= settings.min_line_length as usize {
            for point in &new_line.points {
                explored.insert(*point);
                points_out.push(all_points[*point].clone());
            }
            lines.push(new_line);
        }
    }

    points_out.sort_by_key(|p| p.time);
    points_out.dedup_by_key(|p| p.time);

    println!("Cleanup finished, {} points remaining", points_out.len());

    points_out
}
