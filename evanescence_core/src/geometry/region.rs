use std::f32::consts::PI;

use super::SphericalPoint3;
use crate::numerics::random::WyRand;

pub trait Region: PartialEq + Clone {
    fn sample(&self, rng: &mut WyRand) -> SphericalPoint3;
}

pub trait BoundingRegion {
    type Geometry: Region;
    fn bounding_region(&self) -> Self::Geometry;
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct BallCenteredAtOrigin {
    pub radius: f32,
}

impl Region for BallCenteredAtOrigin {
    /// Produce random points uniformly distributed within `self`.
    ///
    /// Reference: <http://extremelearning.com.au/how-to-generate-uniformly-random-points-on-n-spheres-and-n-balls/>,
    /// specifically method 16.
    #[inline]
    fn sample(&self, rng: &mut WyRand) -> SphericalPoint3 {
        // For an explanation of taking the cube root of the random value, see
        // https://stackoverflow.com/a/50746409.
        let [r, cos_theta] = rng.gen_f32x2();
        let r /* [0, radius] */ = r.cbrt() * self.radius;
        let cos_theta /* [-1, 1] */ = cos_theta * 2.0 - 1.0;
        // Pythagorean identity: sin^2(x) + cos^2(x) = 1.
        let sin_theta = (1.0 - cos_theta.powi(2)).sqrt();
        let phi /* [0, 2pi) */ = rng.gen_f32() * 2.0 * PI;
        SphericalPoint3 {
            x: r * sin_theta * phi.cos(),
            y: r * sin_theta * phi.sin(),
            z: r * cos_theta,
            r,
            cos_theta,
            phi,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::iter;

    use approx::assert_ulps_eq;

    use super::{BallCenteredAtOrigin, Region};
    use crate::geometry::SphericalPoint3;
    use crate::numerics::random::WyRand;

    /// This is very crude and only ensures that all pointsare at least inside
    /// the expected radius. It makes no attempt to verify the uniformity of
    /// the distribution produced.
    #[test]
    fn point_rng_max_radius() {
        let radius = 2_f32;
        let mut rng = WyRand::new();
        let ball = BallCenteredAtOrigin { radius };
        iter::repeat_with(|| ball.sample(&mut rng))
            .take(10_000)
            .for_each(|pt| assert!(pt.r < radius));
    }

    #[test]
    fn rng_spherical_coordinates() {
        let ball = BallCenteredAtOrigin { radius: 2. };
        let mut rng = WyRand::new();
        let rng_point = ball.sample(&mut rng);
        let recomputed_point = SphericalPoint3::new(rng_point.x, rng_point.y, rng_point.z);
        assert_ulps_eq!(rng_point.r, recomputed_point.r, max_ulps = 1);
        assert_ulps_eq!(
            rng_point.cos_theta,
            recomputed_point.cos_theta,
            max_ulps = 1
        );
        assert_ulps_eq!(rng_point.phi, recomputed_point.phi, max_ulps = 1);
    }
}
