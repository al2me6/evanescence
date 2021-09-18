//! Implementations of real and complex hydrogen-atom orbitals.
//!
//! Access to radial and angular components, as well as related functions (ex. radial probability
//! and probability density) are available through the [`wavefunctions`] module.
//!
//! Types for quantum numbers are available through the [`quantum_numbers`] module.
pub mod hybrid;
pub mod quantum_numbers;
pub mod wavefunctions;

use std::marker::PhantomData;

use num_complex::Complex32;

pub use self::quantum_numbers::Qn;
use self::wavefunctions::{
    Radial, RadialProbabilityDistribution, RealSphericalHarmonic, SphericalHarmonic,
};
use crate::geometry::{ComponentForm, Point, Vec3};
use crate::numerics::{Evaluate, EvaluateBounded};

/// Get the conventional subshell name (s, p, d, f, etc.) for common (i.e., small) values of `l`;
/// will otherwise return `None`.
pub fn subshell_name(l: u32) -> Option<&'static str> {
    match l {
        0 => Some("s"),
        1 => Some("p"),
        2 => Some("d"),
        3 => Some("f"),
        4 => Some("g"),
        5 => Some("h"),
        6 => Some("i"),
        7 => Some("k"), // Note that "j" is skipped!
        _ => None,
    }
}

/// Trait representing a type of wavefunction.
pub trait Orbital: EvaluateBounded {
    /// Give the probability density value corresponding to a certain wavefunction value.
    fn probability_density_of(value: Self::Output) -> f32;

    /// Give the conventional name of an orbital.
    ///
    /// Superscripts are represented using Unicode superscript symbols and subscripts are
    /// represented with the HTML tag `<sub></sub>`.
    fn name(params: &Self::Parameters) -> String;
}

/// Implementation of the real hydrogenic orbitals.
pub struct Real;

impl Evaluate for Real {
    type Output = f32;
    type Parameters = Qn;

    #[inline]
    fn evaluate(qn: &Qn, point: &Point) -> f32 {
        Radial::evaluate(&qn.into(), point) * RealSphericalHarmonic::evaluate(&qn.into(), point)
    }
}

impl EvaluateBounded for Real {
    /// This is an empirically derived heuristic. See the attached Mathematica notebook
    /// `radial_wavefunction.nb` for plots.
    #[inline]
    fn bound(qn: &Qn) -> f32 {
        let n = qn.n() as f32;
        0.9 * n * (2.5 * n - 0.625 * qn.l() as f32 + 3.0)
    }
}

impl Orbital for Real {
    #[inline]
    fn probability_density_of(value: f32) -> f32 {
        value * value
    }

    /// Try to give the orbital's conventional name (ex. `4d_{z^2}`) before falling back to giving
    /// the quantum numbers only (ex. `ψ_{420}`).
    fn name(qn: &Qn) -> String {
        if let (Some(subshell), Some(linear_combination)) = (
            subshell_name(qn.l()),
            RealSphericalHarmonic::expression(&qn.into()),
        ) {
            format!("{}{}<sub>{}</sub>", qn.n(), subshell, linear_combination)
        } else {
            Complex::name(qn)
        }
    }
}

impl Real {
    /// Give the number of radial nodes in an orbital.
    pub fn num_radial_nodes(qn: &Qn) -> u32 {
        qn.n() - qn.l() - 1
    }

    /// Give the number of angular nodes in an orbital.
    pub fn num_angular_nodes(qn: &Qn) -> u32 {
        qn.l()
    }
}

/// Implementation of the complex hydrogenic orbitals.
pub struct Complex;

impl Evaluate for Complex {
    type Output = Complex32;
    type Parameters = Qn;

    #[inline]
    fn evaluate(qn: &Qn, point: &Point) -> Complex32 {
        Radial::evaluate(&qn.into(), point) * SphericalHarmonic::evaluate(&qn.into(), point)
    }
}

impl EvaluateBounded for Complex {
    #[inline]
    fn bound(params: &Self::Parameters) -> f32 {
        Real::bound(params)
    }
}

impl Orbital for Complex {
    #[inline]
    fn probability_density_of(value: Self::Output) -> f32 {
        let norm = value.norm();
        norm * norm
    }

    /// Give the name of the wavefunction (ex. `ψ_{420}`).
    fn name(qn: &Qn) -> String {
        format!("ψ<sub>{}{}{}</sub>", qn.n(), qn.l(), qn.m())
    }
}

/// A radially symmetrical property associated with an orbital.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RadialPlot {
    Wavefunction,
    ProbabilityDistribution,
}

/// Compute a plot of a property of an orbital's radial wavefunction (see [`RadialPlot`]).
///
/// The property will be evaluated at `num_points` points evenly spaced between the origin
/// and the maximum extent of the orbital, which is automatically estimated.
///
/// The result is returned as a 2-tuple of `Vec`s, the first containing the radial points,
/// and the second containing the values associated with the radial points.
pub fn sample_radial(qn: &Qn, variant: RadialPlot, num_points: usize) -> (Vec<f32>, Vec<f32>) {
    let evaluator = match variant {
        RadialPlot::Wavefunction => Radial::evaluate_on_line_segment,
        RadialPlot::ProbabilityDistribution => {
            RadialProbabilityDistribution::evaluate_on_line_segment
        }
    };
    let (xs, _, _, vals) = ComponentForm::from(evaluator(
        &qn.into(),
        Vec3::ZERO..=(Vec3::I * Real::bound(qn)), // We use the x-axis for simplicity; this function is radially symmetric.
        num_points,
    ))
    .into_components();

    (xs, vals)
}

/// Type that evaluates the probability density of an [`Orbital`].
///
/// Example:
///
/// ```
/// use approx::assert_relative_eq;
/// use evanescence_core::geometry::Point;
/// use evanescence_core::numerics::Evaluate;
/// use evanescence_core::orbital::{ProbabilityDensity, Qn, Real};
///
/// let qn = Qn::new(3, 2, 1).unwrap();
///
/// assert_relative_eq!(
///     2.446E-4,
///     ProbabilityDensity::<Real>::evaluate(&qn, &Point::new(6.0, -0.3, 8.5))
/// );
/// ```
pub struct ProbabilityDensity<O> {
    marker: PhantomData<O>,
}

impl<O: Orbital> Evaluate for ProbabilityDensity<O> {
    type Output = f32;
    type Parameters = O::Parameters;

    #[inline]
    fn evaluate(params: &Self::Parameters, point: &Point) -> Self::Output {
        O::probability_density_of(O::evaluate(params, point))
    }
}

impl<O: Orbital> EvaluateBounded for ProbabilityDensity<O> {
    #[inline]
    fn bound(params: &Self::Parameters) -> f32 {
        O::bound(params)
    }
}

#[cfg(test)]
mod tests {
    use super::{Qn, RadialPlot};
    use crate::numerics;

    #[test]
    fn test_radial_probability_density_unity() {
        Qn::enumerate_up_to_n(8)
            .unwrap()
            .map(|qn| super::sample_radial(&qn, RadialPlot::ProbabilityDistribution, 1_000))
            .for_each(|(xs, ys)| {
                approx::assert_abs_diff_eq!(
                    1.0,
                    numerics::trapezoidal_integrate(&xs, &ys),
                    epsilon = 0.005
                );
            });
    }
}
