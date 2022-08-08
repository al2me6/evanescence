use super::Evaluate;
use crate::geometry::region::BoundingRegion;
use crate::geometry::{Point, PointValue};

pub mod kolmogorov_smirnov;

/// An [`Evaluate`]-able type that can also be interpreted as a probability density function.
pub trait Distribution: Evaluate {
    /// Give the probability density corresponding to a `value` of the underlying function.
    fn probability_density_of(&self, value: Self::Output) -> f32;

    #[inline]
    fn probability_density(&self, point: &Point) -> f32 {
        self.probability_density_of(self.evaluate(point))
    }

    #[inline]
    fn evaluate_with_probability_density(&self, point: &Point) -> (Self::Output, f32) {
        let output = self.evaluate(point);
        let prob_density = self.probability_density_of(output);
        (output, prob_density)
    }

    #[inline]
    fn evaluate_at_with_probability_density(
        &self,
        point: &Point,
    ) -> (PointValue<Self::Output>, f32) {
        let output = self.evaluate_at(point);
        let prob_density = self.probability_density_of(output.1);
        (output, prob_density)
    }
}

/// Type that evaluates the probability density of a [`Distribution`].
///
/// Example:
///
/// ```
/// use approx::assert_relative_eq;
/// use evanescence_core::geometry::Point;
/// use evanescence_core::numerics::statistics::ProbabilityDensityEvaluator;
/// use evanescence_core::numerics::Evaluate;
/// use evanescence_core::orbital::{Qn, Real};
///
/// let qn = Qn::new(3, 2, 1).unwrap();
///
/// assert_relative_eq!(
///     2.446E-4,
///     ProbabilityDensityEvaluator::new(Real::new(qn)).evaluate(&Point::new(6.0, -0.3, 8.5))
/// );
/// ```
pub struct ProbabilityDensityEvaluator<D>(D);

impl<D> ProbabilityDensityEvaluator<D> {
    pub fn new(inner: D) -> Self {
        Self(inner)
    }
}

impl<D: Distribution> Evaluate for ProbabilityDensityEvaluator<D> {
    type Output = f32;

    #[inline]
    fn evaluate(&self, point: &Point) -> Self::Output {
        self.0.probability_density_of(self.0.evaluate(point))
    }
}

impl<D: Distribution + BoundingRegion> BoundingRegion for ProbabilityDensityEvaluator<D> {
    type Geometry = D::Geometry;

    #[inline]
    fn bounding_region(&self) -> Self::Geometry {
        self.0.bounding_region()
    }
}
