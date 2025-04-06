use std::ops::{Add, Mul, Sub};

use serde::{Deserialize, Serialize};

fn lerp_i32(a: i32, b: i32, t: f64) -> i32 {
    (a as f64 * (1. - t) + b as f64 * t) as i32
}

fn lerp_f32(a: f32, b: f32, t: f64) -> f32 {
    (a as f64 * (1. - t) + b as f64 * t) as f32
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub(crate) struct Point {
    pub(crate) x: i32,
    pub(crate) y: i32,
}

impl Sub for Point {
    type Output = Vector;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Add<Vector> for Point {
    type Output = Point;

    fn add(self, rhs: Vector) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Point {
    pub(crate) fn lerp(a: Self, b: Self, t: f64) -> Self {
        Self {
            x: lerp_i32(a.x, b.x, t),
            y: lerp_i32(a.y, b.y, t),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub(crate) struct Complex {
    pub(crate) r: f32,
    pub(crate) i: f32,
}

impl Complex {
    pub(crate) fn lerp(a: Self, b: Self, t: f64) -> Self {
        Self {
            r: lerp_f32(a.r, b.r, t),
            i: lerp_f32(a.i, b.i, t),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub(crate) struct Vector {
    pub(crate) x: i32,
    pub(crate) y: i32,
}

impl Vector {
    pub(crate) fn len(&self) -> f32 {
        (self.x as f32 * self.x as f32 + self.y as f32 * self.y as f32).sqrt()
    }

    pub(crate) fn normalize(self) -> Complex {
        let len = self.len();
        Complex {
            r: self.x as f32 / len,
            i: self.y as f32 / len,
        }
    }
}

impl Mul<Complex> for Vector {
    type Output = Vector;

    fn mul(self, rhs: Complex) -> Self::Output {
        Self::Output {
            x: (self.x as f32 * rhs.r - self.y as f32 * rhs.i) as i32,
            y: (self.x as f32 * rhs.i + self.y as f32 * rhs.r) as i32,
        }
    }
}
