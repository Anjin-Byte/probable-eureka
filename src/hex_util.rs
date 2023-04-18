use std::cmp::{PartialEq, PartialOrd};
use std::convert::From;
use std::ops::{Add, Div, Mul, Rem, Sub};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn from_hex(layout: &Layout, hex: &Hex) -> Self {
        let o = &layout.orientation;

        let x = (o.f0 * hex.q + o.f1 * hex.r) * layout.size.x;
        let y = (o.f2 * hex.q + o.f3 * hex.r) * layout.size.y;

        Self {
            x: x + layout.origin.x,
            y: y + layout.origin.y,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Orientation {
    pub f0: f64,
    pub f1: f64,
    pub f2: f64,
    pub f3: f64,
    pub b0: f64,
    pub b1: f64,
    pub b2: f64,
    pub b3: f64,
    pub start_angle: f64,
}

pub struct Layout {
    orientation: Orientation,
    size: Point,
    origin: Point,
}

impl Layout {
    pub fn new(size: Point, origin: Point) -> Self {

// layout_pointy = Orientation(math.sqrt(3.0), math.sqrt(3.0) / 2.0, 0.0, 3.0 / 2.0, math.sqrt(3.0) / 3.0, -1.0 / 3.0, 0.0, 2.0 / 3.0, 0.5)
// layout_flat = Orientation(3.0 / 2.0, 0.0, math.sqrt(3.0) / 2.0, math.sqrt(3.0), 2.0 / 3.0, 0.0, -1.0 / 3.0, math.sqrt(3.0) / 3.0, 0.0)

        let pointy: Orientation = Orientation {
            f0: (3.0_f64.sqrt()),
            f1: (3.0_f64.sqrt() / 2.0),
            f2: (0.0),
            f3: (3.0 / 2.0),
            b0: (3.0_f64.sqrt() / 3.0),
            b1: (-1.0 / 3.0),
            b2: (0.0),
            b3: (2.0 / 3.0),
            start_angle: (0.5),
        };

        let flat: Orientation = Orientation {
            f0: (3.0 / 2.0),
            f1: (0.0),
            f2: (3.0_f64.sqrt() / 2.0),
            f3: (3.0_f64.sqrt()),
            b0: (2.0 / 3.0),
            b1: (0.0),
            b2: (-1.0 / 3.0),
            b3: (3.0_f64.sqrt() / 3.0),
            start_angle: (0.0),
        };

        Self {
            orientation: pointy,
            size,
            origin,
        }
    }

    fn hex_corner_offset(&self, i: u8) -> Point {
        let angle = 2.0 * std::f64::consts::PI * (self.orientation.start_angle - i as f64) / 6.0;
        Point {
            x: self.size.x * angle.cos(),
            y: self.size.y * angle.sin(),
        }
    }

    pub fn polygon_corners(&self, hex: &Hex) -> Vec<Point> {
        let mut corners: Vec<Point> = Vec::new();
        let center = Point::from_hex(&self, &hex);

        for i in 0..6 {
            let offset = self.hex_corner_offset(i);
            corners.push(Point {
                x: center.x + offset.x,
                y: center.y + offset.y,
            })
        }

        corners
    }
}

pub enum Direction {
    North,
    NorthEast,
    SouthEast,
    South,
    SouthWest,
    NorthWest,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Hex {
    pub q: f64,
    pub r: f64,
    pub s: f64,
}

impl Hex {
    pub fn new(q: f64, r: f64) -> Self {
        let s = -q - r;
        Self { q, r, s }
    }

    pub fn from_point(layout: &Layout, point: &Point) -> Self {
        let o = &layout.orientation;

        let p = Point {
            x: (point.x - layout.origin.x) / layout.size.x,
            y: (point.y - layout.origin.y) / layout.size.y,
        };

        let q = o.b0 * p.x + o.b1 * p.y;
        let r = o.b2 * p.x + o.b3 * p.y;

        Self::new(q, r)
    }

    pub fn neighbor(&self, direction: Direction) -> Self {
        *self + Hex::from(direction)
    }

    pub fn round(&self) -> Self {
        let q = self.q.round();
        let r = self.r.round();
        let s = self.s.round();

        let q_diff = (q - self.q).abs();
        let r_diff = (r - self.r).abs();
        let s_diff = (s - self.s).abs();

        if q_diff > r_diff && q_diff > s_diff {
            Self::new(-r - s, r)
        } else if r_diff > s_diff {
            Self::new(q, -q - s)
        } else {
            Self::new(q, r)
        }
    }

    pub fn rotate(&self, counter: bool) -> Self {
        match counter {
            true => Self::new(-&self.s, -&self.q),
            false => Self::new(-&self.r, -&self.s)
        }
    }
}

impl From<Direction> for Hex {
    fn from(direction: Direction) -> Self {
        match direction {
            Direction::North => Hex::new(1.0, 0.0),
            Direction::NorthEast => Hex::new(1.0, -1.0),
            Direction::SouthEast => Hex::new(0.0, -1.0),
            Direction::South => Hex::new(-1.0, 0.0),
            Direction::SouthWest => Hex::new(-1.0, 1.0),
            Direction::NorthWest => Hex::new(0.0, 1.0),
        }
    }
}

impl Add for Hex {
    type Output = Hex;

    fn add(self, rhs: Self) -> Self::Output {
        Hex::new(self.q + rhs.q, self.r + rhs.r)
    }
}

impl Sub for Hex {
    type Output = Hex;

    fn sub(self, rhs: Self) -> Self::Output {
        Hex::new(self.q - rhs.q, self.r - rhs.r)
    }
}

impl Mul for Hex {
    type Output = Hex;

    fn mul(self, rhs: Self) -> Self::Output {
        Hex::new(self.q * rhs.q, self.r * rhs.r)
    }
}

impl Mul<f64> for Hex {
    type Output = Hex;

    fn mul(self, rhs: f64) -> Self::Output {
        Hex::new(self.q * rhs, self.r * rhs)
    }
}

impl Div for Hex {
    type Output = Hex;

    fn div(self, rhs: Self) -> Self::Output {
        Hex::new(self.q / rhs.q, self.r / rhs.r)
    }
}

impl Div<f64> for Hex {
    type Output = Hex;

    fn div(self, rhs: f64) -> Self::Output {
        Hex::new(self.q / rhs, self.r / rhs)
    }
}

impl Rem for Hex {
    type Output = Hex;

    fn rem(self, rhs: Self) -> Self::Output {
        Hex::new(self.q % rhs.q, self.r % rhs.r)
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    fn assert_f64_eq(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-6, "{} != {}", a, b);
    }

    #[test]
    fn test_hex_rounding() {
        let hex = Hex::new(1.5, -0.5);
        let rounded = hex.round();
        assert_eq!(rounded, Hex::new(2.0, -1.0));
    }

    #[test]
    fn test_hex_rotation() {
        let hex = Hex::new(1.0, -1.0);

        let rotated_counter = hex.rotate(true);
        assert_eq!(rotated_counter, Hex::new(0.0, -1.0));

        let rotated_clockwise = hex.rotate(false);
        assert_eq!(rotated_clockwise, Hex::new(1.0, 0.0));
    }

    #[test]
    fn test_layout_creation() {
        let size = Point { x: 10.0, y: 10.0 };
        let origin = Point { x: 5.0, y: 5.0 };

        let layout = Layout::new(size, origin);
        assert_eq!(layout.size, size);
        assert_eq!(layout.origin, origin);
    }

    #[test]
    fn test_point_from_hex_conversion() {
        let layout = Layout::new(Point { x: 10.0, y: 10.0 }, Point { x: 0.0, y: 0.0 });
        let hex = Hex::new(1.0, -1.0);
        let point = Point::from_hex(&layout, &hex);

        assert_f64_eq(point.x, 15.0);
        assert_f64_eq(point.y, 5.0);
    }

    #[test]
    fn test_hex_from_point_conversion() {
        let layout = Layout::new(Point { x: 10.0, y: 10.0 }, Point { x: 0.0, y: 0.0 });
        let point = Point { x: 15.0, y: 5.0 };
        let hex = Hex::from_point(&layout, &point);

        assert_f64_eq(hex.q, 1.0);
        assert_f64_eq(hex.r, -1.0);
    }

    #[test]
    fn test_hex_corner_offset() {
        let layout = Layout::new(Point { x: 10.0, y: 10.0 }, Point { x: 0.0, y: 0.0 });

        let corner_offset = layout.hex_corner_offset(0);
        assert_f64_eq(corner_offset.x, 5.0);
        assert_f64_eq(corner_offset.y, 0.0);

        let corner_offset = layout.hex_corner_offset(2);
        assert_f64_eq(corner_offset.x, -5.0);
        assert_f64_eq(corner_offset.y, 0.0);
    }

    #[test]
    fn test_polygon_corners() {
        let layout = Layout::new(Point { x: 10.0, y: 10.0 }, Point { x: 0.0, y: 0.0 });
        let hex = Hex::new(1.0, -1.0);
        let corners = layout.polygon_corners(&hex);

        assert_eq!(corners.len(), 6);
        let expected_corners = vec![
            Point { x: 20.0, y: 5.0 },
            Point { x: 15.0, y: 10.0 },
            Point { x: 5.0, y: 10.0 },
            Point { x: 0.0, y: 5.0 },
            Point { x: 5.0, y: 0.0 },
            Point { x: 15.0, y: 0.0 },
        ];

        for i in 0..6 {
            assert_f64_eq(corners[i].x, expected_corners[i].x);
            assert_f64_eq(corners[i].y, expected_corners[i].y);
        }
    }

    #[test]
    fn test_hex_operations_with_scalar() {
        let hex = Hex::new(2.0, -1.0);

        let multiplied = hex * 2.0;
        assert_eq!(multiplied, Hex::new(4.0, -2.0));

        let divided = hex / 2.0;
        assert_eq!(divided, Hex::new(1.0, -0.5));
    }

    #[test]
    fn test_hex_mul_div_rem_ops() {
        let hex1 = Hex::new(2.0, -1.0);
        let hex2 = Hex::new(3.0, -2.0);

        let multiplied = hex1 * hex2;
        assert_eq!(multiplied, Hex::new(6.0, 2.0));

        let divided = hex1 / hex2;
        assert_eq!(divided, Hex::new(2.0 / 3.0, -1.0 / -2.0));

        let remainder = hex1 % hex2;
        assert_eq!(remainder, Hex::new(2.0 % 3.0, -1.0 % -2.0));
    }
}
