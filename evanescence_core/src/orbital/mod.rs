//! Implementations of real and complex hydrogen-atom orbitals.
//!
//! Access to radial and angular components, as well as related functions (ex. radial probability
//! and probability density) are available through the [`wavefunctions`] module.
//!
//! Types for working with and validating quantum numbers are also provided.

use getset::CopyGetters;
use num_complex::Complex32;

use crate::geometry::{ComponentForm, GridValues, Plane, Point, Vec3};
use crate::numerics::Evaluate;
use wavefunctions::{
    Radial, RadialProbability, RadialProbabilityDistribution, RealSphericalHarmonic,
    SphericalHarmonic,
};

pub mod wavefunctions;

/// Type representing the quantum numbers `n`, `l`, and `m`.
///
/// # Safety
/// `QuantumNumbers` must satisfy that `n > l` and `l >= |m|`.
#[derive(Clone, Copy, Debug, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct QuantumNumbers {
    /// The principal quantum number.
    n: u32,
    /// The azimuthal quantum number.
    l: u32,
    /// The magnetic quantum number.
    m: i32,
}

impl QuantumNumbers {
    /// Create a new `QuantumNumbers`, verifying that the passed values are valid. Returns `None`
    /// if that is not the case.
    pub const fn new(n: u32, l: u32, m: i32) -> Option<Self> {
        if n > l && l >= m.abs() as u32 {
            Some(Self { n, l, m })
        } else {
            None
        }
    }

    /// List all possible quantum number sets with `n` less than or equal to the value passed.
    pub fn enumerate_up_to_n(n: u32) -> impl Iterator<Item = Self> {
        // n = 1, 2, 3, ...
        (1..=n).flat_map(|n| {
            // l = 0, 1, ..., n - 1
            (0..n).flat_map(move |l| {
                // m = -l, -l + 1, ..., 0, ..., l -1, l
                (-(l as i32)..=(l as i32)).map(move |m| Self::new(n, l, m).unwrap())
            })
        })
    }

    /// List all possible quantum number sets with both `n` and `l` less than or equal to
    /// the values passed.
    pub fn enumerate_up_to_n_l(n: u32, l: u32) -> impl Iterator<Item = Self> {
        // n = 1, 2, 3, ...
        (1..=n).flat_map(move |n| {
            // l = 0, 1, ..., minimum of n - 1 and the limit passed in the parameter
            (0..=(n - 1).min(l)).flat_map(move |l| {
                // m = -l, -l + 1, ..., 0, ..., l -1, l
                (-(l as i32)..=(l as i32)).map(move |m| Self::new(n, l, m).unwrap())
            })
        })
    }
}

impl std::fmt::Display for QuantumNumbers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{},{}", self.n, self.l, self.m)
    }
}

/// Type representing the quantum numbers `n` and `l`.
///
/// # Safety
/// `NL` must satisfy that `n > l`.
#[derive(Clone, Copy, Debug, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct NL {
    /// The principal quantum number.
    n: u32,
    /// The azimuthal quantum number.
    l: u32,
}

impl NL {
    /// Create a new `NL`, verifying that the passed values are valid. Returns `None`
    /// if that is not the case.
    pub const fn new(n: u32, l: u32) -> Option<Self> {
        if n > l {
            Some(Self { n, l })
        } else {
            None
        }
    }
}

impl From<QuantumNumbers> for NL {
    fn from(QuantumNumbers { n, l, m: _ }: QuantumNumbers) -> Self {
        Self { n, l }
    }
}

/// Type representing the quantum numbers `l` and `m`.
///
/// # Safety
/// `LM` must satisfy that `l >= |m|`.
#[derive(Clone, Copy, Debug, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct LM {
    /// The azimuthal quantum number.
    l: u32,
    /// The magnetic quantum number.
    m: i32,
}

impl LM {
    /// Create a new `LM`, verifying that the passed values are valid. Returns `None`
    /// if that is not the case.
    pub const fn new(l: u32, m: i32) -> Option<Self> {
        if l >= m.abs() as u32 {
            Some(Self { l, m })
        } else {
            None
        }
    }
}

impl From<QuantumNumbers> for LM {
    fn from(QuantumNumbers { n: _, l, m }: QuantumNumbers) -> Self {
        Self { l, m }
    }
}

/// A radially symmetrical property associated with an orbital.
#[non_exhaustive]
pub enum RadialPlot {
    Wavefunction,
    Probability,
    ProbabilityDistribution,
}

/// Trait representing a hydrogenic orbital.
pub trait Orbital: Evaluate<Parameters = QuantumNumbers> {
    /// An empirically derived heuristic for estimating the radius of a specific orbital
    /// (in the sense that the vast majority of probability density is confined within a sphere
    /// of that radius). See the attached Mathematica notebook `radial_wavefunction.nb`
    /// for plots.
    #[inline]
    fn estimate_radius(qn: QuantumNumbers) -> f32 {
        let n = qn.n() as f32;
        n * (2.5 * n - 0.625 * qn.l() as f32 + 3.0)
    }

    /// Compute a plot of a property of an orbital's radial wavefunction (see [`RadialPlot`]).
    ///
    /// The property will be evaluated at `num_points` points evenly spaced between the origin
    /// and the maximum extent of the orbital, which is automatically estimated.
    ///
    /// The result is returned as a 2-tuple of `Vec`s, the first containing the radial points,
    /// and the second containing the values associated with the radial points.
    fn sample_radial(
        qn: QuantumNumbers,
        variant: RadialPlot,
        num_points: usize,
    ) -> (Vec<f32>, Vec<f32>) {
        let evaluator = match variant {
            RadialPlot::Wavefunction => Radial::evaluate_on_line_segment,
            RadialPlot::Probability => RadialProbability::evaluate_on_line_segment,
            RadialPlot::ProbabilityDistribution => {
                RadialProbabilityDistribution::evaluate_on_line_segment
            }
        };
        let (xs, _, _, vals) = ComponentForm::from(evaluator(
            qn.into(),
            Vec3::ZERO,
            Vec3::I * Self::estimate_radius(qn), // We use the x-axis for simplicity; this function is radially symmetrical.
            num_points,
        ))
        .into_components();
        (xs, vals)
    }

    /// Compute a plot of the cross section of an orbital along a given `plane`.
    ///
    /// `num_points` points will be evaluated in a grid centered at the origin and covering
    /// the extent of the orbital, which is automatically estimated.
    ///
    /// For more information, see the documentation on [`GridValues`].
    fn sample_cross_section(
        qn: QuantumNumbers,
        plane: Plane,
        num_points: usize,
    ) -> GridValues<Self::Output> {
        Self::evaluate_on_plane(qn, plane, Self::estimate_radius(qn), num_points)
    }

    /// Compute a plot of an orbital in a cube centered at the origin. `num_points` are sampled
    /// in each dimension, producing an evenly-spaced lattice of values the size of the orbital's
    /// extent.
    ///
    /// For more information, see [`Evaluate::evaluate_in_region`].
    fn sample_region(qn: QuantumNumbers, num_points: usize) -> ComponentForm<Self::Output> {
        Self::evaluate_in_region(qn, Self::estimate_radius(qn), num_points).into()
    }
}

/// Implementation of the real hydrogenic orbitals.
pub struct Real;

impl Evaluate for Real {
    type Output = f32;
    type Parameters = QuantumNumbers;

    #[inline]
    fn evaluate(qn: QuantumNumbers, point: &Point) -> f32 {
        Radial::evaluate(qn.into(), point) * RealSphericalHarmonic::evaluate(qn.into(), point)
    }
}

impl Orbital for Real {}

/// Implementation of the complex hydrogenic orbitals.
pub struct Complex;

impl Evaluate for Complex {
    type Output = Complex32;
    type Parameters = QuantumNumbers;

    #[inline]
    fn evaluate(qn: QuantumNumbers, point: &Point) -> Complex32 {
        Radial::evaluate(qn.into(), point) * SphericalHarmonic::evaluate(qn.into(), point)
    }
}

impl Orbital for Complex {}
