use std::marker::PhantomData;

use super::Function;
use crate::geometry::point::IPoint;
use crate::geometry::region::BoundingRegion;
use crate::geometry::storage::PointValue;

pub mod kolmogorov_smirnov;

/// A [`Function`] that can also be interpreted as a probability density function.
pub trait Distribution<const N: usize, P: IPoint<N>>: Function<N, P> {
    /// Give the probability density corresponding to a `value` of the underlying function.
    fn probability_density_of(&self, value: Self::Output) -> f32;

    #[inline]
    fn probability_density(&self, point: &P) -> f32 {
        self.probability_density_of(self.evaluate(point))
    }

    #[inline]
    fn evaluate_with_probability_density(&self, point: &P) -> (Self::Output, f32) {
        let output = self.evaluate(point);
        let prob_density = self.probability_density_of(output);
        (output, prob_density)
    }

    #[inline]
    fn evaluate_at_with_probability_density(
        &self,
        point: P,
    ) -> (PointValue<N, P, Self::Output>, f32) {
        let output = self.evaluate_at(point);
        let prob_density = self.probability_density_of(output.1);
        (output, prob_density)
    }
}

/// The probability density function of a [`Distribution`].
///
/// Example:
///
/// ```
/// use approx::assert_relative_eq;
/// use evanescence_core::geometry::point::SphericalPoint3;
/// use evanescence_core::numerics::statistics::Pdf;
/// use evanescence_core::numerics::Function;
/// use evanescence_core::orbital::{Qn, Real};
///
/// let qn = Qn::new(3, 2, 1).unwrap();
///
/// assert_relative_eq!(
///     2.446E-4,
///     Pdf::new(Real::new(qn)).evaluate(&SphericalPoint3::new(6.0, -0.3, 8.5))
/// );
/// ```
pub struct Pdf<const N: usize, P, D> {
    inner: D,
    _phantom: PhantomData<P>,
}

impl<const N: usize, P, D> Pdf<N, P, D> {
    pub fn new(inner: D) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }
}

impl<const N: usize, P, D> Function<N, P> for Pdf<N, P, D>
where
    P: IPoint<N>,
    D: Distribution<N, P>,
{
    type Output = f32;

    #[inline]
    fn evaluate(&self, point: &P) -> Self::Output {
        self.inner
            .probability_density_of(self.inner.evaluate(point))
    }
}

impl<const N: usize, P, D> BoundingRegion<N, P> for Pdf<N, P, D>
where
    P: IPoint<N>,
    D: Distribution<N, P> + BoundingRegion<N, P>,
{
    type Geometry = D::Geometry;

    #[inline]
    fn bounding_region(&self) -> Self::Geometry {
        self.inner.bounding_region()
    }
}