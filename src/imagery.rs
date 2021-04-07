use super::geometry::{Line, Point};
use crate::image::DynamicImage;
use crate::image::GenericImageView;
use crate::inout::Data;
use std::collections::HashMap;

/// Monochromatic line of pixels
pub struct PixLineMon(HashMap<Point, u8>);

impl PixLineMon {
    pub fn iter(&self) -> std::collections::hash_map::Iter<Point, u8> {
        self.0.iter()
    }
}

impl<T: Into<Line>> std::convert::From<(T, f64, f64)> for PixLineMon {
    fn from((line, step_size, string_alpha): (T, f64, f64)) -> Self {
        let coloring_val = step_size * string_alpha;
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
                .map(|(p, v)| (p, (f64::min(1.0, v) * (u8::MAX as f64)) as u8))
                .collect::<HashMap<_, _>>(),
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RGB {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn inverted(&self) -> Self {
        Self::new(u8::MAX - self.r, u8::MAX - self.g, u8::MAX - self.b)
    }
}

impl std::fmt::Display for RGB {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "[{:>3}, {:>3}, {:>3}]", self.r, self.g, self.b)
    }
}

impl std::ops::Add<Self> for RGB {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(
            self.r.saturating_add(rhs.r),
            self.g.saturating_add(rhs.g),
            self.b.saturating_add(rhs.b),
        )
    }
}

impl std::ops::Mul<u8> for RGB {
    type Output = Self;
    fn mul(self, rhs: u8) -> Self {
        Self::new(
            self.r.saturating_mul(rhs),
            self.g.saturating_mul(rhs),
            self.b.saturating_mul(rhs),
        )
    }
}

#[derive(Debug, Clone, Copy)]
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
        let r = f64::min(u8::MAX as f64, f64::max(u8::MIN as f64, rgbf.r)) as u8;
        let g = f64::min(u8::MAX as f64, f64::max(u8::MIN as f64, rgbf.g)) as u8;
        let b = f64::min(u8::MAX as f64, f64::max(u8::MIN as f64, rgbf.b)) as u8;
        Self::new(r, g, b)
    }
}

impl<T: Into<u8>> std::convert::From<(T, T, T)> for RGB {
    fn from((r, g, b): (T, T, T)) -> Self {
        RGB::new(r.into(), g.into(), b.into())
    }
}

/// Color line of pixels
pub struct PixLineCol(HashMap<Point, RGB>);

impl PixLineCol {
    pub fn iter(&self) -> std::collections::hash_map::Iter<Point, RGB> {
        self.0.iter()
    }
}

impl<T: Into<Line>> std::convert::From<(T, RGB, f64, f64)> for PixLineCol {
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

/// Monochromatic reference image
pub struct RefImageMon(Vec<Vec<i64>>);

impl RefImageMon {
    pub fn new(width: u32, height: u32) -> Self {
        Self(vec![vec![0; width as usize]; height as usize])
    }

    pub fn invert(&mut self) {
        self.0
            .iter_mut()
            .for_each(|row| row.iter_mut().for_each(|v| *v = (u8::MAX as i64) - *v))
    }

    pub fn score(&self) -> i64 {
        self.0.iter().flatten().map(|p| p * p).sum()
    }

    pub fn subtract_line<T: Into<PixLineMon>>(&mut self, line: T) -> &mut Self {
        *self -= line;
        self
    }

    pub fn add_line<T: Into<PixLineMon>>(&mut self, line: T) -> &mut Self {
        *self += line;
        self
    }

    pub fn width(&self) -> u32 {
        self.0[0].len() as u32
    }

    pub fn height(&self) -> u32 {
        self.0.len() as u32
    }

    pub fn grayscale(&self) -> image::GrayImage {
        let mut img = image::GrayImage::new(self.width(), self.height());
        for (y, row) in self.0.iter().enumerate() {
            for (x, p) in row.iter().enumerate() {
                img.get_pixel_mut(x as u32, y as u32)[0] =
                    i64::max(0, i64::min(u8::MAX as i64, *p)) as u8;
            }
        }
        img
    }
}

impl<T: Into<Line> + Copy> std::convert::From<(&Vec<T>, u32, u32, f64, f64)> for RefImageMon {
    fn from(
        (line_segmentables, width, height, step_size, string_alpha): (&Vec<T>, u32, u32, f64, f64),
    ) -> Self {
        let mut ref_image = Self::new(width, height);
        line_segmentables
            .iter()
            .map(|t| (*t).into())
            .map(|l| (l, step_size, string_alpha))
            .fold(&mut ref_image, |i, a| i.add_line(a));
        ref_image
    }
}

impl std::convert::From<&DynamicImage> for RefImageMon {
    fn from(image: &DynamicImage) -> Self {
        let mut ref_image = Self::new(image.width(), image.height());
        image
            .to_luma8()
            .enumerate_pixels()
            .for_each(|(x, y, p)| ref_image[(x, y)] = p[0].into());
        ref_image
    }
}

impl std::convert::From<&Data> for RefImageMon {
    fn from(data: &Data) -> Self {
        Self::from((
            &data.line_segments,
            data.image_width,
            data.image_height,
            data.args.step_size,
            data.args.string_alpha,
        ))
    }
}

impl<T: Into<PixLineMon>> std::ops::AddAssign<T> for RefImageMon {
    fn add_assign(&mut self, pix_line: T) {
        pix_line
            .into()
            .iter()
            .for_each(|(p, n)| self[p] += *n as i64);
    }
}

impl<T: Into<PixLineMon>> std::ops::SubAssign<T> for RefImageMon {
    fn sub_assign(&mut self, pix_line: T) {
        pix_line
            .into()
            .iter()
            .for_each(|(p, n)| self[p] -= *n as i64);
    }
}

impl std::ops::Index<&Point> for RefImageMon {
    type Output = i64;
    fn index(&self, point: &Point) -> &Self::Output {
        &self.0[point.y as usize][point.x as usize]
    }
}

impl std::ops::Index<(u32, u32)> for RefImageMon {
    type Output = i64;
    fn index(&self, (x, y): (u32, u32)) -> &Self::Output {
        &self.0[y as usize][x as usize]
    }
}

impl std::ops::IndexMut<&Point> for RefImageMon {
    fn index_mut(&mut self, point: &Point) -> &mut Self::Output {
        &mut self.0[point.y as usize][point.x as usize]
    }
}

impl std::ops::IndexMut<(u32, u32)> for RefImageMon {
    fn index_mut(&mut self, (x, y): (u32, u32)) -> &mut Self::Output {
        &mut self.0[y as usize][x as usize]
    }
}

/// Color reference image
#[derive(Debug)]
pub struct RefImageCol(Vec<Vec<(i64, i64, i64)>>);

impl RefImageCol {
    pub fn new(width: u32, height: u32) -> Self {
        Self(vec![vec![(0, 0, 0); width as usize]; height as usize])
    }

    pub fn invert(&mut self) {
        let max = u8::MAX as i64;
        self.0.iter_mut().for_each(|row| {
            row.iter_mut()
                .for_each(|v| *v = (max - v.0, max - v.1, max - v.2))
        })
    }

    pub fn score(&self) -> i64 {
        let m = u8::MAX as i64;
        self.0
            .iter()
            .flatten()
            .flat_map(|(r, g, b)| {
                vec![
                    (m - r).saturating_pow(2),
                    (m - g).saturating_pow(2),
                    (m - b).saturating_pow(2),
                ]
            })
            .sum()
    }

    pub fn subtract_line<T: Into<PixLineCol>>(&mut self, line: T) -> &mut Self {
        *self -= line;
        self
    }

    pub fn add_line<T: Into<PixLineCol>>(&mut self, line: T) -> &mut Self {
        *self += line;
        self
    }

    pub fn width(&self) -> u32 {
        self.0[0].len() as u32
    }

    pub fn height(&self) -> u32 {
        self.0.len() as u32
    }

    pub fn color(&self) -> image::RgbImage {
        let mut img = image::RgbImage::new(self.width(), self.height());
        for (y, row) in self.0.iter().enumerate() {
            for (x, p) in row.iter().enumerate() {
                let pixel = img.get_pixel_mut(x as u32, y as u32);
                pixel[0] = i64_to_u8_clamped(p.0);
                pixel[1] = i64_to_u8_clamped(p.1);
                pixel[2] = i64_to_u8_clamped(p.2);
            }
        }
        img
    }
}

fn i64_to_u8_clamped(num: i64) -> u8 {
    i64::max(u8::MIN as i64, i64::min(u8::MAX as i64, num)) as u8
}

impl<T: Into<PixLineCol> + Copy> std::convert::From<(&Vec<T>, u32, u32)> for RefImageCol {
    fn from((line_segmentables, width, height): (&Vec<T>, u32, u32)) -> Self {
        let mut ref_image = Self::new(width, height);
        line_segmentables
            .iter()
            .fold(&mut ref_image, |i, a| i.add_line(*a));
        ref_image
    }
}

impl std::convert::From<&DynamicImage> for RefImageCol {
    fn from(image: &DynamicImage) -> Self {
        let mut ref_image = Self::new(image.width(), image.height());
        image.to_rgb8().enumerate_pixels().for_each(|(x, y, p)| {
            ref_image[(x, y)] = (p[0].into(), p[1].into(), p[2].into());
        });
        ref_image
    }
}

impl std::convert::From<&Data> for RefImageCol {
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

impl<T: Into<PixLineCol>> std::ops::AddAssign<T> for RefImageCol {
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

impl<T: Into<PixLineCol>> std::ops::SubAssign<T> for RefImageCol {
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

impl std::ops::Index<&Point> for RefImageCol {
    type Output = (i64, i64, i64);
    fn index(&self, point: &Point) -> &Self::Output {
        &self.0[point.y as usize][point.x as usize]
    }
}

impl std::ops::Index<(u32, u32)> for RefImageCol {
    type Output = (i64, i64, i64);
    fn index(&self, (x, y): (u32, u32)) -> &Self::Output {
        &self.0[y as usize][x as usize]
    }
}

impl std::ops::IndexMut<&Point> for RefImageCol {
    fn index_mut(&mut self, point: &Point) -> &mut Self::Output {
        &mut self.0[point.y as usize][point.x as usize]
    }
}

impl std::ops::IndexMut<(u32, u32)> for RefImageCol {
    fn index_mut(&mut self, (x, y): (u32, u32)) -> &mut Self::Output {
        &mut self.0[y as usize][x as usize]
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::geometry::Vector;

    #[test]
    fn test_pix_line_iter() {
        let pix_line =
            PixLineMon::from(((Vector::new(0.0, 0.0), Vector::new(4.0, 1.0)), 0.1, 1.0));
        assert_eq!(6, pix_line.iter().count());
    }
}
