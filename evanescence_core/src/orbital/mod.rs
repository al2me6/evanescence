//! Implementations of real and complex hydrogen-atom orbitals.
//!
//! Access to radial and angular components, as well as related functions (ex. radial probability
//! and probability density) are available through the [`wavefunctions`] module.
//!
//! Types for quantum numbers are available through the [`quantum_numbers`] module.
pub mod quantum_numbers;
pub mod wavefunctions;

use num_complex::Complex32;

pub use quantum_numbers::Qn;

use crate::geometry::{ComponentForm, GridValues, Plane, Point, Vec3};
use crate::numerics::Evaluate;
use wavefunctions::{
    Radial, RadialProbabilityDensity, RadialProbabilityDistribution, RealSphericalHarmonic,
    SphericalHarmonic,
};

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

/// Trait representing a hydrogenic orbital.
pub trait Orbital: Evaluate<Parameters = Qn> {
    /// An empirically derived heuristic for estimating the radius of a specific orbital
    /// (in the sense that the vast majority of probability density is confined within a sphere
    /// of that radius). See the attached Mathematica notebook `radial_wavefunction.nb`
    /// for plots.
    #[inline]
    fn estimate_radius(qn: Qn) -> f32 {
        let n = qn.n() as f32;
        n * (2.5 * n - 0.625 * qn.l() as f32 + 3.0)
    }

    /// Compute a plot of the cross section of an orbital along a given `plane`.
    ///
    /// `num_points` points will be evaluated in a grid centered at the origin and covering
    /// the extent of the orbital, which is automatically estimated.
    ///
    /// For more information, see the documentation on [`GridValues`].
    fn sample_cross_section(qn: Qn, plane: Plane, num_points: usize) -> GridValues<Self::Output> {
        Self::evaluate_on_plane(qn, plane, Self::estimate_radius(qn), num_points)
    }

    /// Give the conventional name of an orbital.
    ///
    /// Superscripts are represented using Unicode superscript symbols and subscripts are
    /// represented with the HTML tag `<sub></sub>`.
    fn name(qn: Qn) -> String;
}

/// Implementation of the real hydrogenic orbitals.
pub struct Real;

impl Evaluate for Real {
    type Output = f32;
    type Parameters = Qn;

    #[inline]
    fn evaluate(qn: Qn, point: &Point) -> f32 {
        Radial::evaluate(qn.into(), point) * RealSphericalHarmonic::evaluate(qn.into(), point)
    }
}

impl Orbital for Real {
    /// Try to give the orbital's conventional name (ex. `4d_{z^2}`) before falling back to giving
    /// the quantum numbers only (ex. `ψ_{420}`).
    fn name(qn: Qn) -> String {
        if let (Some(subshell), Some(linear_combination)) = (
            subshell_name(qn.l()),
            RealSphericalHarmonic::expression(qn.into()),
        ) {
            format!("{}{}<sub>{}</sub>", qn.n(), subshell, linear_combination)
        } else {
            Complex::name(qn)
        }
    }
}

impl Real {
    /// Give the number of radial nodes in an orbital.
    pub fn num_radial_nodes(qn: Qn) -> u32 {
        qn.n() - qn.l() - 1
    }

    // Give the number of angular nodes in an orbital.
    pub fn num_angular_nodes(qn: Qn) -> u32 {
        qn.l()
    }
}

/// Implementation of the complex hydrogenic orbitals.
pub struct Complex;

impl Evaluate for Complex {
    type Output = Complex32;
    type Parameters = Qn;

    #[inline]
    fn evaluate(qn: Qn, point: &Point) -> Complex32 {
        Radial::evaluate(qn.into(), point) * SphericalHarmonic::evaluate(qn.into(), point)
    }
}

impl Orbital for Complex {
    /// Give the name of the wavefunction (ex. `ψ_{420}`).
    fn name(qn: Qn) -> String {
        format!("ψ<sub>{}{}{}</sub>", qn.n(), qn.l(), qn.m())
    }
}

/// A radially symmetrical property associated with an orbital.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RadialPlot {
    Wavefunction,
    ProbabilityDensity,
    ProbabilityDistribution,
}

/// Compute a plot of a property of an orbital's radial wavefunction (see [`RadialPlot`]).
///
/// The property will be evaluated at `num_points` points evenly spaced between the origin
/// and the maximum extent of the orbital, which is automatically estimated.
///
/// The result is returned as a 2-tuple of `Vec`s, the first containing the radial points,
/// and the second containing the values associated with the radial points.
pub fn sample_radial(qn: Qn, variant: RadialPlot, num_points: usize) -> (Vec<f32>, Vec<f32>) {
    let evaluator = match variant {
        RadialPlot::Wavefunction => Radial::evaluate_on_line_segment,
        RadialPlot::ProbabilityDensity => RadialProbabilityDensity::evaluate_on_line_segment,
        RadialPlot::ProbabilityDistribution => {
            RadialProbabilityDistribution::evaluate_on_line_segment
        }
    };
    let (xs, _, _, vals) = ComponentForm::from(evaluator(
        qn.into(),
        Vec3::ZERO..=(Vec3::I * Real::estimate_radius(qn)), // We use the x-axis for simplicity; this function is radially symmetric.
        num_points,
    ))
    .into_components();
    (xs, vals)
}

/// Compute a plot of a function related to an orbital in a cube centered at the origin.
/// `num_points` are sampled in each dimension, producing an evenly-spaced lattice of values the
/// size of the orbital's extent.
///
/// The optional value `extent_multiplier` is used to scale the extent plotted. Passing `None`
/// retains the original extent.
///
/// This function is intended to be used for plotting [radial](wavefunctions::Radial) and
/// [angular](wavefunctions::RealSphericalHarmonic) nodes.
///
/// For more information, see [`Evaluate::evaluate_in_region`].
pub fn sample_region_for<E>(
    qn: Qn,
    num_points: usize,
    extent_multiplier: Option<f32>,
) -> ComponentForm<E::Output>
where
    E: Evaluate,
    <E as Evaluate>::Parameters: From<Qn>,
{
    E::evaluate_in_region(
        qn.into(),
        Real::estimate_radius(qn) * extent_multiplier.unwrap_or(1.0),
        num_points,
    )
    .into()
}
