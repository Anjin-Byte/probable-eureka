use std::ops;

use crate::hex::layout::Layout;
use crate::hex::hex::Hex;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    #[allow(dead_code)]
    pub fn from_hex(layout: &Layout, hex: &Hex) -> Self {
        let o = &layout.orientation;

        let x = (o.f0 * hex.q as f64 + o.f1 * hex.r as f64) * layout.size.x;
        let y = (o.f2 * hex.q as f64 + o.f3 * hex.r as f64) * layout.size.y;

        Self {
            x: x + layout.origin.x,
            y: y + layout.origin.y,
        }
    }
}

impl ops::Add for Point {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl ops::Mul<f64> for Point {
    type Output = Point;

    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl ops::Div for Point {
    type Output = Point;

    fn div(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x / rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl ops::Div<f64> for Point {
    type Output = Point;

    fn div(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}