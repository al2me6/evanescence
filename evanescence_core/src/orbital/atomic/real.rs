use std::cmp::Ordering;
use std::f32::consts::PI;

use super::Radial;
use crate::geometry::Point;
use crate::numerics::special::orthogonal_polynomials::renormalized_associated_legendre;
use crate::numerics::spherical_harmonics::RealSphericalHarmonic;
use crate::numerics::{self, Evaluate, EvaluateBounded};
use crate::orbital::quantum_numbers::{Lm, Qn};
use crate::orbital::Orbital;

/// Implementation of the real hydrogenic orbitals.
pub struct Real {
    pub(in crate::orbital) qn: Qn,
    radial: Radial,
    sph: RealSphericalHarmonic,
}

impl Real {
    pub fn new(qn: Qn) -> Self {
        Self {
            qn,
            radial: Radial::new(qn.into()),
            sph: RealSphericalHarmonic::new(qn.into()),
        }
    }

    pub fn name_qn(qn: Qn) -> String {
        if let (Some(subshell), Some(linear_combination)) = (
            super::subshell_name(qn.l()),
            RealSphericalHarmonic::expression(qn.into()),
        ) {
            format!("{}{subshell}<sub>{linear_combination}</sub>", qn.n())
        } else {
            super::basic_name(qn)
        }
    }
}

impl Evaluate for Real {
    type Output = f32;

    #[inline]
    fn evaluate(&self, point: &Point) -> f32 {
        self.radial.evaluate(point) * self.sph.evaluate(point)
    }
}

impl EvaluateBounded for Real {
    /// Return the radius of the sphere within which the probability of the electron being found is
    /// [`Self::PROBABILITY_WITHIN_BOUND`].
    #[inline]
    fn bound(&self) -> f32 {
        super::bound(self.qn)
    }
}

impl Orbital for Real {
    #[inline]
    fn probability_density_of(&self, value: f32) -> f32 {
        value * value
    }

    /// Try to give the orbital's conventional name (ex. `4d_{z^2}`) before falling back to giving
    /// the quantum numbers only (ex. `ψ_{420}`).
    fn name(&self) -> String {
        Self::name_qn(self.qn)
    }
}

impl Real {
    /// Give the number of radial nodes in an orbital.
    pub fn num_radial_nodes(qn: Qn) -> u32 {
        qn.n() - qn.l() - 1
    }

    /// Give the number of angular nodes in an orbital.
    pub fn num_angular_nodes(qn: Qn) -> u32 {
        qn.l()
    }

    /// Give the number of conical angular nodes in an orbital.
    pub fn num_conical_nodes(lm: Lm) -> u32 {
        lm.l() - lm.m().unsigned_abs()
    }

    /// Give the number of planar angular nodes in an orbital.
    pub fn num_planar_nodes(lm: Lm) -> u32 {
        lm.m().unsigned_abs()
    }

    /// Give the `r` values of all radial nodes of a given `n` and `l` pair.
    #[allow(clippy::missing_panics_doc)] // The `assert_eq` is an internal sanity check.
    pub fn radial_node_positions(qn: Qn) -> Vec<f32> {
        let radial = Radial::new(qn.into());
        let roots = numerics::find_roots_in_interval(0.05..=super::bound(qn), 100, |r| {
            radial.evaluate_r(r)
        })
        .collect::<Vec<_>>();
        assert_eq!(
            roots.len(),
            Self::num_radial_nodes(qn) as usize,
            "not all radial nodes were found"
        );
        roots
    }

    /// Give the theta angles of all conical nodes of a given `l` and `m` pair.
    #[allow(clippy::missing_panics_doc)] // The `assert_eq` is an internal sanity check.
    pub fn conical_node_angles(lm: Lm) -> Vec<f32> {
        let roots = numerics::find_roots_in_interval(0.0..=PI, 90, |theta| {
            renormalized_associated_legendre((lm.l(), lm.m().unsigned_abs()), theta.cos())
        })
        .collect::<Vec<_>>();
        assert_eq!(
            roots.len(),
            Self::num_conical_nodes(lm) as usize,
            "not all conical node angles were found"
        );
        roots
    }

    /// Give the phi angles of all planar nodes of a given `l` and `m` pair.
    #[allow(clippy::missing_panics_doc)] // The `assert_eq` is an internal sanity check.
    pub fn planar_node_angles(lm: Lm) -> Vec<f32> {
        let m = lm.m();
        let m_abs = m.unsigned_abs();
        // Offset the search interval clockwise by ~0.9 degrees to ensure that planes at 0 degs are
        // correctly sampled, and that they aren't double-counted at 180 degs.
        let roots =
            numerics::find_roots_in_interval((0.0 - 0.015)..=(PI - 0.015), 90, |phi| {
                match m.cmp(&0) {
                    Ordering::Greater => (m as f32 * phi).cos(),
                    Ordering::Equal => 1.0,
                    Ordering::Less => (m_abs as f32 * phi).sin(),
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(
            roots.len(),
            Self::num_planar_nodes(lm) as usize,
            "not all planar node angles were found"
        );
        roots
    }
}

/// See attached Mathematica notebooks for the computation of test values.
#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use rayon::prelude::*;

    use super::Real;
    use crate::geometry::Point;
    use crate::numerics::{Evaluate, EvaluateBounded};
    use crate::orbital::atomic::PROBABILITY_WITHIN_BOUND;
    use crate::orbital::quantum_numbers::{Lm, Qn};
    use crate::orbital::ProbabilityDensity;

    #[test]
    fn real_radial_nodes() {
        Qn::enumerate_up_to_n(10)
            .unwrap()
            .filter(|qn| qn.m() == 0)
            .map(|qn| (qn, Real::radial_node_positions(qn)))
            .for_each(|(qn, pts)| {
                println!("[{qn}]: {}", pts.iter().join(", "));
            });
    }

    #[test]
    fn real_conical_node_angles() {
        Qn::enumerate_up_to_n(10)
            .unwrap()
            .map(Lm::from)
            .unique()
            .map(|lm| (lm, Real::conical_node_angles(lm)))
            .for_each(|(lm, pts)| {
                println!(
                    "[{lm:?}]: {}",
                    pts.iter().map(|theta| theta.to_degrees()).join(", ")
                );
            });
    }

    #[test]
    fn real_planar_node_angles() {
        Qn::enumerate_up_to_n(10)
            .unwrap()
            .map(Lm::from)
            .unique()
            .map(|lm| (lm, Real::planar_node_angles(lm)))
            .for_each(|(lm, pts)| {
                println!(
                    "[{lm:?}]: {}",
                    pts.iter().map(|phi| phi.to_degrees()).join(", ")
                );
            });
    }

    #[test]
    fn real_probability_unity() {
        let qns = Qn::enumerate_up_to_n(7).unwrap().step_by(3).collect_vec();
        qns.into_par_iter().for_each(|qn| {
            let psi_sq = ProbabilityDensity::new(Real::new(qn));
            let bound = psi_sq.bound();
            let prob = integrate_simpson_multiple!(
                psi_sq.evaluate(&Point::new(x, y, z)),
                step: bound / 40_f32,
                x: (-bound, bound),
                y: (-bound, bound),
                z: (-bound, bound),
            );
            println!("{qn}: {prob}");
            assert!(PROBABILITY_WITHIN_BOUND < prob && prob < 1_f32 + 5E-5);
        });
    }
}
