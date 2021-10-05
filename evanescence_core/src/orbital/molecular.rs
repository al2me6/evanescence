use std::fmt;

use super::{Orbital, Qn, Real};
use crate::geometry::{Point, Vec3};
use crate::numerics::{Evaluate, EvaluateBounded};

#[derive(Clone, PartialEq, Debug)]
pub struct OffsetQnWeight {
    pub qn: Qn,
    pub weight: f32,
    pub offset: Vec3,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Lcao {
    pub name: String,
    pub weights: Vec<OffsetQnWeight>,
}

impl fmt::Display for Lcao {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)
    }
}

pub struct Molecular;

impl Evaluate for Molecular {
    type Output = f32;
    type Parameters = Lcao;

    fn evaluate(params: &Self::Parameters, point: &Point) -> Self::Output {
        let pt_vec = Vec3::from(point);
        params
            .weights
            .iter()
            .map(|OffsetQnWeight { qn, weight, offset }| {
                weight * Real::evaluate(qn, &Point::from(pt_vec - offset))
            })
            .sum()
    }
}

impl EvaluateBounded for Molecular {
    fn bound(params: &Self::Parameters) -> f32 {
        let mut max = 0_f32;
        for OffsetQnWeight { qn, offset, .. } in &params.weights {
            let bound = Real::bound(qn);
            let max_offset = [offset.x, offset.y, offset.z]
                .into_iter()
                .map(f32::abs)
                .reduce(f32::max)
                .expect("bound is well defined");
            max = max.max(bound + max_offset);
        }
        max
    }
}

impl Orbital for Molecular {
    fn probability_density_of(value: Self::Output) -> f32 {
        value * value
    }

    fn name(params: &Self::Parameters) -> String {
        params.name.clone()
    }
}
