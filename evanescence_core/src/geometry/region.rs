use std::f32::consts::PI;

use nanorand::{Rng, WyRand};

use super::Point;

pub trait Region {
    fn sample(&self, rng: &mut WyRand) -> Point;
}

pub trait BoundingRegion {
    type Geometry: Region;
    fn bounding_region(&self) -> Self::Geometry;
}

pub struct BallCenteredAtOrigin {
    pub radius: f32,
}

impl Region for BallCenteredAtOrigin {
    #[inline]
    fn sample(&self, rng: &mut WyRand) -> Point {
        // For an explanation of taking the cube root of the random value, see
        // https://stackoverflow.com/a/50746409.
        let r /* [0, radius] */ = rng.generate::<f32>().cbrt() * self.radius;
        let cos_theta /* [-1, 1] */ = rng.generate::<f32>() * 2.0 - 1.0;
        // Pythagorean identity: sin^2(x) + cos^2(x) = 1.
        let sin_theta = (1.0 - cos_theta.powi(2)).sqrt();
        let phi /* [0, 2pi) */ = rng.generate::<f32>() * 2.0 * PI;
        Point {
            x: r * sin_theta * phi.cos(),
            y: r * sin_theta * phi.sin(),
            z: r * cos_theta,
            r,
            cos_theta,
            phi,
        }
    }
}
