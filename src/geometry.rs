#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vector {
    x: f64,
    y: f64,
}

impl Vector {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn len(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn basis(&self) -> Self {
        *self / self.len()
    }

    pub fn dist(&self, other: &Self) -> f64 {
        (*other - *self).len()
    }
}

impl std::ops::Add for Vector {
    type Output = Self;
    fn add(self, rhs: Self) -> <Self as std::ops::Add>::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl std::ops::Sub for Vector {
    type Output = Self;
    fn sub(self, rhs: Self) -> <Self as std::ops::Add>::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl std::ops::Mul<f64> for Vector {
    type Output = Self;
    fn mul(self, num: f64) -> <Self as std::ops::Mul<f64>>::Output {
        Self::new(self.x * num, self.y * num)
    }
}

impl std::ops::Div<f64> for Vector {
    type Output = Self;
    fn div(self, num: f64) -> <Self as std::ops::Div<f64>>::Output {
        Self::new(self.x / num, self.y / num)
    }
}

impl std::convert::From<Point> for Vector {
    fn from(point: Point) -> Self {
        Self::new(point.x as f64, point.y as f64)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Line(Vector, Vector);

impl Line {
    pub fn iter(&self, step_size: f64) -> LineIter {
        let basis = (self.1 - self.0).basis();
        let current = self.0;
        let distance = self.0.dist(&self.1);

        LineIter {
            basis,
            current,
            distance,
            step_size,
        }
    }
}

impl<T: Into<Vector>> std::convert::From<(T, T)> for Line {
    fn from((a, b): (T, T)) -> Self {
        Self(a.into(), b.into())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LineIter {
    basis: Vector,
    current: Vector,
    distance: f64,
    step_size: f64,
}

impl Iterator for LineIter {
    type Item = Vector;
    fn next(&mut self) -> std::option::Option<<Self as std::iter::Iterator>::Item> {
        if self.distance >= 0.0 {
            let current = self.current;
            self.current = self.current + self.basis * self.step_size;
            self.distance -= self.step_size;
            Some(current)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

impl Point {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "({:>6}, {:>6})", self.x, self.y)
    }
}

impl std::convert::From<Vector> for Point {
    fn from(vector: Vector) -> Self {
        Self::new(vector.x.round() as u32, vector.y.round() as u32)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_line_iter() {
        let line = Line(Vector::new(0.0, 0.0), Vector::new(3.0, 4.0));
        let iter = line.iter(1.0);
        assert_eq!(6, iter.count());
    }
}
