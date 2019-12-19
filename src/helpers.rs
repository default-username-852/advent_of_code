use std::fmt::{Formatter, Display, Error};
use std::ops::{Sub, Add, Div, Mul};

#[derive(Clone, Debug, Hash, Copy, Eq, PartialEq)]
pub struct Point {
    x: i64,
    y: i64,
}

impl Point {
    pub fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }

    pub fn euclidian(&self, other: &Point) -> f64 {
        (((self.x - other.x) as f64).powi(2) + ((self.y - other.y) as f64).powi(2)).sqrt()
    }

    pub fn manhattan(&self, other: &Point) -> i64 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }

    pub fn neighbours(&self) -> Vec<Point> {
        let mut out = Vec::new();

        out.push(*self + Point::from((0, 1)));
        out.push(*self + Point::from((0, -1)));
        out.push(*self + Point::from((1, 0)));
        out.push(*self + Point::from((-1, 0)));

        out
    }
}

impl From<(i64, i64)> for Point {
    fn from(a: (i64, i64)) -> Self {
        Self { x: a.0, y: a.1 }
    }
}

impl Mul<i64> for Point {
    type Output = Point;

    fn mul(self, rhs: i64) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Mul<Point> for i64 {
    type Output = Point;

    fn mul(self, rhs: Point) -> Self::Output {
        Point {
            x: self * rhs.x,
            y: self * rhs.y,
        }
    }
}

impl Div<i64> for Point {
    type Output = Point;

    fn div(self, rhs: i64) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl Div<Point> for i64 {
    type Output = Point;

    fn div(self, rhs: Point) -> Self::Output {
        Point {
            x: rhs.x / self,
            y: rhs.y / self,
        }
    }
}

impl Add<i64> for Point {
    type Output = Point;

    fn add(self, rhs: i64) -> Self::Output {
        Self {
            x: self.x + rhs,
            y: self.y + rhs,
        }
    }
}

impl Add<(i64, i64)> for Point {
    type Output = Point;

    fn add(mut self, rhs: (i64, i64)) -> Self::Output {
        self.x += rhs.0;
        self.y += rhs.1;
        self
    }
}

impl Add for Point {
    type Output = Point;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Point {
    type Output = Point;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn rotate_left(&self) -> Self {
        match self {
            Direction::Left => Direction::Down,
            Direction::Down => Direction::Right,
            Direction::Right => Direction::Up,
            Direction::Up => Direction::Left,
        }
    }

    pub fn rotate_right(&self) -> Self {
        match self {
            Direction::Left => Direction::Up,
            Direction::Up => Direction::Right,
            Direction::Right => Direction::Down,
            Direction::Down => Direction::Left,
        }
    }

    pub fn offset(&self) -> (i64, i64) {
        match self {
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
        }
    }
}