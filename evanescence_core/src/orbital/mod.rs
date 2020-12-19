use getset::CopyGetters;
use num_complex::Complex64;

use crate::geometry::Point;

pub mod wavefunction;
pub use wavefunction::Wavefunction;
pub mod monte_carlo;

use wavefunction::{RadialWavefunction, RealSphericalHarmonic, SphericalHarmonic};

#[derive(Clone, Copy, Debug, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct QuantumNumbers {
    n: u32,
    l: u32,
    m: i32,
}

impl QuantumNumbers {
    pub const fn new(n: u32, l: u32, m: i32) -> Option<Self> {
        if n > l && l >= m.abs() as u32 {
            Some(Self { n, l, m })
        } else {
            None
        }
    }

    pub fn enumerate_all_up_to_n(n: u32) -> impl Iterator<Item = Self> {
        // n = 1, 2, 3, ...
        (1..=n).flat_map(|n| {
            // l = 0, 1, ..., n - 1
            (0..n).flat_map(move |l| {
                // m = -l, -l + 1, ..., 0, ..., l -1, l
                (-(l as i32)..=(l as i32)).map(move |m| Self { n, l, m })
            })
        })
    }

    pub fn enumerate_all_up_to_n_l(n: u32, l: u32) -> impl Iterator<Item = Self> {
        // n = 1, 2, 3, ...
        (1..=n).flat_map(move |n| {
            // l = 0, 1, ..., minimum of n - 1 and the limit passed in the parameter
            (0..n.min(l)).flat_map(move |l| {
                // m = -l, -l + 1, ..., 0, ..., l -1, l
                (-(l as i32)..=(l as i32)).map(move |m| Self { n, l, m })
            })
        })
    }
}

impl std::fmt::Display for QuantumNumbers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{},{}", self.n, self.l, self.m)
    }
}

#[derive(Clone, Copy, Debug, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct NL {
    n: u32,
    l: u32,
}

impl NL {
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

#[derive(Clone, Copy, Debug, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct LM {
    l: u32,
    m: i32,
}

impl LM {
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

pub struct RealOrbital;

impl Wavefunction for RealOrbital {
    type Output = f64;
    type Parameters = QuantumNumbers;

    #[inline]
    fn evaluate(qn: QuantumNumbers, point: &Point) -> f64 {
        RadialWavefunction::evaluate(qn.into(), point)
            * RealSphericalHarmonic::evaluate(qn.into(), point)
    }
}

pub struct ComplexOrbital;

impl Wavefunction for ComplexOrbital {
    type Output = Complex64;
    type Parameters = QuantumNumbers;

    #[inline]
    fn evaluate(qn: QuantumNumbers, point: &Point) -> Complex64 {
        RadialWavefunction::evaluate(qn.into(), point)
            * SphericalHarmonic::evaluate(qn.into(), point)
    }
}
