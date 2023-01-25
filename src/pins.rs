use crate::geometry::Point;
use crate::rand::RngCore;
use crate::serde::Serialize;
use std::collections::HashSet;

const P: fn(u32, u32) -> Point = Point::new;

pub fn generate(
    pin_arrangement: &PinArrangement,
    desired_count: u32,
    width: u32,
    height: u32,
) -> Vec<Point> {
    generator(pin_arrangement)(desired_count, width, height)
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum PinArrangement {
    Perimeter,
    Grid,
    Circle,
    Random,
}

impl core::str::FromStr for PinArrangement {
    type Err = String;
    fn from_str(string: &str) -> std::result::Result<Self, Self::Err> {
        match string {
            "perimeter" => Ok(PinArrangement::Perimeter),
            "grid" => Ok(PinArrangement::Grid),
            "circle" => Ok(PinArrangement::Circle),
            "random" => Ok(PinArrangement::Random),
            _ => Err(format!("Invalid pin arrangement: \"{}\"", string)),
        }
    }
}

fn generator(pin_arrangement: &PinArrangement) -> fn(u32, u32, u32) -> Vec<Point> {
    match pin_arrangement {
        PinArrangement::Perimeter => perimeter,
        PinArrangement::Grid => grid,
        PinArrangement::Circle => circle,
        PinArrangement::Random => random,
    }
}

fn perimeter(desired_count: u32, width: u32, height: u32) -> Vec<Point> {
    let perimeter_pixels = (width + height - 2) * 2;
    let spacing = f64::max(1.0, perimeter_pixels as f64 / desired_count as f64);
    let count = perimeter_pixels as f64 / spacing;
    let ratio = width as f64 / height as f64;
    let h_count = count / 2.0 * ratio / (1.0 + ratio);
    let v_count = count / 2.0 - h_count;

    let h_count = h_count.round() as u32;
    let v_count = v_count.round() as u32;
    let h_spacing = (width as f64) / (h_count as f64);
    let v_spacing = (height as f64) / (v_count as f64);

    let top = (0..h_count).map(|i| P(f_mul(i, h_spacing), 0));
    let bottom = (0..h_count).map(|i| P(width - f_mul(i, h_spacing) - 1, height - 1));
    let left = (0..v_count).map(|i| P(0, height - f_mul(i, v_spacing) - 1));
    let right = (0..v_count).map(|i| P(width - 1, f_mul(i, v_spacing)));

    top.chain(right).chain(bottom).chain(left).collect()
}

fn f_mul(i: u32, f: f64) -> u32 {
    (i as f64 * f) as u32
}

fn grid(desired_count: u32, width: u32, height: u32) -> Vec<Point> {
    let ratio = width as f64 / height as f64;
    let x = u32::min(width, (desired_count as f64 * ratio).sqrt().round() as u32);
    let y = u32::min(height, (desired_count as f64 / ratio).sqrt().round() as u32);
    let dx = (width - 1) as f64 / (u32::max(x, 1) - 1) as f64;
    let dy = (height - 1) as f64 / (u32::max(y, 1) - 1) as f64;

    (0..y)
        .flat_map(|j| (0..x).map(move |i| P(f_mul(i, dx), f_mul(j, dy))))
        .collect()
}

fn random(desired_count: u32, width: u32, height: u32) -> Vec<Point> {
    let desired_count = u32::min(width * height, desired_count);
    let mut points = HashSet::new();
    let mut rng = rand::thread_rng();
    loop {
        if points.len() == desired_count as usize {
            return points.into_iter().collect();
        } else {
            points.insert(P(rng.next_u32() % width, rng.next_u32() % height));
        }
    }
}

fn circle(desired_count: u32, width: u32, height: u32) -> Vec<Point> {
    let center_x = (width - 1) as f64 / 2.0;
    let center_y = (height - 1) as f64 / 2.0;
    let radius = f64::min(center_x, center_y);
    let step_size = std::f64::consts::PI * 2.0 / desired_count as f64;
    (0..desired_count).fold(Vec::new(), |mut points, step| {
        let point = P(
            ((radius * (step as f64 * step_size).cos()).round() + center_x) as u32,
            ((radius * (step as f64 * step_size).sin()).round() + center_y) as u32,
        );
        if points.iter().all(|p| p != &point) {
            points.push(point)
        }
        points
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_perimeter_specifying_0_points_works() {
        let pins = perimeter(0, 1234, 1234);
        assert_eq!(0, pins.len())
    }

    #[test]
    fn test_grid_specifying_0_points_works() {
        let pins = grid(0, 1234, 1234);
        assert_eq!(0, pins.len())
    }

    #[test]
    fn test_random_specifying_0_points_works() {
        let pins = random(0, 1234, 1234);
        assert_eq!(0, pins.len())
    }

    #[test]
    fn test_circle_specifying_0_points_works() {
        let pins = circle(0, 1234, 1234);
        assert_eq!(0, pins.len())
    }

    #[test]
    fn test_perimeter_specifying_too_many_pins_returns_maximum() {
        let pins = perimeter(60, 10, 10);
        assert_eq!(36, pins.len())
    }

    #[test]
    fn test_grid_specifying_too_many_pins_returns_maximum() {
        let pins = grid(600, 10, 10);
        assert_eq!(100, pins.len())
    }

    #[test]
    fn test_random_specifying_too_many_pins_returns_maximum() {
        let pins = random(600, 10, 10);
        assert_eq!(100, pins.len())
    }

    #[test]
    fn test_circle_specifying_too_many_pins_returns_maximum() {
        let pins = circle(600, 10, 10);
        assert_eq!(34, pins.len())
    }

    #[test]
    fn test_perimeter_generate_pins_count() {
        for count in [4, 8, 16, 60, 120, 200, 400, 1000].iter() {
            for (width, height) in [(123, 457), (2880, 1800), (1234, 5678), (10, 10000)].iter() {
                let pins = perimeter(*count, *width, *height);
                assert_eq!(
                    *count,
                    pins.len() as u32,
                    "failed on count: {}, width: {}, height: {}",
                    count,
                    width,
                    height
                );
            }
        }
    }

    #[test]
    fn test_perimeter_generate_pins_locations() {
        assert_eq!(
            vec![
                P(0, 0),
                P(12, 0),
                P(24, 0),
                P(24, 12),
                P(24, 24),
                P(12, 24),
                P(0, 24),
                P(0, 12)
            ],
            perimeter(8, 25, 25)
        )
    }

    #[test]
    fn test_grid_generate_pins_locations() {
        assert_eq!(
            vec![
                P(0, 0),
                P(12, 0),
                P(24, 0),
                P(0, 12),
                P(12, 12),
                P(24, 12),
                P(0, 24),
                P(12, 24),
                P(24, 24),
            ],
            grid(9, 25, 25)
        )
    }
}
