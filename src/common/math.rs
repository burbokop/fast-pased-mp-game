use std::ops::{Add, Mul, Neg, Sub};

use serde::{Deserialize, Serialize};

fn lerp_f32(a: f32, b: f32, t: f64) -> f32 {
    (a as f64 * (1. - t) + b as f64 * t) as f32
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub(crate) struct Point {
    pub(crate) x: f32,
    pub(crate) y: f32,
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
            x: lerp_f32(a.x, b.x, t),
            y: lerp_f32(a.y, b.y, t),
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
    pub(crate) fn inverted_r(self) -> Self {
        Self {
            r: -self.r,
            i: self.i,
        }
    }

    pub(crate) fn inverted_i(self) -> Self {
        Self {
            r: self.r,
            i: -self.i,
        }
    }
}

impl Neg for Complex {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::Output {
            r: -self.r,
            i: -self.i,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub(crate) struct Vector {
    pub(crate) x: f32,
    pub(crate) y: f32,
}

impl Vector {
    pub(crate) fn len(&self) -> f32 {
        (self.x as f32 * self.x as f32 + self.y as f32 * self.y as f32).sqrt()
    }

    pub(crate) fn cross(self, rhs: Self) -> f32 {
        (self.x * rhs.y) - (self.y * rhs.x)
    }

    pub(crate) fn dot(self, rhs: Self) -> f32 {
        (self.x * rhs.x) + (self.y * rhs.y)
    }

    pub(crate) fn normalize(self) -> Complex {
        let len = self.len();
        Complex {
            r: self.x as f32 / len,
            i: self.y as f32 / len,
        }
    }

    pub(crate) fn polar(rot: Complex, len: f32) -> Self {
        Self {
            x: rot.r * len,
            y: rot.i * len,
        }
    }
}

impl Mul<Complex> for Vector {
    type Output = Self;

    fn mul(self, rhs: Complex) -> Self::Output {
        Self::Output {
            x: self.x as f32 * rhs.r - self.y as f32 * rhs.i,
            y: self.x as f32 * rhs.i + self.y as f32 * rhs.r,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub(crate) struct Rect {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) w: f32,
    pub(crate) h: f32,
}

impl Rect {
    pub(crate) fn points(self) -> [Point; 4] {
        [
            Point {
                x: self.x,
                y: self.y,
            },
            Point {
                x: self.x + self.w,
                y: self.y,
            },
            Point {
                x: self.x + self.w,
                y: self.y + self.h,
            },
            Point {
                x: self.x,
                y: self.y + self.h,
            },
        ]
    }

    pub(crate) fn edges(self) -> [Segment; 4] {
        let p = self.points();
        [
            Segment { p0: p[0], p1: p[1] },
            Segment { p0: p[1], p1: p[2] },
            Segment { p0: p[2], p1: p[3] },
            Segment { p0: p[3], p1: p[0] },
        ]
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub(crate) struct Segment {
    pub(crate) p0: Point,
    pub(crate) p1: Point,
}

pub(crate) struct RayCastScalars {
    pub(crate) t: f32,
    pub(crate) u: f32,
}

impl RayCastScalars {
    pub(crate) fn intersects(&self) -> bool {
        self.t > 0. && self.t < 1. && self.u > 0. && self.u < 1.
    }
    pub(crate) fn intersects_including(&self) -> bool {
        self.t >= 0. && self.t <= 1. && self.u >= 0. && self.u <= 1.
    }
}

impl Segment {
    pub(crate) fn inverted(self) -> Self {
        Self {
            p0: self.p1,
            p1: self.p0,
        }
    }

    pub(crate) fn inverted_x(self) -> Self {
        Self {
            p0: Point {
                x: self.p1.x,
                y: self.p0.y,
            },
            p1: Point {
                x: self.p0.x,
                y: self.p1.y,
            },
        }
    }

    pub(crate) fn inverted_y(self) -> Self {
        Self {
            p0: Point {
                x: self.p0.x,
                y: self.p1.y,
            },
            p1: Point {
                x: self.p1.x,
                y: self.p0.y,
            },
        }
    }

    pub(crate) fn stretch_with_fixed_center(self, t: f32) -> Self {
        Self {
            p0: Point {
                x: (self.p1.x + self.p0.x) / 2. - t * (self.p1.x - self.p0.x) / 2.,
                y: (self.p1.y + self.p0.y) / 2. - t * (self.p1.y - self.p0.y) / 2.,
            },
            p1: Point {
                x: (self.p1.x + self.p0.x) / 2. + t * (self.p1.x - self.p0.x) / 2.,
                y: (self.p1.y + self.p0.y) / 2. + t * (self.p1.y - self.p0.y) / 2.,
            },
        }
    }

    pub(crate) fn stretch_with_fixed_first_point_x(self, x: f32) -> Self {
        Self {
            p0: Point {
                x: self.p0.x,
                y: self.p0.y,
            },
            p1: Point {
                x: self.p0.x + x * (self.p1.x - self.p0.x),
                y: self.p1.y,
            },
        }
    }

    pub(crate) fn stretch_with_fixed_first_point_y(self, y: f32) -> Self {
        Self {
            p0: Point {
                x: self.p0.x,
                y: self.p0.y,
            },
            p1: Point {
                x: self.p1.x,
                y: self.p0.y + y * (self.p1.y - self.p0.y),
            },
        }
    }

    pub(crate) fn ray_cast(self, rhs: Self) -> Option<RayCastScalars> {
        let a = self.p0;
        let b = self.p1;

        let c = rhs.p0;
        let d = rhs.p1;

        let ac = c - a;
        let cd = d - c;
        let ab = b - a;

        let t = Vector::cross(ac, cd) / Vector::cross(ab, cd);
        let u = Vector::cross(ac, ab) / Vector::cross(ab, cd);

        Some(RayCastScalars { t, u })
    }
}
