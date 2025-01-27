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
const SQRT_3: f64 = 1.73205080756888;

#[allow(dead_code)]
pub const POINTY: Orientation = Orientation {
    f0: SQRT_3,
    f1: SQRT_3 / 2.0,
    f2: 0.0,
    f3: 3.0 / 2.0,
    b0: SQRT_3 / 3.0,
    b1: -1.0 / 3.0,
    b2: 0.0,
    b3: 2.0 / 3.0,
    start_angle: 0.5,
};

pub const FLAT: Orientation = Orientation {
    f0: 3.0 / 2.0,
    f1: 0.0,
    f2: SQRT_3 / 2.0,
    f3: SQRT_3,
    b0: 2.0 / 3.0,
    b1: 0.0,
    b2: -1.0 / 3.0,
    b3: SQRT_3 / 3.0,
    start_angle: 0.0,
};