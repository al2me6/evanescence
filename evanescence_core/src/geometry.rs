use getset::CopyGetters;
use std::{f64::consts::PI, iter};

use crate::numerics::new_rng;

/// A point in 3D space.
///
/// Note that we use the physics convention of (r, theta, phi): theta is the inclination
/// and phi is the azimuth.
#[derive(Clone, Copy, Debug, PartialEq, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Point {
    /// Cartesian x.
    x: f64,
    /// Cartesian y.
    y: f64,
    /// Cartesian z.
    z: f64,
    /// Spherical radius.
    r: f64,
    /// Cosine of spherical longitude.
    cos_theta: f64,
    /// Spherical azimuth.
    phi: f64,
}

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

impl Point {
    /// Construct a new Point at Cartesian position (x, y, z).
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        let r = (x * x + y * y + z * z).sqrt();
        Self {
            x,
            y,
            z,
            r,
            cos_theta: z / r,
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

    /// Construct a new Point at spherical position (r, theta, phi).
    pub fn new_spherical(r: f64, theta: f64, phi: f64) -> Self {
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
    pub fn sample_from_ball_iter(radius: f64) -> impl Iterator<Item = Self> {
        let mut rng = new_rng();
        iter::repeat_with(move || {
            // For an explanation of taking the cube root of the random value, see
            // https://stackoverflow.com/a/50746409.
            let r /* [0, radius] */ = rng.rand_float().cbrt() * radius;
            let cos_theta /* [-1, 1] */ = rng.rand_float() * 2.0 - 1.0;
            // Pythagorean identity: sin^2(x) + cos^2(x) = 1.
            let sin_theta = (1.0 - cos_theta.powi(2)).sqrt();
            let phi /* [0, 2pi) */ = rng.rand_float() * 2.0 * PI;
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
    /// # use evanescence_core::geometry::Point;
    /// assert_eq!(
    ///     Some(Point::ORIGIN_EPSILON),
    ///     Point::sample_from_ball_with_origin_iter(1.0).next()
    /// );
    /// ```
    pub fn sample_from_ball_with_origin_iter(radius: f64) -> impl Iterator<Item = Self> {
        iter::once(Self::ORIGIN_EPSILON).chain(Self::sample_from_ball_iter(radius))
    }

    /// A point representing the origin.
    pub const ORIGIN: Point = Point {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        r: 0.0,
        cos_theta: 1.0,
        phi: 0.0,
    };

    /// The origin, but offset in z by [`f64::EPSILON`] for when division-by-zero needs to be avoided.
    pub const ORIGIN_EPSILON: Point = Point {
        x: 0.0,
        y: 0.0,
        z: f64::EPSILON,
        r: f64::EPSILON,
        cos_theta: 1.0,
        phi: 0.0,
    };
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use crate::geometry::Point;

    /// This is very crude and only ensures that all pointsare at least inside
    /// the expected radius. It makes no attempt to verify the uniformity of
    /// the distribution produced.
    #[test]
    fn test_point_rng_max_radius() {
        let sampling_radius = 2_f64;
        Point::sample_from_ball_iter(sampling_radius)
            .take(10_000)
            .for_each(|pt| assert!(pt.r < sampling_radius));
    }

    #[test]
    fn test_spherical_coordinates() {
        let point = Point::new(1.0, 2.0, -3.0);
        assert_relative_eq!(point.r, 3.7416573867739413856);
        assert_relative_eq!(point.cos_theta.acos(), 2.5010703409103686643);
        assert_relative_eq!(point.phi, 1.1071487177940905030);
        let recomputed_point = Point::new_spherical(point.r, point.cos_theta.acos(), point.phi);
        assert_relative_eq!(point.x, recomputed_point.x);
        assert_relative_eq!(point.y, recomputed_point.y);
        assert_relative_eq!(point.z, recomputed_point.z);
    }

    #[test]
    fn test_rng_spherical_coordinates() {
        let rng_point = Point::sample_from_ball_iter(2.0).next().unwrap();
        let recomputed_point = Point::new(rng_point.x, rng_point.y, rng_point.z);
        assert_relative_eq!(rng_point.r, recomputed_point.r);
        assert_relative_eq!(rng_point.cos_theta, recomputed_point.cos_theta);
        assert_relative_eq!(rng_point.phi, recomputed_point.phi);
    }

    // #[test]
    // fn print_random_points() {
    //     Point::random_in_ball_iter(10.0)
    //         .take(10)
    //         .for_each(|pt| println!("Point::new({}, {}, {}),", pt.x, pt.y, pt.z));
    // }
}
