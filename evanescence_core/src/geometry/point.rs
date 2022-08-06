use std::f32::consts::PI;
use std::{fmt, iter};

use getset::CopyGetters;
use nanorand::{Rng, WyRand};

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
pub struct Point {
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

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:.4}, {:.4}, {:.4})", self.x, self.y, self.z)
    }
}

impl Point {
    /// A point representing the origin.
    pub const ORIGIN: Point = Point {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        r: 0.0,
        cos_theta: 1.0,
        phi: 0.0,
    };
    /// The origin, but offset in z by [`f32::EPSILON`] for when division-by-zero needs to be avoided.
    pub const ORIGIN_EPSILON: Point = Point {
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

    /// Produce random points uniformly distributed within a ball of the given radius.
    ///
    /// Reference: <http://extremelearning.com.au/how-to-generate-uniformly-random-points-on-n-spheres-and-n-balls/>,
    /// specifically method 16.
    pub fn sample_from_ball_iter(radius: f32, rng: &mut WyRand) -> impl Iterator<Item = Self> + '_ {
        iter::repeat_with(move || {
            // For an explanation of taking the cube root of the random value, see
            // https://stackoverflow.com/a/50746409.
            let r /* [0, radius] */ = rng.generate::<f32>().cbrt() * radius;
            let cos_theta /* [-1, 1] */ = rng.generate::<f32>() * 2.0 - 1.0;
            // Pythagorean identity: sin^2(x) + cos^2(x) = 1.
            let sin_theta = (1.0 - cos_theta.powi(2)).sqrt();
            let phi /* [0, 2pi) */ = rng.generate::<f32>() * 2.0 * PI;
            Self {
                x: r * sin_theta * phi.cos(),
                y: r * sin_theta * phi.sin(),
                z: r * cos_theta,
                r,
                cos_theta,
                phi,
            }
        })
    }

    /// Same as [`Point::sample_from_ball_iter`], but with [`Point::ORIGIN_EPSILON`] guaranteed as
    /// the first point sampled:
    /// ```
    /// use evanescence_core::geometry::Point;
    /// let mut rng = nanorand::WyRand::new();
    /// assert_eq!(
    ///     Some(Point::ORIGIN_EPSILON),
    ///     Point::sample_from_ball_with_origin_iter(1.0, &mut rng).next()
    /// );
    /// ```
    pub fn sample_from_ball_with_origin_iter(
        radius: f32,
        rng: &mut WyRand,
    ) -> impl Iterator<Item = Self> + '_ {
        iter::once(Self::ORIGIN_EPSILON).chain(Self::sample_from_ball_iter(radius, rng))
    }
}

/// A point and the value of a function evaluated at that point.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointValue<T>(pub Point, pub T);

#[cfg(test)]
mod tests {
    use approx::assert_ulps_eq;
    use nanorand::WyRand;

    use crate::geometry::Point;

    /// This is very crude and only ensures that all pointsare at least inside
    /// the expected radius. It makes no attempt to verify the uniformity of
    /// the distribution produced.
    #[test]
    fn point_rng_max_radius() {
        let sampling_radius = 2_f32;
        let mut rng = WyRand::new();
        Point::sample_from_ball_iter(sampling_radius, &mut rng)
            .take(10_000)
            .for_each(|pt| assert!(pt.r < sampling_radius));
    }

    #[test]
    fn spherical_coordinates() {
        let point = Point::new(1.0, 2.0, -3.0);
        assert_ulps_eq!(point.r, 3.7416573867739413856, max_ulps = 1);
        assert_ulps_eq!(point.cos_theta.acos(), 2.5010703409103686643, max_ulps = 1);
        assert_ulps_eq!(point.phi, 1.1071487177940905030, max_ulps = 1);
        let recomputed_point = Point::new_spherical(point.r, point.cos_theta.acos(), point.phi);
        assert_ulps_eq!(point.x, recomputed_point.x, max_ulps = 1);
        assert_ulps_eq!(point.y, recomputed_point.y, max_ulps = 1);
        assert_ulps_eq!(point.z, recomputed_point.z, max_ulps = 1);
    }

    #[test]
    fn rng_spherical_coordinates() {
        let mut rng = WyRand::new();
        let rng_point = Point::sample_from_ball_iter(2.0, &mut rng).next().unwrap();
        let recomputed_point = Point::new(rng_point.x, rng_point.y, rng_point.z);
        assert_ulps_eq!(rng_point.r, recomputed_point.r, max_ulps = 1);
        assert_ulps_eq!(
            rng_point.cos_theta,
            recomputed_point.cos_theta,
            max_ulps = 1
        );
        assert_ulps_eq!(rng_point.phi, recomputed_point.phi, max_ulps = 1);
    }

    // #[test]
    // fn print_random_points() {
    //     Point::random_in_ball_iter(10.0)
    //         .take(10)
    //         .for_each(|pt| println!("Point::new({}, {}, {}),", pt.x, pt.y, pt.z));
    // }
}
