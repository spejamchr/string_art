use crate::serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vector {
    x: f64,
    y: f64,
}

impl Vector {
    fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    fn len(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    fn basis(&self) -> Self {
        *self / self.len()
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
        let step = (self.1 - self.0).basis() * step_size;
        let current = self.0;
        let distance = (self.1 - self.0).len();

        LineIter {
            step,
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

impl<T: Into<Vector>, A> std::convert::From<(T, T, A)> for Line {
    fn from((a, b, _): (T, T, A)) -> Self {
        Self(a.into(), b.into())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LineIter {
    step: Vector,
    current: Vector,
    distance: f64,
    step_size: f64,
}

impl Iterator for LineIter {
    type Item = Vector;
    fn next(&mut self) -> std::option::Option<<Self as std::iter::Iterator>::Item> {
        if self.distance >= 0.0 {
            let current = self.current;
            self.current = self.current + self.step;
            self.distance -= self.step_size;
            Some(current)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
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

    fn v(a: f64, b: f64) -> Vector {
        Vector::new(a, b)
    }

    fn origin() -> Vector {
        v(0.0, 0.0)
    }

    fn a() -> Vector {
        v(3.0, 4.0)
    }

    fn b() -> Vector {
        v(6.0, 0.0)
    }

    #[test]
    fn test_line_iter() {
        let line = Line(origin(), a());
        let iter = line.iter(1.0);
        assert_eq!(6, iter.count());
    }

    #[test]
    fn test_line_iter_detail() {
        let line = Line(origin(), v(0.0, 10.0));
        let vectors: Vec<Vector> = line.iter(10.0).collect();
        assert_eq!(vec![v(0.0, 0.0), v(0.0, 10.0)], vectors);

        let line = Line(origin(), v(0.0, 10.0));
        let vectors: Vec<Vector> = line.iter(2.0).collect();
        assert_eq!(
            vec![
                v(0.0, 0.0),
                v(0.0, 2.0),
                v(0.0, 4.0),
                v(0.0, 6.0),
                v(0.0, 8.0),
                v(0.0, 10.0)
            ],
            vectors
        );
    }

    #[test]
    fn test_vector_len() {
        assert_eq!(5.0, a().len());
        assert_eq!(6.0, b().len());
    }

    #[test]
    fn test_vector_basis() {
        assert_eq!(v(1.0, 0.0), b().basis());
    }

    #[test]
    fn test_vector_add() {
        assert_eq!(v(9.0, 4.0), a() + b());
    }
    #[test]
    fn test_vector_sub() {
        assert_eq!(v(-3.0, 4.0), a() - b());
    }
    #[test]
    fn test_vector_mul() {
        assert_eq!(v(6.0, 8.0), a() * 2.0);
    }

    #[test]
    fn test_vector_div() {
        assert_eq!(v(2.0, 0.0), b() / 3.0);
    }

    #[test]
    fn test_vector_from_point() {
        assert_eq!(v(2.0, 3.0), Vector::from(Point::new(2, 3)));
    }
}
