use std::f32::consts::PI;
use std::fmt;

use getset::CopyGetters;

/// A point in `R^3`.
///
/// Note that we use the physics convention of (r, theta, phi): theta is the inclination
/// and phi is the azimuth.
///
/// # Invariants
/// The spherical elements must be kept in sync with Cartesian elements. For this reason,
/// direct (i.e., mutable) access to struct members is not allowed.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct SphericalPoint3 {
    /// Cartesian x.
    pub(super) x: f32,
    /// Cartesian y.
    pub(super) y: f32,
    /// Cartesian z.
    pub(super) z: f32,
    /// Spherical radius.
    pub(super) r: f32,
    /// Cosine of spherical longitude.
    pub(super) cos_theta: f32,
    /// Spherical azimuth.
    pub(super) phi: f32,
}

impl fmt::Display for SphericalPoint3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:.4}, {:.4}, {:.4})", self.x, self.y, self.z)
    }
}

impl SphericalPoint3 {
    /// A point representing the origin.
    pub const ORIGIN: SphericalPoint3 = SphericalPoint3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        r: 0.0,
        cos_theta: 1.0,
        phi: 0.0,
    };
    /// The origin, but offset in z by [`f32::EPSILON`] for when division-by-zero needs to be avoided.
    pub const ORIGIN_EPSILON: SphericalPoint3 = SphericalPoint3 {
        x: 0.0,
        y: 0.0,
        z: f32::EPSILON,
        r: f32::EPSILON,
        cos_theta: 1.0,
        phi: 0.0,
    };

    /// Construct a new `Point` at Cartesian position (x, y, z).
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        let r = (x * x + y * y + z * z).sqrt();
        Self {
            x,
            y,
            z,
            r,
            cos_theta: if r == 0.0 { 1.0 } else { z / r }, // Handle degeneracy.
            phi: {
                let atan2 = y.atan2(x);
                if atan2.is_sign_positive() {
                    atan2
                } else {
                    2.0 * PI + atan2
                }
            },
        }
    }

    /// Construct a new `Point` at spherical position (r, theta, phi).
    pub fn new_spherical(r: f32, theta: f32, phi: f32) -> Self {
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();
        Self {
            x: r * sin_theta * phi.cos(),
            y: r * sin_theta * phi.sin(),
            z: r * cos_theta,
            r,
            cos_theta,
            phi,
        }
    }
}

/// A point and the value of a function evaluated at that point.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointValue<T>(pub SphericalPoint3, pub T);

#[cfg(test)]
mod tests {
    use approx::assert_ulps_eq;

    use crate::geometry::SphericalPoint3;

    #[test]
    fn spherical_coordinates() {
        let point = SphericalPoint3::new(1.0, 2.0, -3.0);
        assert_ulps_eq!(point.r, 3.7416573867739413856, max_ulps = 1);
        assert_ulps_eq!(point.cos_theta.acos(), 2.5010703409103686643, max_ulps = 1);
        assert_ulps_eq!(point.phi, 1.1071487177940905030, max_ulps = 1);
        let recomputed_point =
            SphericalPoint3::new_spherical(point.r, point.cos_theta.acos(), point.phi);
        assert_ulps_eq!(point.x, recomputed_point.x, max_ulps = 1);
        assert_ulps_eq!(point.y, recomputed_point.y, max_ulps = 1);
        assert_ulps_eq!(point.z, recomputed_point.z, max_ulps = 1);
    }
}
