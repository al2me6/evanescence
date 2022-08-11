use std::fmt::{self, Display};
use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub};

use getset::CopyGetters;

use super::Linspace;
use crate::geometry::SphericalPoint3;

/// A vector (the mathematical kind) in `R^3`.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, CopyGetters)]
pub struct Vec3 {
    // The `with_prefix` attribute doesn't appear to work on the entire struct.
    #[getset(get_copy = "pub with_prefix")]
    /// The component in the î direction.
    pub x: f32,
    #[getset(get_copy = "pub with_prefix")]
    /// The component in the ĵ direction.
    pub y: f32,
    #[getset(get_copy = "pub with_prefix")]
    /// The component in the k̂ direction.
    pub z: f32,
}

impl Vec3 {
    /// The î unit vector.
    pub const I: Vec3 = Self::new(1.0, 0.0, 0.0);
    /// The ĵ unit vector.
    pub const J: Vec3 = Self::new(0.0, 1.0, 0.0);
    /// The k̂ unit vector.
    pub const K: Vec3 = Self::new(0.0, 0.0, 1.0);
    /// The zero vector.
    pub const ZERO: Vec3 = Self::new(0.0, 0.0, 0.0);

    /// Construct a new `Vec3` with value xî + yĵ + zk̂.
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Produce `num_points` vectors evenly spaced across the interval `-extent` to `extent`.
    pub fn symmetric_linspace(
        extent: Self,
        num_points: usize,
    ) -> impl ExactSizeIterator<Item = Self> {
        (-extent..=extent).linspace(num_points)
    }
}

impl Add for Vec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Sub<&Self> for Vec3 {
    type Output = Self;

    fn sub(self, rhs: &Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl Div<f32> for Vec3 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl Display for Vec3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:+.4}i{:+.4}j{:+.4}k", self.x, self.y, self.z)
    }
}

impl From<&SphericalPoint3> for Vec3 {
    fn from(point: &SphericalPoint3) -> Self {
        Self::new(point.x(), point.y(), point.z())
    }
}

impl From<Vec3> for SphericalPoint3 {
    fn from(vec: Vec3) -> Self {
        Self::new(vec.x, vec.y, vec.z)
    }
}

impl From<&Vec3> for SphericalPoint3 {
    fn from(vec: &Vec3) -> Self {
        Self::new(vec.x, vec.y, vec.z)
    }
}
