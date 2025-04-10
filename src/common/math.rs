use serde::{Deserialize, Serialize};
use std::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

fn lerp_f32(a: f32, b: f32, t: f64) -> f32 {
    (a as f64 * (1. - t) + b as f64 * t) as f32
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
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

impl Sub<Vector> for Point {
    type Output = Point;

    fn sub(self, rhs: Vector) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Point {
    pub(crate) fn origin() -> Point {
        Point { x: 0., y: 0. }
    }

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
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub(crate) fn cross(self, rhs: Self) -> f32 {
        (self.x * rhs.y) - (self.y * rhs.x)
    }

    pub(crate) fn dot(self, rhs: Self) -> f32 {
        (self.x * rhs.x) + (self.y * rhs.y)
    }

    pub(crate) fn normalize(self) -> Self {
        let len = self.len();
        Self {
            x: self.x as f32 / len,
            y: self.y as f32 / len,
        }
    }

    pub(crate) fn normalize_into_complex(self) -> Complex {
        let len = self.len();
        Complex {
            r: self.x as f32 / len,
            i: self.y as f32 / len,
        }
    }

    pub(crate) fn left_perpendicular(self) -> Self {
        Self {
            x: -self.y,
            y: self.x,
        }
    }

    pub(crate) fn right_perpendicular(self) -> Self {
        Self {
            x: self.y,
            y: -self.x,
        }
    }

    pub(crate) fn polar(rot: Complex, len: f32) -> Self {
        Self {
            x: rot.r * len,
            y: rot.i * len,
        }
    }

    pub(crate) fn project_on(self, axis: Vector) -> Vector {
        (self.dot(axis) / axis.dot(axis)) * axis
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

impl SubAssign<Vector> for Point {
    fn sub_assign(&mut self, rhs: Vector) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl AddAssign<Vector> for Point {
    fn add_assign(&mut self, rhs: Vector) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Mul<f32> for Vector {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::Output {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Mul<Vector> for f32 {
    type Output = Vector;

    fn mul(self, rhs: Vector) -> Self::Output {
        Self::Output {
            x: self * rhs.x,
            y: self * rhs.y,
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
    pub(crate) segments: (Segment, Segment),
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

    pub(crate) fn intersection_point(&self) -> Point {
        self.segments.0.p0 + (self.segments.0.p1 - self.segments.0.p0) * self.t
    }
}

impl Segment {
    pub(crate) fn vec(self) -> Vector {
        self.p1 - self.p0
    }

    pub(crate) fn project_on(self, axis: Vector) -> Segment {
        Segment {
            p0: Point::origin() + (self.p0 - Point::origin()).project_on(axis),
            p1: Point::origin() + (self.p1 - Point::origin()).project_on(axis),
        }
    }

    /// Returns exit vector if segments intersect. Value is correct only if segments lie on the same line (Not on parallel lines).
    fn exit_vec_while_intersects_on_the_same_line(&self, rhs: &Segment) -> Option<Vector> {
        fn intersects_on_basis(s0: (f32, f32), s1: (f32, f32)) -> Option<f32> {
            let s0 = (s0.0.min(s0.1), s0.0.max(s0.1));
            let s1 = (s1.0.min(s1.1), s1.0.max(s1.1));

            fn abs_min(a: f32, b: f32) -> f32 {
                if a.abs() < b.abs() {
                    a
                } else {
                    b
                }
            }

            if s0.0 <= s1.1 && s1.0 <= s0.1 {
                Some(abs_min(s1.1 - s0.0, s1.0 - s0.1))
            } else {
                None
            }
        }

        if let (Some(x), Some(y)) = (
            intersects_on_basis((self.p0.x, self.p1.x), (rhs.p0.x, rhs.p1.x)),
            intersects_on_basis((self.p0.y, self.p1.y), (rhs.p0.y, rhs.p1.y)),
        ) {
            Some(Vector { x, y })
        } else {
            None
        }
    }

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

        Some(RayCastScalars {
            t,
            u,
            segments: (self, rhs),
        })
    }
}

pub(crate) trait Collide<Rhs: ?Sized = Self> {
    fn collide(&self, rhs: &Rhs) -> Option<Vector>;
}

impl<const C0: usize, const C1: usize> Collide<[Segment; C1]> for [Segment; C0] {
    fn collide(&self, rhs: &[Segment; C1]) -> Option<Vector> {
        let axes: Vec<_> = self
            .iter()
            .chain(rhs.iter())
            .map(|x| x.vec().left_perpendicular().normalize())
            .collect();

        let mut exit_vectors = Vec::with_capacity(axes.len());

        for axis in axes {
            let proj = |edges: &[Segment]| -> Segment {
                fn min_point(p0: Point, p1: Point) -> Point {
                    Point {
                        x: p0.x.min(p1.x),
                        y: p0.y.min(p1.y),
                    }
                }

                fn max_point(p0: Point, p1: Point) -> Point {
                    Point {
                        x: p0.x.max(p1.x),
                        y: p0.y.max(p1.y),
                    }
                }

                edges
                    .into_iter()
                    .map(|edge| {
                        let proj = edge.project_on(axis);

                        [proj.p0, proj.p1]
                    })
                    .flatten()
                    .fold(None, |m: Option<Segment>, x| {
                        m.map_or(Some(Segment { p0: x, p1: x }), |Segment { p0, p1 }| {
                            Some(Segment {
                                p0: min_point(p0, x),
                                p1: max_point(p1, x),
                            })
                        })
                    })
                    .unwrap()
            };

            let proj0 = proj(self);
            let proj1 = proj(rhs);

            if let Some(exit_vec) = proj0.exit_vec_while_intersects_on_the_same_line(&proj1) {
                exit_vectors.push(exit_vec);
            } else {
                return None;
            }
        }

        exit_vectors
            .into_iter()
            .min_by(|a, b| a.len().partial_cmp(&b.len()).unwrap())
    }
}

// impl Collide<[Segment]> for [Segment] {
//     fn collide(&self, rhs: &[Segment]) -> Option<Vector> {
//         let axes: Vec<_> = self
//             .iter()
//             .chain(rhs.iter())
//             .map(|x| x.vec().left_perpendicular().normalize())
//             .collect();

//         let mut exit_vectors = Vec::with_capacity(axes.len());

//         for axis in axes {
//             let proj = |edges: &[Segment]| -> Segment {
//                 fn min_point(p0: Point, p1: Point) -> Point {
//                     Point {
//                         x: p0.x.min(p1.x),
//                         y: p0.y.min(p1.y),
//                     }
//                 }

//                 fn max_point(p0: Point, p1: Point) -> Point {
//                     Point {
//                         x: p0.x.max(p1.x),
//                         y: p0.y.max(p1.y),
//                     }
//                 }

//                 edges
//                     .into_iter()
//                     .map(|edge| {
//                         let proj = edge.project_on(axis);

//                         [proj.p0, proj.p1]
//                     })
//                     .flatten()
//                     .fold(None, |m: Option<Segment>, x| {
//                         m.map_or(Some(Segment { p0: x, p1: x }), |Segment { p0, p1 }| {
//                             Some(Segment {
//                                 p0: min_point(p0, x),
//                                 p1: max_point(p1, x),
//                             })
//                         })
//                     })
//                     .unwrap()
//             };

//             let proj0 = proj(self);
//             let proj1 = proj(rhs);

//             if let Some(exit_vec) = proj0.exit_vec_while_intersects_on_the_same_line(&proj1) {
//                 exit_vectors.push(exit_vec);
//             } else {
//                 return None;
//             }
//         }

//         exit_vectors
//             .into_iter()
//             .min_by(|a, b| a.len().partial_cmp(&b.len()).unwrap())
//     }
// }

pub(crate) trait Segments<const C: usize> {
    fn segments_ringe(self) -> [Segment; C];
}

impl<const C: usize> Segments<C> for [Point; C] {
    fn segments_ringe(self) -> [Segment; C] {
        let mut i: usize = 0;
        self.map(|p0| {
            let r = Segment {
                p0,
                p1: self[(i + 1) % self.len()],
            };
            i += 1;
            r
        })
    }
}

pub(crate) trait DynSizeSegments {
    fn segments_ringe(&self) -> impl Iterator<Item = Segment>;

    fn segments(&self) -> impl Iterator<Item = Segment>;
}

impl DynSizeSegments for [Point] {
    fn segments_ringe(&self) -> impl Iterator<Item = Segment> {
        (0..self.len()).map(|i| Segment {
            p0: self[i],
            p1: self[(i + 1) % self.len()],
        })
    }

    fn segments(&self) -> impl Iterator<Item = Segment> {
        (1..self.len()).map(|i| Segment {
            p0: self[i - 1],
            p1: self[i],
        })
    }
}
