use std::fmt;

use itertools::Itertools;

use super::{Orbital, Qn, Real1};
use crate::geometry::{Point, Vec3};
use crate::numerics::{Evaluate, EvaluateBounded};
use crate::orbital::hybrid::Component;

#[derive(Clone, Copy, PartialEq, Eq, Debug, strum::Display)]
pub enum Geometry {
    #[strum(serialize = "σ")]
    Sigma,
    #[strum(serialize = "π")]
    Pi,
}

macro_rules! real_z {
    (name: $name:ident; $($elem:ident : $z:literal),+ $(,)?) => {
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, strum::Display)]
        pub enum Element {
            $($elem),+
        }

        pub struct $name;

        impl Evaluate for $name {
            type Output = f32;
            type Parameters = (Qn, Element);

            #[inline]
            fn evaluate((qn, elem): &Self::Parameters, point: &Point) -> Self::Output {
                match elem {
                    $(Element::$elem => $crate::orbital::Real::<$z>::evaluate(qn, point)),+
                }
            }
        }

        impl EvaluateBounded for $name {
            fn bound((qn, elem): &Self::Parameters) -> f32 {
                match elem {
                    $(Element::$elem => $crate::orbital::Real::<$z>::bound(qn)),+
                }
            }
        }

        impl Orbital for $name {
            fn probability_density_of(value: Self::Output) -> f32 {
                value * value
            }

            fn name((qn, elem): &Self::Parameters) -> String {
                format!("{} ({elem})", Real1::name(qn))
            }
        }
    }
}

real_z! {
    name: RealZ;
    H: 1,
    He: 2,
    Li: 3,
    Be: 4,
    B: 5,
    C: 6,
    N: 7,
    O: 8,
    F: 9,
    Ne: 10,
}

#[derive(Clone, PartialEq)]
pub struct Lcao {
    pub bonding: bool,
    pub geometry: Geometry,
    pub combination: Vec<LcaoAtom>,
}

#[derive(Clone, PartialEq)]
pub struct LcaoAtom {
    pub elem: Element,
    pub offset: Vec3,
    pub orbitals: Vec<Component>,
}

pub struct LcaoAo {
    pub qn: Qn,
    pub elem: Element,
    pub offset: Vec3,
    pub weight: f32,
}

impl Lcao {
    pub fn iter(&self) -> impl Iterator<Item = LcaoAo> + '_ {
        self.combination.iter().flat_map(
            |&LcaoAtom {
                 elem,
                 offset,
                 ref orbitals,
             }| {
                orbitals
                    .iter()
                    .map(move |&Component { qn, weight }| LcaoAo {
                        qn,
                        elem,
                        offset,
                        weight,
                    })
            },
        )
    }
}

impl fmt::Display for Lcao {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(
            &Molecular::name(self)
                .replace("<sub>", " ")
                .replace("</sub>", ""),
        )
    }
}

pub struct Molecular;

impl Evaluate for Molecular {
    type Output = f32;
    type Parameters = Lcao;

    fn evaluate(params: &Self::Parameters, point: &Point) -> Self::Output {
        let pt_vec = Vec3::from(point);
        params
            .iter()
            .map(
                |LcaoAo {
                     qn,
                     elem,
                     offset,
                     weight,
                 }| {
                    weight * RealZ::evaluate(&(qn, elem), &Point::from(pt_vec - offset))
                },
            )
            .sum()
    }
}

impl EvaluateBounded for Molecular {
    fn bound(params: &Self::Parameters) -> f32 {
        let mut max = 0_f32;
        for LcaoAo {
            qn, elem, offset, ..
        } in params.iter()
        {
            let bound = RealZ::bound(&(qn, elem));
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
        let same_element = params.combination.iter().map(|elem| elem.elem).all_equal();
        let mut character = Self::unique_atomic_orbitals(params)
            .map(|qn_element| {
                if same_element {
                    Real1::name(&qn_element.0)
                } else {
                    RealZ::name(&qn_element)
                }
            })
            .join(" + ");
        if same_element {
            character.push_str(&format!(" ({})", params.combination[0].elem));
        }
        format!(
            "{name_short} [{character}]",
            name_short = Self::orbital_type(params),
        )
    }
}

impl Molecular {
    pub fn unique_atomic_orbitals(params: &Lcao) -> impl Iterator<Item = (Qn, Element)> + '_ {
        params.iter().map(|ao| (ao.qn, ao.elem)).unique()
    }

    pub fn orbital_type(params: &Lcao) -> String {
        format!(
            "{geometry}{bonding}",
            geometry = params.geometry,
            bonding = if params.bonding { "" } else { "*" },
        )
    }
}
