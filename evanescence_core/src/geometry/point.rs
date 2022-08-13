use std::f32::consts::PI;

use na::{vector, SVector, Vector3};

/// A rudimentary wrapper trait around [`na::Point`] to allow
/// [`numerics::Function`](crate::numerics::Function) to be generic over the
/// custom type [`SphericalPoint3`] as well as normal nalgebra points.
///
/// The scalar type is `f32`.
///
/// Note that spherical coordinate caching is crucial for atomic wavefunction performance.
pub trait IPoint<const N: usize>:
    PartialEq + Clone + From<SVector<f32, N>> + std::fmt::Debug
{
    fn origin() -> Self;
    fn coordinates(&self) -> &SVector<f32, N>;
}

impl<const N: usize> IPoint<N> for na::Point<f32, N> {
    fn origin() -> Self {
        na::Point::<f32, N>::origin()
    }

    fn coordinates(&self) -> &SVector<f32, N> {
        &self.coords
    }
}

/// Note that we use the physics convention of (r, theta, phi): theta is the inclination
/// and phi is the azimuth.
pub trait SphericalPoint3Ext: IPoint<3> {
    fn r(&self) -> f32;
    fn cos_theta(&self) -> f32;
    fn phi(&self) -> f32;
}

/// A [`na::Point3<f32>`] along with cached spherical coordinates.
/// # Invariants
/// The spherical coordinates must always correspond to the Cartesian coordinates in `self.pt`.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct SphericalPoint3 {
    pt: na::Point3<f32>,
    r: f32,
    cos_theta: f32,
    phi: f32,
}

impl SphericalPoint3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        vector![x, y, z].into()
    }

    pub fn new_spherical(r: f32, theta: f32, phi: f32) -> Self {
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();
        vector![
            r * sin_theta * phi.cos(),
            r * sin_theta * phi.sin(),
            r * cos_theta
        ]
        .into()
    }

    /// Construct a new `SphericalPoint3` with the provided Cartesian and spherical coordinates,
    /// _without checking that the passed values are coherent_. This is the responsibility of the
    /// caller and failure to uphold the invariant can and will result in erroneous (but not
    /// Undefined) behavior.
    pub fn new_unvalidated(cartesian: SVector<f32, 3>, r: f32, cos_theta: f32, phi: f32) -> Self {
        Self {
            pt: cartesian.into(),
            r,
            cos_theta,
            phi,
        }
    }
}

impl From<SVector<f32, 3>> for SphericalPoint3 {
    #[inline]
    fn from(coords: SVector<f32, 3>) -> Self {
        let r = coords.norm();
        Self {
            r,
            cos_theta: if r == 0.0 { 1.0 } else { coords.z / r }, // Handle degeneracy.
            phi: {
                let atan2 = coords.y.atan2(coords.x);
                if atan2.is_sign_positive() {
                    atan2
                } else {
                    2.0 * PI + atan2
                }
            },
            pt: coords.into(),
        }
    }
}

impl IPoint<3> for SphericalPoint3 {
    fn origin() -> Self {
        Self {
            pt: na::Point3::origin(),
            r: 0.,
            cos_theta: 1.,
            phi: 0.,
        }
    }

    fn coordinates(&self) -> &Vector3<f32> {
        &self.pt.coords
    }
}

impl SphericalPoint3Ext for SphericalPoint3 {
    fn r(&self) -> f32 {
        self.r
    }

    fn cos_theta(&self) -> f32 {
        self.cos_theta
    }

    fn phi(&self) -> f32 {
        self.phi
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_ulps_eq;
    use na::vector;

    use crate::geometry::point::{IPoint, SphericalPoint3};

    #[test]
    fn spherical_coordinates() {
        let point = SphericalPoint3::from(vector![1.0, 2.0, -3.0]);
        assert_ulps_eq!(point.r, 3.7416573867739413856, max_ulps = 1);
        assert_ulps_eq!(point.cos_theta.acos(), 2.5010703409103686643, max_ulps = 1);
        assert_ulps_eq!(point.phi, 1.1071487177940905030, max_ulps = 1);
        let recomputed_point =
            SphericalPoint3::new_spherical(point.r, point.cos_theta.acos(), point.phi);
        assert_ulps_eq!(
            point.coordinates(),
            recomputed_point.coordinates(),
            max_ulps = 1
        );
    }
}
