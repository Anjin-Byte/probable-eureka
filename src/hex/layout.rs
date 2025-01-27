use crate::hex::orientation::{Orientation, FLAT};
use crate::hex::point::Point;
use crate::hex::hex::Hex;

#[allow(dead_code)]
pub enum Direction {
    North,
    NorthEast,
    SouthEast,
    South,
    SouthWest,
    NorthWest,
}

#[derive(Debug)]
pub struct Layout {
    pub orientation: Orientation,
    pub size: Point,
    pub origin: Point,
}

impl Layout {
    pub fn new(size: Point, origin: Point) -> Self {
        Self {
            orientation: FLAT,
            size,
            origin,
        }
    }

    #[allow(dead_code)]
    fn hex_corner_offset(&self, i: u8) -> Point {
        let angle = 2.0 * std::f64::consts::PI * (self.orientation.start_angle - i as f64) / 6.0;
        Point {
            x: self.size.x * angle.cos(),
            y: self.size.y * angle.sin(),
        }
    }

    #[allow(dead_code)]
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