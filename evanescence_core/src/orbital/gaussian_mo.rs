use std::str::FromStr;

use getset::{CopyGetters, Getters};
use na::{vector, Point3, Vector3};

use super::Orbital;
use crate::geometry::region::{BoundingRegion, RectangularPrism};
use crate::numerics::consts::ANGSTROM_TO_BOHR;
use crate::numerics::monte_carlo::accept_reject::AcceptRejectParameters;
use crate::numerics::statistics::AsDistribution;
use crate::numerics::{trilinear_interpolate, Function};
use crate::utils::sup_sub_string::SupSubString;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, PartialEq, Debug)]
pub struct Atom {
    pub atomic_number: u32,
    pub charge: f32,
    pub position: Vector3<f32>,
}

/// MO data in Gaussian's `cube` format.
///
/// All lengths are converted to Bohr and only steps in the standard basis are supported.
///
/// Documentation:
/// * <https://gaussian.com/cubegen/>
/// * <http://paulbourke.net/dataformats/cube/>
#[derive(Clone, PartialEq, Debug, Getters, CopyGetters)]
pub struct MoCube {
    #[getset(get = "pub")]
    comment_1: String,
    #[getset(get = "pub")]
    comment_2: String,
    #[getset(get = "pub")]
    atoms: Vec<Atom>,
    #[getset(get = "pub")]
    point_count: Vector3<usize>,
    #[getset(get = "pub")]
    step_size: Vector3<f32>,
    /// Cached value - number of index increments in `values` for each step in x, y, and z.
    indices_per_step: Vector3<usize>,
    #[getset(get_copy = "pub")]
    mo_number: u32,
    values: Vec<f32>,
    /// Cached value - bounds of the cube region.
    volume: RectangularPrism,
}

impl MoCube {
    #[inline]
    fn get_at_steps(&self, steps: Vector3<usize>) -> Option<f32> {
        let idx = steps.dot(&self.indices_per_step);
        self.values.get(idx).copied()
    }

    // TODO: use a better interpolation method?
    pub fn interpolate_value(&self, pt: Vector3<f32>) -> f32 {
        let clamped_to_volume = self.volume.clamp(pt);

        let offset_from_bottom_left = clamped_to_volume - self.volume.bottom_left;

        let step_bottom_left = offset_from_bottom_left.component_div(&self.step_size);
        let normalized_cube_offset = step_bottom_left.map(f32::fract);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let step_bottom_left = step_bottom_left.map(|x_i| x_i as usize);
        let step_top_right = step_bottom_left + Vector3::from_element(1);

        let cube = [step_bottom_left, step_top_right];
        let mut cube_values = [0.; 8];
        let mut dest = cube_values.iter_mut();
        for x in 0..2 {
            for y in 0..2 {
                for z in 0..2 {
                    *dest.next().unwrap() = self
                        .get_at_steps(vector![cube[x].x, cube[y].y, cube[z].z])
                        .unwrap();
                }
            }
        }
        trilinear_interpolate(cube_values, normalized_cube_offset)
    }
}

#[derive(Clone, PartialEq, Eq, Debug, thiserror::Error)]
pub enum ParseCubeError {
    #[error("unexpectedly reached end-of-file")]
    UnexpectedEof,
    #[error("unexpectedly reached end-of-line")]
    UnexpectedEol,
    #[error(transparent)]
    InvalidInt(#[from] std::num::ParseIntError),
    #[error(transparent)]
    InvalidFloat(#[from] std::num::ParseFloatError),
    #[error("only steps aligned with the standard basis are supported")]
    NonStandardStepBasis,
    #[error("contained data is not a molecular orbital")]
    NotMo,
    #[error("too many values ({0}) per point; expected 1")]
    TooManyValuesPerPoint(u32),
    #[error("can only parse 1 MO; got {0}")]
    TooManyMos(u32),
    #[error("expected {0} values, got {1}")]
    IncorrectValueCount(usize, usize),
}

type Result<T, E = ParseCubeError> = std::result::Result<T, E>;

fn read_next<'a>(src: &mut impl Iterator<Item = &'a str>) -> Result<&'a str> {
    src.next().ok_or(ParseCubeError::UnexpectedEof)
}

fn read_next_split<'a>(
    src: &mut impl Iterator<Item = &'a str>,
) -> Result<impl Iterator<Item = &'a str>> {
    read_next(src).map(str::split_ascii_whitespace)
}

fn parse_next_vector3<'a>(src: &mut impl Iterator<Item = &'a str>) -> Result<Vector3<f32>> {
    let mut res = [0.; 3];
    for coord in &mut res {
        *coord = src.next().ok_or(ParseCubeError::UnexpectedEof)?.parse()?;
    }
    Ok(res.into())
}

fn parse_next<'a, T: FromStr>(src: &mut impl Iterator<Item = &'a str>) -> Result<T>
where
    ParseCubeError: From<T::Err>,
{
    Ok(src.next().ok_or(ParseCubeError::UnexpectedEol)?.parse()?)
}

impl FromStr for MoCube {
    type Err = ParseCubeError;

    fn from_str(s: &str) -> Result<Self> {
        let lines = &mut s.lines();

        let comment_1 = read_next(lines)?.trim().to_owned();
        let comment_2 = read_next(lines)?.trim().to_owned();

        let mut line = read_next_split(lines)?;
        let atom_count = parse_next::<i32>(&mut line)?;
        if atom_count >= 0 {
            Err(ParseCubeError::NotMo)?;
        }
        let atom_count = atom_count.unsigned_abs();
        let mut initial_point = parse_next_vector3(&mut line)?;
        let values_per_point = parse_next::<u32>(&mut line)?;
        if values_per_point != 1 {
            Err(ParseCubeError::TooManyValuesPerPoint(values_per_point))?;
        }

        let mut point_count = Vector3::<usize>::zeros();
        let mut step_size = Vector3::<f32>::zeros();
        for i in 0..3 {
            line = read_next_split(lines)?;
            let point_count_i = parse_next::<isize>(&mut line)?;
            point_count[i] = point_count_i.unsigned_abs();

            let mut step_vec_i = parse_next_vector3(&mut line)?;
            // Positive - unit is Angstroms.
            if point_count_i >= 0 {
                step_vec_i *= ANGSTROM_TO_BOHR;
                initial_point[i] *= ANGSTROM_TO_BOHR;
            }

            let e_i = Vector3::ith_axis(i);
            if !approx::relative_eq!(step_vec_i.normalize(), e_i, max_relative = 1E-5) {
                Err(ParseCubeError::NonStandardStepBasis)?;
            }

            step_size[i] = step_vec_i[i];
        }

        let mut atoms = Vec::with_capacity(atom_count as _);
        for _ in 0..atom_count {
            line = read_next_split(lines)?;
            let atomic_number = parse_next::<u32>(&mut line)?;
            let charge = parse_next::<f32>(&mut line)?;
            let position = parse_next_vector3(&mut line)?;
            atoms.push(Atom {
                atomic_number,
                charge,
                position,
            });
        }

        line = read_next_split(lines)?;
        let mo_count = parse_next::<u32>(&mut line)?;
        if mo_count > 1 {
            Err(ParseCubeError::TooManyMos(mo_count))?;
        }
        let mo_number = parse_next::<u32>(&mut line)?;

        let expected_value_count = point_count.product() as usize;
        let mut values = Vec::<f32>::with_capacity(expected_value_count);
        while let Ok(line) = read_next_split(lines) {
            for float in line {
                values.push(float.parse()?);
            }
        }
        if expected_value_count != values.len() {
            Err(ParseCubeError::IncorrectValueCount(
                expected_value_count,
                values.len(),
            ))?;
        }

        Ok(MoCube {
            comment_1,
            comment_2,
            atoms,
            point_count,
            step_size,
            indices_per_step: vector![point_count.yz().product(), point_count.z, 1],
            mo_number,
            values,
            volume: RectangularPrism {
                bottom_left: initial_point,
                // Beware fencepost.
                side_lengths: step_size
                    .component_mul(&(point_count - Vector3::from_element(1)).cast()),
            },
        })
    }
}

pub struct GaussianMo {
    cube: MoCube,
}

impl GaussianMo {
    pub fn new(cube: MoCube) -> Self {
        Self { cube }
    }
}

impl Function<3> for GaussianMo {
    type Output = f32;

    fn evaluate(&self, point: &Point3<f32>) -> Self::Output {
        self.cube.interpolate_value(point.coords)
    }
}

impl BoundingRegion<3> for GaussianMo {
    type Geometry = RectangularPrism;

    fn bounding_region(&self) -> Self::Geometry {
        self.cube.volume.clone()
    }
}

impl AsDistribution<3> for GaussianMo {
    fn probability_density_of(&self, value: Self::Output) -> f32 {
        value * value
    }
}

impl Orbital for GaussianMo {
    fn name(&self) -> SupSubString {
        sup_sub_string![nrm(self.cube.comment_1().clone())]
    }
}

impl AcceptRejectParameters<3> for GaussianMo {
    fn maximum(&self) -> f32 {
        self.cube
            .values
            .iter()
            .copied()
            .map(f32::abs)
            .reduce(f32::max)
            .unwrap()
    }
}
