use super::geometry::{Line, Point};
use crate::image::DynamicImage;
use crate::image::GenericImageView;
use crate::inout::Data;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RGB {
    pub r: i64,
    pub g: i64,
    pub b: i64,
}

impl RGB {
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

    pub fn white() -> Self {
        Self::new(u8::MAX, u8::MAX, u8::MAX)
    }

    pub fn black() -> Self {
        Self::new(u8::MIN, u8::MIN, u8::MIN)
    }

    pub fn inverted(&self) -> Self {
        Self::white() - *self
    }

    pub fn clamped(&self) -> Self {
        Self::new(u8_clamp(self.r), u8_clamp(self.g), u8_clamp(self.b))
    }
}

fn u8_clamp(n: i64) -> i64 {
    i64::max(u8::MIN.into(), i64::min(u8::MAX.into(), n))
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

// NOTE: This is different from RGB's `inverted()`
impl std::ops::Neg for RGB {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.r, -self.g, -self.b)
    }
}

impl std::ops::Mul<i64> for RGB {
    type Output = Self;
    fn mul(self, rhs: i64) -> Self {
        Self::new(
            self.r.saturating_mul(rhs),
            self.g.saturating_mul(rhs),
            self.b.saturating_mul(rhs),
        )
    }
}

#[derive(Clone, Copy)]
struct RGBf {
    r: f64,
    g: f64,
    b: f64,
}

impl RGBf {
    pub fn new(r: f64, g: f64, b: f64) -> Self {
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

/// Line of pixels
pub struct PixLine(HashMap<Point, RGB>);

impl PixLine {
    pub fn iter(&self) -> std::collections::hash_map::Iter<Point, RGB> {
        self.0.iter()
    }
}

impl<T: Into<Line>> std::convert::From<(T, RGB, f64, f64)> for PixLine {
    fn from((line, rgb, step_size, string_alpha): (T, RGB, f64, f64)) -> Self {
        let coloring_val = RGBf::from(rgb) * step_size * string_alpha;
        Self(
            line.into()
                .iter(step_size)
                .map(Point::from)
                .fold(HashMap::new(), |mut a, p| {
                    if let Some(old) = a.insert(p, coloring_val) {
                        a.insert(p, old + coloring_val);
                    }
                    a
                })
                .into_iter()
                .map(|(p, v)| (p, RGB::from(v)))
                .collect::<HashMap<_, _>>(),
        )
    }
}

#[derive(Debug)]
pub struct RefImage(Vec<Vec<(i64, i64, i64)>>);

impl RefImage {
    pub fn new(width: u32, height: u32) -> Self {
        Self(vec![vec![(0, 0, 0); width as usize]; height as usize])
    }

    pub fn inverted(mut self) -> Self {
        let max = u8::MAX as i64;
        self.0.iter_mut().for_each(|row| {
            row.iter_mut()
                .for_each(|v| *v = (max - v.0, max - v.1, max - v.2))
        });
        self
    }

    pub fn score(&self) -> i64 {
        self.0
            .iter()
            .flatten()
            .map(|pixel| pixel_score(pixel))
            .sum()
    }

    pub fn score_change_if_added<T: Into<PixLine>>(&self, line: T) -> i64 {
        line.into()
            .iter()
            .map(|(p, rgb)| {
                let a = self[p];
                let b = (a.0 + rgb.r as i64, a.1 + rgb.g as i64, a.2 + rgb.b as i64);
                pixel_score(&b) - pixel_score(&a)
            })
            .sum()
    }

    pub fn score_change_if_removed<T: Into<PixLine>>(&self, line: T) -> i64 {
        line.into()
            .iter()
            .map(|(p, rgb)| {
                let a = self[p];
                let b = (a.0 - rgb.r as i64, a.1 - rgb.g as i64, a.2 - rgb.b as i64);
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
            for (x, p) in row.iter().enumerate() {
                let pixel = img.get_pixel_mut(x as u32, y as u32);
                pixel[0] = i64_to_u8_clamped(p.0);
                pixel[1] = i64_to_u8_clamped(p.1);
                pixel[2] = i64_to_u8_clamped(p.2);
                pixel[3] = u8::MAX; // Alpha channel
            }
        }
        img
    }
}

fn pixel_score((r, g, b): &(i64, i64, i64)) -> i64 {
    let m = u8::MAX as i64;
    (m - r).saturating_pow(2) + (m - g).saturating_pow(2) + (m - b).saturating_pow(2)
}

fn i64_to_u8_clamped(num: i64) -> u8 {
    i64::max(u8::MIN as i64, i64::min(u8::MAX as i64, num)) as u8
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

impl std::convert::From<&DynamicImage> for RefImage {
    fn from(image: &DynamicImage) -> Self {
        let mut ref_image = Self::new(image.width(), image.height());
        image.to_rgb8().enumerate_pixels().for_each(|(x, y, p)| {
            ref_image[(x, y)] = (p[0].into(), p[1].into(), p[2].into());
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
    }
}

impl<T: Into<PixLine>> std::ops::AddAssign<T> for RefImage {
    fn add_assign(&mut self, pix_line: T) {
        pix_line.into().iter().for_each(|(p, rgb)| {
            let pixel = self[p];
            self[p] = (
                pixel.0 + rgb.r as i64,
                pixel.1 + rgb.g as i64,
                pixel.2 + rgb.b as i64,
            );
        })
    }
}

impl<T: Into<PixLine>> std::ops::SubAssign<T> for RefImage {
    fn sub_assign(&mut self, pix_line: T) {
        pix_line.into().iter().for_each(|(p, rgb)| {
            let pixel = self[p];
            self[p] = (
                pixel.0 - rgb.r as i64,
                pixel.1 - rgb.g as i64,
                pixel.2 - rgb.b as i64,
            );
        })
    }
}

impl std::ops::Index<&Point> for RefImage {
    type Output = (i64, i64, i64);
    fn index(&self, point: &Point) -> &Self::Output {
        &self.0[point.y as usize][point.x as usize]
    }
}

impl std::ops::Index<(u32, u32)> for RefImage {
    type Output = (i64, i64, i64);
    fn index(&self, (x, y): (u32, u32)) -> &Self::Output {
        &self.0[y as usize][x as usize]
    }
}

impl std::ops::IndexMut<&Point> for RefImage {
    fn index_mut(&mut self, point: &Point) -> &mut Self::Output {
        &mut self.0[point.y as usize][point.x as usize]
    }
}

impl std::ops::IndexMut<(u32, u32)> for RefImage {
    fn index_mut(&mut self, (x, y): (u32, u32)) -> &mut Self::Output {
        &mut self.0[y as usize][x as usize]
    }
}
