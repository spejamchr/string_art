use super::geometry::{Line, Point};
use std::collections::HashMap;

pub struct PixLine(HashMap<Point, u8>);

impl PixLine {
    pub fn iter(&self) -> std::collections::hash_map::Iter<Point, u8> {
        self.0.iter()
    }
}

impl<T: Into<Line>> std::convert::From<(T, f64, f64)> for PixLine {
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

pub struct RefImage(Vec<Vec<i64>>);

impl RefImage {
    pub fn new(width: u32, height: u32) -> Self {
        Self(vec![vec![0; width as usize]; height as usize])
    }

    pub fn score(&self) -> i64 {
        self.0.iter().flatten().map(|p| p * p).sum() // Seems to be worse?
    }

    pub fn subtract_line<T: Into<PixLine>>(&mut self, line: T) -> &mut Self {
        *self -= line;
        self
    }

    pub fn add_line<T: Into<PixLine>>(&mut self, line: T) -> &mut Self {
        *self += line;
        self
    }

    fn width(&self) -> u32 {
        self.0[0].len() as u32
    }

    fn height(&self) -> u32 {
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

impl<T: Into<PixLine>> std::ops::AddAssign<T> for RefImage {
    fn add_assign(&mut self, pix_line: T) {
        pix_line
            .into()
            .iter()
            .for_each(|(p, n)| self[p] += *n as i64);
    }
}

impl<T: Into<PixLine>> std::ops::SubAssign<T> for RefImage {
    fn sub_assign(&mut self, pix_line: T) {
        pix_line
            .into()
            .iter()
            .for_each(|(p, n)| self[p] -= *n as i64);
    }
}

impl std::ops::Index<&Point> for RefImage {
    type Output = i64;
    fn index(&self, point: &Point) -> &Self::Output {
        &self.0[point.y as usize][point.x as usize]
    }
}

impl std::ops::Index<(u32, u32)> for RefImage {
    type Output = i64;
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::string_art::geometry::Vector;

    #[test]
    fn test_pix_line_iter() {
        let pix_line = PixLine::from(((Vector::new(0.0, 0.0), Vector::new(4.0, 1.0)), 0.1, 1.0));
        assert_eq!(6, pix_line.iter().count());
    }
}
