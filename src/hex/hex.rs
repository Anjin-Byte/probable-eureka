use crate::hex::layout::{Layout, Direction};
use crate::hex::point::Point;

use std::borrow::Borrow;
use std::hash::{Hash, Hasher};
use std::cmp::Ordering;
use std::ops;


#[derive(Hash, Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct Hex {
    pub q: i32,
    pub r: i32,
    pub s: i32,
}

impl Hex {
    pub fn new(q: i32, r: i32) -> Self {
        let s = -q - r;
        Self { q, r, s }
    }

    pub fn from_point(layout: &Layout, point: &Point) -> FractionalHex {
        let o = &layout.orientation;

        let p = Point {
            x: (point.x - layout.origin.x) / layout.size.x,
            y: (point.y - layout.origin.y) / layout.size.y,
        };

        let q = o.b0 * p.x + o.b1 * p.y;
        let r = o.b2 * p.x + o.b3 * p.y;

        /*
        let s = -q - r;
        let a = (q - r).ceil();
        let b = (r - s).ceil();
        let c = (s - q).ceil();

        let q = ((a - c) / 3.0).round() as i32;
        let r = ((b - a) / 3.0).round() as i32; 
        */

        FractionalHex::new(q, r)
    }

    #[allow(dead_code)]
    pub fn neighbor(&self, direction: Direction) -> Self {
        *self + Self::from(direction)
    }
}

impl From<Point> for Hex {
    fn from(p: Point) -> Self {
        const SQRT_3: f64 = 1.73205080756888;

        //let x = p.x * 1_f64/SQRT_3;
        //let y = p.y * -1_f64/SQRT_3;
        let x = p.x;
        let y = p.y;

        let t = SQRT_3 * y + 1_f64;
        let temp1 = (t + x).floor();
        let temp2 = t - x;
        let temp3 = 2_f64 * x + 1_f64;

        let q = ((temp1 + temp3) / 3.0).floor() as i32;
        let r = ((temp1 + temp2) / 3.0).floor() as i32;

        Self::new(q, r)       
    }
}

impl From<FractionalHex> for Hex {
    fn from(h: FractionalHex) -> Self {
        let q = h.q.round();
        let r = h.r.round();
        let s = h.s.round();

        let q_diff = (q - h.q).abs();
        let r_diff = (r - h.r).abs();
        let s_diff = (s - h.s).abs();

        if q_diff > r_diff && q_diff > s_diff {
            Self::new((-r -s) as i32, r as i32)
        } else if r_diff > s_diff {
            Self::new(q as i32, (-q - s) as i32)
        } else {
            Self::new(q as i32, r as i32)
        }
    }
}

impl From<Direction> for Hex {
    fn from(direction: Direction) -> Self {
        match direction {
            Direction::North => Self::new(1, 0),
            Direction::NorthEast => Self::new(1, -1),
            Direction::SouthEast => Self::new(0, -1),
            Direction::South => Self::new(-1, 0),
            Direction::SouthWest => Self::new(-1, 1),
            Direction::NorthWest => Self::new(0, 1),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct FractionalHex {
    pub q: f64,
    pub r: f64,
    pub s: f64,
}

impl FractionalHex {
    pub fn new(q: f64, r: f64) -> Self {
        let s = -q - r;
        Self { q, r, s }
    }

    pub fn round(&self) -> Hex {
        let mut qi = self.q.round() as i32;
        let mut ri = self.r.round() as i32;
        let mut si = self.s.round() as i32;

        let q_diff = (qi as f64 - self.q).abs();
        let r_diff = (ri as f64 - self.r).abs();
        let s_diff = (si as f64 - self.s).abs();

        if q_diff > r_diff && q_diff > s_diff {
            qi = -ri - si;
        } else if r_diff > s_diff {
            ri = -qi - si;
        } else {
            si = -qi - ri;
        }

        Hex { q: qi, r: ri, s: si }
    }
}

impl Eq for FractionalHex {}

impl Ord for FractionalHex {
    fn cmp(&self, other: &Self) -> Ordering {
        self.round().cmp(&other.round())
    }
}

impl Hash for FractionalHex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.round().hash(state)
    }
}

impl ops::Add for Hex {
    type Output = Hex;

    fn add(self, rhs: Self) -> Self::Output {
        Hex::new(self.q + rhs.q, self.r + rhs.r)
    }
}

impl ops::Sub for Hex {
    type Output = Hex;

    fn sub(self, rhs: Self) -> Self::Output {
        Hex::new(self.q - rhs.q, self.r - rhs.r)
    }
}

impl ops::Mul for Hex {
    type Output = Hex;

    fn mul(self, rhs: Self) -> Self::Output {
        Hex::new(self.q * rhs.q, self.r * rhs.r)
    }
}

impl ops::Mul<i32> for Hex {
    type Output = Hex;

    fn mul(self, rhs: i32) -> Self::Output {
        Hex::new(self.q * rhs, self.r * rhs)
    }
}

impl ops::Div for Hex {
    type Output = FractionalHex;

    fn div(self, rhs: Self) -> Self::Output {
        Self::Output::new((self.q / rhs.q) as f64, (self.r / rhs.r) as f64)
    }
}

impl ops::Div<f64> for Hex {
    type Output = FractionalHex;

    fn div(self, rhs: f64) -> Self::Output {
        Self::Output::new(self.q as f64 / rhs, self.r as f64 / rhs)
    }
}

impl ops::Rem for Hex {
    type Output = FractionalHex;

    fn rem(self, rhs: Self) -> Self::Output {
        Self::Output::new((self.q % rhs.q) as f64, (self.r % rhs.r) as f64)
    }
}







// Implement Borrow<FractionalHex> for Hex
impl Borrow<FractionalHex> for Hex {
    fn borrow(&self) -> &FractionalHex {
        // Unsafe block is required to create a reference from a Hex to a FractionalHex
        // This assumes that FractionalHex has a similar layout to Hex
        // If they don't, this code will cause undefined behavior.
        // You should provide a better implementation if they differ in memory layout.
        unsafe { &*(self as *const Hex as *const FractionalHex) }
    }
}