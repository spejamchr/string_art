use super::geometry::{Line, Point};
use crate::image::DynamicImage;
use crate::image::GenericImageView;
use crate::inout::Data;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct RGB {
    pub r: i64,
    pub g: i64,
    pub b: i64,
}

pub type LineSegment = (Point, Point, RGB);

impl RGB {
    pub const WHITE: Self = RGB {
        r: 255,
        g: 255,
        b: 255,
    };

    pub const BLACK: Self = RGB { r: 0, g: 0, b: 0 };

    pub fn new<T>(r: T, g: T, b: T) -> Self
    where
        T: Into<i64>,
    {
        Self {
            r: r.into(),
            g: g.into(),
            b: b.into(),
        }
    }

    fn clamped(&self) -> Self {
        Self::new(u8_clamp(self.r), u8_clamp(self.g), u8_clamp(self.b))
    }
}

fn u8_clamp(n: i64) -> u8 {
    i64::max(u8::MIN.into(), i64::min(u8::MAX.into(), n)) as u8
}

impl std::fmt::Display for RGB {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let rgb = self.clamped();
        write!(f, "#{:0>2X}{:0>2X}{:0>2X}", rgb.r, rgb.g, rgb.b)
    }
}

impl<T: Into<Self>> std::ops::Add<T> for RGB {
    type Output = Self;
    fn add(self, rhs: T) -> Self {
        let rgb = rhs.into();
        Self::new(
            self.r.saturating_add(rgb.r),
            self.g.saturating_add(rgb.g),
            self.b.saturating_add(rgb.b),
        )
    }
}

impl<T: Into<Self>> std::ops::Sub<T> for RGB {
    type Output = Self;
    fn sub(self, rhs: T) -> Self {
        let rgb = rhs.into();
        Self::new(
            self.r.saturating_sub(rgb.r),
            self.g.saturating_sub(rgb.g),
            self.b.saturating_sub(rgb.b),
        )
    }
}

impl std::ops::Neg for RGB {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.r, -self.g, -self.b)
    }
}

#[derive(Clone, Copy)]
struct RGBf {
    r: f64,
    g: f64,
    b: f64,
}

impl RGBf {
    fn new(r: f64, g: f64, b: f64) -> Self {
        Self { r, g, b }
    }
}

impl std::ops::Add<Self> for RGBf {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.r + rhs.r, self.g + rhs.g, self.b + rhs.b)
    }
}

impl std::ops::Mul<f64> for RGBf {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Self::new(self.r * rhs, self.g * rhs, self.b * rhs)
    }
}

impl std::convert::From<RGB> for RGBf {
    fn from(rgb: RGB) -> Self {
        Self::new(rgb.r as f64, rgb.g as f64, rgb.b as f64)
    }
}

impl std::convert::From<RGBf> for RGB {
    fn from(rgbf: RGBf) -> Self {
        Self::new(
            rgbf.r.round() as i64,
            rgbf.g.round() as i64,
            rgbf.b.round() as i64,
        )
    }
}

impl<T: Into<i64>> std::convert::From<(T, T, T)> for RGB {
    fn from((r, g, b): (T, T, T)) -> Self {
        RGB::new(r, g, b)
    }
}

impl<T: Into<i64>> std::convert::From<[T; 3]> for RGB {
    fn from([r, g, b]: [T; 3]) -> Self {
        RGB::new(r, g, b)
    }
}

/// Line of pixels
pub struct PixLine(HashMap<Point, RGB>);

impl PixLine {
    fn into_iter(self) -> std::collections::hash_map::IntoIter<Point, RGB> {
        self.0.into_iter()
    }
}

impl<T: Into<Line>> std::convert::From<(T, RGB, f64, f64)> for PixLine {
    fn from((line, rgb, step_size, string_alpha): (T, RGB, f64, f64)) -> Self {
        let coloring_val = RGBf::from(rgb) * step_size * string_alpha;
        Self(
            line.into()
                .iter(step_size)
                .map(Point::from)
                .fold(HashMap::new(), |mut hash, point| {
                    if let Some(old) = hash.insert(point, coloring_val) {
                        hash.insert(point, old + coloring_val);
                    }
                    hash
                })
                .into_iter()
                .map(|(point, rgbf)| (point, RGB::from(rgbf)))
                .collect::<HashMap<_, _>>(),
        )
    }
}

#[derive(Debug)]
pub struct RefImage(Vec<Vec<RGB>>);

impl RefImage {
    pub fn new(width: u32, height: u32) -> Self {
        Self(vec![vec![RGB::BLACK; width as usize]; height as usize])
    }

    pub fn negated(mut self) -> Self {
        self.0
            .iter_mut()
            .for_each(|row| row.iter_mut().for_each(|rgb| *rgb = -*rgb));
        self
    }

    pub fn add_rgb(mut self, other: RGB) -> Self {
        self.0
            .iter_mut()
            .for_each(|row| row.iter_mut().for_each(|rgb| *rgb = *rgb + other));
        self
    }

    pub fn score(&self) -> i64 {
        self.0.iter().flatten().map(pixel_score).sum()
    }

    pub fn score_change_if_added<T: Into<PixLine>>(&self, line: T) -> i64 {
        line.into()
            .into_iter()
            .map(|(p, rgb)| {
                let a = self[p];
                let b = a + rgb;
                pixel_score(&b) - pixel_score(&a)
            })
            .sum()
    }

    pub fn score_change_if_removed<T: Into<PixLine>>(&self, line: T) -> i64 {
        line.into()
            .into_iter()
            .map(|(p, rgb)| {
                let a = self[p];
                let b = a - rgb;
                pixel_score(&b) - pixel_score(&a)
            })
            .sum()
    }

    pub fn subtract_line<T: Into<PixLine>>(&mut self, line: T) -> &mut Self {
        *self -= line;
        self
    }

    pub fn add_line<T: Into<PixLine>>(&mut self, line: T) -> &mut Self {
        *self += line;
        self
    }

    pub fn width(&self) -> u32 {
        self.0[0].len() as u32
    }

    pub fn height(&self) -> u32 {
        self.0.len() as u32
    }

    pub fn color(&self) -> image::RgbaImage {
        let mut img = image::RgbaImage::new(self.width(), self.height());
        for (y, row) in self.0.iter().enumerate() {
            for (x, rgb) in row.iter().map(|rgb| rgb.clamped()).enumerate() {
                let pixel = img.get_pixel_mut(x as u32, y as u32);
                pixel[0] = rgb.r as u8;
                pixel[1] = rgb.g as u8;
                pixel[2] = rgb.b as u8;
                pixel[3] = u8::MAX; // Alpha channel
            }
        }
        img
    }
}

fn pixel_score(RGB { r, g, b }: &RGB) -> i64 {
    r * r + g * g + b * b
}

impl<T: Into<PixLine> + Copy> std::convert::From<(&Vec<T>, u32, u32)> for RefImage {
    fn from((line_segmentables, width, height): (&Vec<T>, u32, u32)) -> Self {
        let mut ref_image = Self::new(width, height);
        line_segmentables
            .iter()
            .fold(&mut ref_image, |i, a| i.add_line(*a));
        ref_image
    }
}

impl std::convert::From<DynamicImage> for RefImage {
    fn from(image: DynamicImage) -> Self {
        let mut ref_image = Self::new(image.width(), image.height());
        image.to_rgb8().enumerate_pixels().for_each(|(x, y, p)| {
            ref_image[(x, y)] = RGB::from(p.0);
        });
        ref_image
    }
}

impl std::convert::From<&Data> for RefImage {
    fn from(data: &Data) -> Self {
        Self::from((
            &data
                .line_segments
                .iter()
                .map(|(a, b, rgb)| ((*a, *b), *rgb, data.args.step_size, data.args.string_alpha))
                .collect(),
            data.image_width,
            data.image_height,
        ))
        .add_rgb(data.args.background_color)
    }
}

impl<T: Into<PixLine>> std::ops::AddAssign<T> for RefImage {
    fn add_assign(&mut self, pix_line: T) {
        pix_line.into().into_iter().for_each(|(point, rgb)| {
            self[point] = self[point] + rgb;
        })
    }
}

impl<T: Into<PixLine>> std::ops::SubAssign<T> for RefImage {
    fn sub_assign(&mut self, pix_line: T) {
        pix_line.into().into_iter().for_each(|(point, rgb)| {
            self[point] = self[point] - rgb;
        })
    }
}

impl std::ops::Index<Point> for RefImage {
    type Output = RGB;
    fn index(&self, point: Point) -> &Self::Output {
        &self.0[point.y as usize][point.x as usize]
    }
}

impl std::ops::Index<(u32, u32)> for RefImage {
    type Output = RGB;
    fn index(&self, (x, y): (u32, u32)) -> &Self::Output {
        &self.0[y as usize][x as usize]
    }
}

impl std::ops::IndexMut<Point> for RefImage {
    fn index_mut(&mut self, point: Point) -> &mut Self::Output {
        &mut self.0[point.y as usize][point.x as usize]
    }
}

impl std::ops::IndexMut<(u32, u32)> for RefImage {
    fn index_mut(&mut self, (x, y): (u32, u32)) -> &mut Self::Output {
        &mut self.0[y as usize][x as usize]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_rgb_to_string() {
        assert_eq!("#000000", RGB::BLACK.to_string());
        assert_eq!("#FFFFFF", RGB::WHITE.to_string());
        assert_eq!("#123456", RGB::new(18, 52, 86).to_string());
        assert_eq!("#00FF56", RGB::new(-18, 520, 86).to_string()); // Clamp to u8 range
    }

    #[test]
    fn test_rgb_add() {
        assert_eq!(
            RGB::new(10, 20, 30),
            RGB::new(5, 15, 27) + RGB::new(5, 5, 3)
        );
    }

    #[test]
    fn test_rgb_sub() {
        assert_eq!(RGB::new(0, 10, 24), RGB::new(5, 15, 27) - RGB::new(5, 5, 3));
    }

    #[test]
    fn test_rgb_neg() {
        assert_eq!(RGB::new(-5, -5, -3), -RGB::new(5, 5, 3));
    }

    #[test]
    fn test_pix_line() {
        let line = PixLine::from(((Point::new(0, 0), Point::new(0, 2)), RGB::WHITE, 1.0, 0.2));
        assert_eq!(
            vec![
                (Point::new(0, 0), RGB::new(51, 51, 51)),
                (Point::new(0, 1), RGB::new(51, 51, 51)),
                (Point::new(0, 2), RGB::new(51, 51, 51))
            ]
            .into_iter()
            .collect::<HashMap<_, _>>(),
            line.0
        );
    }

    #[test]
    fn test_new_ref_image_is_black() {
        assert_eq!(vec![vec![RGB::BLACK]], RefImage::new(1, 1).0);
    }

    #[test]
    fn test_ref_image_add_rgb() {
        assert_eq!(
            vec![vec![RGB::WHITE]],
            RefImage::new(1, 1).add_rgb(RGB::WHITE).0
        );
    }

    #[test]
    fn test_ref_image_negated() {
        assert_eq!(
            vec![vec![-RGB::WHITE]],
            RefImage::new(1, 1).add_rgb(RGB::WHITE).negated().0
        );
    }

    #[test]
    fn test_black_ref_image_score_is_zero() {
        assert_eq!(0, RefImage::new(500, 500).score());
    }

    #[test]
    fn test_white_ref_image_score() {
        assert_eq!(
            3 * 255 * 255,
            RefImage::new(1, 1).add_rgb(RGB::WHITE).score()
        );
    }

    #[test]
    fn test_inverted_white_ref_image_score() {
        assert_eq!(
            3 * 255 * 255,
            RefImage::new(1, 1).add_rgb(RGB::WHITE).negated().score()
        )
    }

    #[test]
    fn test_score_change_if_added_is_accurate() {
        let pix_line = || {
            PixLine::from((
                (Point::new(0, 0), Point::new(101, 67)),
                RGB::WHITE,
                1.0,
                1.0,
            ))
        };
        let mut ref_image = RefImage::new(150, 150);
        let predicted_score = ref_image.score_change_if_added(pix_line());
        ref_image.add_line(pix_line());
        let real_score = ref_image.score();
        assert_eq!(real_score, predicted_score);
    }

    #[test]
    fn test_score_change_if_removed_is_accurate() {
        let pix_line = || {
            PixLine::from((
                (Point::new(0, 0), Point::new(101, 67)),
                RGB::WHITE,
                1.0,
                1.0,
            ))
        };
        let mut ref_image = RefImage::new(150, 150);
        let predicted_score = ref_image.score_change_if_removed(pix_line());
        ref_image.subtract_line(pix_line());
        let real_score = ref_image.score();
        assert_eq!(real_score, predicted_score);
    }

    #[test]
    fn test_ref_image_width() {
        assert_eq!(5, RefImage::new(5, 1).width());
    }

    #[test]
    fn test_ref_image_height() {
        assert_eq!(5, RefImage::new(1, 5).height());
    }

    #[test]
    fn test_ref_image_color() {
        // Create a ref image where each pixel is unique
        let mut ref_image = RefImage::new(400, 400);
        ref_image
            .0
            .iter_mut()
            .flatten()
            .enumerate()
            .for_each(|(i, rgb)| {
                *rgb = RGB::new(
                    ((i / 255 / 255) % 255) as i64,
                    ((i / 255) % 255) as i64,
                    (i % 255) as i64,
                )
            });

        let ref_pixels: Vec<_> = ref_image
            .0
            .iter()
            .flatten()
            .map(|RGB { r, g, b }| [*r as u8, *g as u8, *b as u8, 255])
            .collect();

        let pixels: Vec<_> = ref_image.color().pixels().map(|p| p.0).collect();

        assert_eq!(ref_pixels, pixels);
    }
}
