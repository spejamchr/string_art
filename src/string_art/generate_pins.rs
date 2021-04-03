use super::geometry::Point;
use rand::RngCore;

pub fn perimeter(desired_count: u32, width: u32, height: u32) -> Vec<Point> {
    let desired_count = u32::max(4, desired_count);
    let spacingf = f64::max(
        1.0,
        ((width + height - 2) * 2) as f64 / desired_count as f64,
    );
    let countf = ((width + height - 2) * 2) as f64 / spacingf;
    let ratio = width as f64 / height as f64;
    let h_countf = countf / 2.0 * ratio / (1.0 + ratio);
    let v_countf = countf / 2.0 - h_countf;

    let horizontal_count = h_countf.round() as u32;
    let vertical_count = v_countf.round() as u32;
    let h_spacingf = (width as f64) / (horizontal_count as f64);
    let v_spacingf = (height as f64) / (vertical_count as f64);

    let top = (0..horizontal_count).map(|i| Point::new(f_mul(i, h_spacingf), 0));
    let bottom =
        (0..horizontal_count).map(|i| Point::new(width - f_mul(i, h_spacingf) - 1, height - 1));
    let left = (0..vertical_count).map(|i| Point::new(0, height - f_mul(i, v_spacingf) - 1));
    let right = (0..vertical_count).map(|i| Point::new(width - 1, f_mul(i, v_spacingf)));

    top.chain(right).chain(bottom).chain(left).collect()
}

fn f_mul(i: u32, f: f64) -> u32 {
    (i as f64 * f) as u32
}

pub fn grid(desired_count: u32, width: u32, height: u32) -> Vec<Point> {
    let r = width as f64 / height as f64;
    let x = (desired_count as f64 * r).sqrt().round() as usize;
    let y = (desired_count as f64 / r).sqrt().round() as usize;
    let dx = width as f64 / x as f64;
    let dy = height as f64 / y as f64;
    let max_x = width - 1;
    let max_y = height - 1;

    (0..=y)
        .flat_map(|j| {
            (0..=x).map(move |i| {
                Point::new(
                    u32::min(max_x, (i as f64 * dx) as u32),
                    u32::min(max_y, (j as f64 * dy) as u32),
                )
            })
        })
        .collect()
}

pub fn random(desired_count: u32, width: u32, height: u32) -> Vec<Point> {
    let mut rng = rand::thread_rng();
    (0..desired_count)
        .map(|_| (rng.next_u32() % width, rng.next_u32() % height))
        .map(|(x, y)| Point::new(x, y))
        .collect()
}

pub fn circle(desired_count: u32, width: u32, height: u32) -> Vec<Point> {
    let center_x = (width - 1) as f64 / 2.0;
    let center_y = (height - 1) as f64 / 2.0;
    let radius = f64::min(center_x, center_y);
    let step_size = std::f64::consts::PI * 2.0 / desired_count as f64;
    (0..desired_count)
        .map(|step| {
            Point::new(
                ((radius * (step as f64 * step_size).cos()).round() + center_x) as u32,
                ((radius * (step as f64 * step_size).sin()).round() + center_y) as u32,
            )
        })
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_specifying_too_few_pins_returns_minimum() {
        let pins = perimeter(0, 1234, 1234);
        assert_eq!(4, pins.len())
    }

    #[test]
    fn test_specifying_too_many_pins_returns_maximum() {
        let pins = perimeter(60, 10, 10);
        assert_eq!(36, pins.len())
    }

    #[test]
    fn test_generate_pins() {
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
}
