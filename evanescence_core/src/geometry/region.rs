use std::f32::consts::PI;

use na::{vector, Point, Point3, Vector3};

use super::point::{IPoint, SphericalPoint3};
use crate::numerics::random::WyRand;

pub trait Region<const N: usize, P: IPoint<N> = Point<f32, N>>: PartialEq + Clone {
    fn sample(&self, rng: &mut WyRand) -> P;
}

pub trait BoundingRegion<const N: usize, P: IPoint<N> = Point<f32, N>> {
    type Geometry: Region<N, P>;
    fn bounding_region(&self) -> Self::Geometry;
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct BallCenteredAtOrigin {
    pub radius: f32,
}

/// Produce random points uniformly distributed within a ball of the given `radius`.
///
/// Reference: <http://extremelearning.com.au/how-to-generate-uniformly-random-points-on-n-spheres-and-n-balls/>,
/// specifically method 16.
#[inline]
fn rand_in_3_ball(radius: f32, rng: &mut WyRand) -> (Vector3<f32>, f32, f32, f32) {
    // For an explanation of taking the cube root of the random value, see
    // https://stackoverflow.com/a/50746409.
    let [r, cos_theta] = rng.gen_f32x2();
    let r /* [0, radius] */ = r.cbrt() * radius;
    let cos_theta /* [-1, 1] */ = cos_theta * 2.0 - 1.0;
    let sin_theta = (1. - cos_theta * cos_theta).sqrt();
    let phi /* [0, 2pi) */ = rng.gen_f32() * 2.0 * PI;
    let cartesian = vector![
        r * sin_theta * phi.cos(),
        r * sin_theta * phi.sin(),
        r * cos_theta
    ];
    (cartesian, r, cos_theta, phi)
}

// Note: two manual impls and no blanket impl to avoid specialization.
impl Region<3> for BallCenteredAtOrigin {
    #[inline]
    fn sample(&self, rng: &mut WyRand) -> Point3<f32> {
        let (cartesian, ..) = rand_in_3_ball(self.radius, rng);
        cartesian.into()
    }
}
impl Region<3, SphericalPoint3> for BallCenteredAtOrigin {
    #[inline]
    fn sample(&self, rng: &mut WyRand) -> SphericalPoint3 {
        let (cartesian, r, cos_theta, phi) = rand_in_3_ball(self.radius, rng);
        SphericalPoint3::new_unvalidated(cartesian, r, cos_theta, phi)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct CubeCenteredAtOrigin {
    pub side_length: f32,
}

impl<P: IPoint<3>> Region<3, P> for CubeCenteredAtOrigin {
    fn sample(&self, rng: &mut WyRand) -> P {
        let [x, y] = rng.gen_f32x2();
        let z = rng.gen_f32();
        let bottom_left = -Vector3::from_element(self.side_length / 2.);
        (vector![x, y, z] * self.side_length + bottom_left).into()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RectangularPrism {
    pub bottom_left: Vector3<f32>,
    pub side_lengths: Vector3<f32>,
}

impl RectangularPrism {
    pub fn top_right(&self) -> Vector3<f32> {
        self.bottom_left + self.side_lengths
    }

    pub fn clamp(&self, mut pt: Vector3<f32>) -> Vector3<f32> {
        for (x_i, min, max) in itertools::izip!(&mut pt, &self.bottom_left, &self.top_right()) {
            *x_i = x_i.clamp(*min, *max);
        }
        pt
    }
}

impl Region<3, Point3<f32>> for RectangularPrism {
    fn sample(&self, rng: &mut WyRand) -> Point3<f32> {
        let [x, y] = rng.gen_f32x2();
        let z = rng.gen_f32();
        (vector![x, y, z].component_mul(&self.side_lengths) + self.bottom_left).into()
    }
}

#[cfg(test)]
mod tests {
    use std::iter;

    use approx::assert_ulps_eq;

    use super::{BallCenteredAtOrigin, Region};
    use crate::geometry::point::{IPoint, SphericalCoordinatesExt, SphericalPoint3};
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
            .for_each(|pt: SphericalPoint3| assert!(pt.r() < radius));
    }

    #[test]
    fn rng_spherical_coordinates() {
        let ball = BallCenteredAtOrigin { radius: 2. };
        let rng = &mut WyRand::new();
        for _ in 0..200 {
            let rng_point: SphericalPoint3 = ball.sample(rng);
            let recomputed_point = SphericalPoint3::from(*rng_point.coordinates());
            println!("rand point: {rng_point:?}");
            assert_ulps_eq!(rng_point.r(), recomputed_point.r(), max_ulps = 2);
            assert_ulps_eq!(
                rng_point.cos_theta(),
                recomputed_point.cos_theta(),
                max_ulps = 2
            );
            assert_ulps_eq!(rng_point.phi(), recomputed_point.phi(), max_ulps = 2);
        }
    }
}
