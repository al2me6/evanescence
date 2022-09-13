pub mod kolmogorov_smirnov;

use std::iter;
use std::marker::PhantomData;
use std::ops::RangeInclusive;

use na::{vector, Point};

use super::Function;
use crate::geometry::point::IPoint;
use crate::geometry::region::BoundingRegion;
use crate::geometry::storage::{PointValue, Soa};

/// Marker trait indicating that a [`Function`] _is_ a probability density function.
pub trait Distribution<const N: usize, P: IPoint<N> = Point<f32, N>>:
    Function<N, P, Output = f32>
{
}

/// Trait describing _the interpretation of a [`Function`] as_ a probability density function.
///
/// In the context of quantum mechanics, this trait may be used to represent probability amplitudes.
pub trait AsDistribution<const N: usize, P: IPoint<N> = Point<f32, N>>: Function<N, P> {
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

impl<const N: usize, P, F> AsDistribution<N, P> for F
where
    F: Distribution<N, P>,
    P: IPoint<N>,
{
    #[inline]
    fn probability_density_of(&self, value: Self::Output) -> f32 {
        value
    }
}

/// Wrapper type for evaluating the PDF of a [`Function`] that is [`AsDistribution`].
///
/// Example:
///
/// ```
/// use approx::assert_relative_eq;
/// use evanescence_core::geometry::point::SphericalPoint3;
/// use evanescence_core::numerics::statistics::Pdf;
/// use evanescence_core::numerics::Function;
/// use evanescence_core::orbital::{AtomicReal, Qn};
///
/// let qn = Qn::new(3, 2, 1).unwrap();
///
/// assert_relative_eq!(
///     2.446E-4,
///     Pdf::new(AtomicReal::new(qn)).evaluate(&SphericalPoint3::new(6.0, -0.3, 8.5))
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
    D: AsDistribution<N, P>,
{
    type Output = f32;

    #[inline]
    fn evaluate(&self, point: &P) -> Self::Output {
        self.inner
            .probability_density_of(self.inner.evaluate(point))
    }
}

// Todo: impl this on `D: Distribution` instead.
impl<P, D> Pdf<1, P, D>
where
    P: IPoint<1>,
    D: AsDistribution<1, P>,
{
    pub fn sample_cdf(&self, interval: RangeInclusive<f32>, count: usize) -> Soa<1, f32> {
        let step = (interval.end() - interval.start()) / (count as f32 - 1.);
        let mut x = *interval.start();
        let mut cdf = 0.;
        Iterator::chain(
            iter::once((vector![x], cdf)),
            iter::repeat_with(|| {
                super::integrators::integrate_simpson_step(
                    |xx| self.evaluate(&vector![xx].into()),
                    &mut x,
                    &mut cdf,
                    step,
                );
                (vector![x], cdf)
            }),
        )
        .take(count)
        .collect()
    }
}

impl<const N: usize, P, D> BoundingRegion<N, P> for Pdf<N, P, D>
where
    P: IPoint<N>,
    D: AsDistribution<N, P> + BoundingRegion<N, P>,
{
    type Geometry = D::Geometry;

    #[inline]
    fn bounding_region(&self) -> Self::Geometry {
        self.inner.bounding_region()
    }
}
