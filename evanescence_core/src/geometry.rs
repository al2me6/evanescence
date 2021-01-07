use std::f64::consts::PI;
use std::fmt::{self, Display};
use std::iter;
use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub};

use getset::{CopyGetters, Getters};
use strum::Display;

use crate::utils::new_rng;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn linspace(
        begin: Self,
        end: Self,
        num_points: usize,
    ) -> impl ExactSizeIterator<Item = Self> {
        let step = (end - begin) / (num_points as f64 - 1.0);
        let mut acc = begin;
        (0..num_points).map(move |_| {
            let next = acc;
            acc += step;
            next
        })
    }

    pub const ZERO: Vec3 = Self::new(0.0, 0.0, 0.0);
    pub const I: Vec3 = Self::new(1.0, 0.0, 0.0);
    pub const J: Vec3 = Self::new(0.0, 1.0, 0.0);
    pub const K: Vec3 = Self::new(0.0, 0.0, 1.0);
}

impl Add for Vec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl Mul<f64> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl Div<f64> for Vec3 {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl Display for Vec3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:+.4}i{:+.4}j{:+.4}k", self.x, self.y, self.z)
    }
}

impl From<Vec3> for Point {
    fn from(point: Vec3) -> Self {
        Self::new(point.x, point.y, point.z)
    }
}

/// A point in 3D space.
///
/// Note that we use the physics convention of (r, theta, phi): theta is the inclination
/// and phi is the azimuth.
///
/// # Safety
/// The spherical elements must be kept in sync with Cartesian elements. For this reason,
/// direct (i.e., mutable) access to struct members is not allowed.
#[derive(Clone, Copy, Debug, PartialEq, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Point {
    /// Cartesian x.
    x: f64,
    /// Cartesian y.
    y: f64,
    /// Cartesian z.
    z: f64,
    /// Spherical radius.
    r: f64,
    /// Cosine of spherical longitude.
    cos_theta: f64,
    /// Spherical azimuth.
    phi: f64,
}

impl Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:.4}, {:.4}, {:.4})", self.x, self.y, self.z)
    }
}

impl Point {
    /// Construct a new Point at Cartesian position (x, y, z).
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        let r = (x * x + y * y + z * z).sqrt();
        Self {
            x,
            y,
            z,
            r,
            cos_theta: z / r,
            phi: {
                let atan2 = y.atan2(x);
                if atan2.is_sign_positive() {
                    atan2
                } else {
                    2.0 * PI + atan2
                }
            },
        }
    }

    /// Construct a new Point at spherical position (r, theta, phi).
    pub fn new_spherical(r: f64, theta: f64, phi: f64) -> Self {
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();
        Self {
            x: r * sin_theta * phi.cos(),
            y: r * sin_theta * phi.sin(),
            z: r * cos_theta,
            r,
            cos_theta,
            phi,
        }
    }

    /// A point representing the origin.
    pub const ORIGIN: Point = Point {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        r: 0.0,
        cos_theta: 1.0,
        phi: 0.0,
    };

    /// The origin, but offset in z by [`f64::EPSILON`] for when division-by-zero needs to be avoided.
    pub const ORIGIN_EPSILON: Point = Point {
        x: 0.0,
        y: 0.0,
        z: f64::EPSILON,
        r: f64::EPSILON,
        cos_theta: 1.0,
        phi: 0.0,
    };

    /// Produce random points uniformly distributed within a ball of the given radius.
    ///
    /// Reference: <http://extremelearning.com.au/how-to-generate-uniformly-random-points-on-n-spheres-and-n-balls/>,
    /// specifically method 16.
    pub fn sample_from_ball_iter(radius: f64) -> impl Iterator<Item = Self> {
        let mut rng = new_rng();
        iter::repeat_with(move || {
            // For an explanation of taking the cube root of the random value, see
            // https://stackoverflow.com/a/50746409.
            let r /* [0, radius] */ = rng.rand_float().cbrt() * radius;
            let cos_theta /* [-1, 1] */ = rng.rand_float() * 2.0 - 1.0;
            // Pythagorean identity: sin^2(x) + cos^2(x) = 1.
            let sin_theta = (1.0 - cos_theta.powi(2)).sqrt();
            let phi /* [0, 2pi) */ = rng.rand_float() * 2.0 * PI;
            Self {
                x: r * sin_theta * phi.cos(),
                y: r * sin_theta * phi.sin(),
                z: r * cos_theta,
                r,
                cos_theta,
                phi,
            }
        })
    }

    /// Same as [`Point::sample_from_ball_iter`], but with [`Point::ORIGIN_EPSILON`] guaranteed as
    /// the first point sampled:
    /// ```
    /// # use evanescence_core::geometry::Point;
    /// assert_eq!(
    ///     Some(Point::ORIGIN_EPSILON),
    ///     Point::sample_from_ball_with_origin_iter(1.0).next()
    /// );
    /// ```
    pub fn sample_from_ball_with_origin_iter(radius: f64) -> impl Iterator<Item = Self> {
        iter::once(Self::ORIGIN_EPSILON).chain(Self::sample_from_ball_iter(radius))
    }
}

/// Type representing a coordinate plane.
#[derive(Clone, Copy, Debug, Display)]
pub enum Plane {
    XY,
    YZ,
    ZX,
}

impl Plane {
    /// Get the basis vectors for the coordinate plane, in terms of the standard basis for `R^3`.
    ///
    /// The basis is oriented such that `e_1 × e_2` is the third standard basis vector for `R^3`.
    pub fn basis_vectors(self) -> (Vec3, Vec3) {
        match self {
            Self::XY => (Vec3::I, Vec3::J),
            Self::YZ => (Vec3::J, Vec3::K),
            Self::ZX => (Vec3::K, Vec3::I),
        }
    }

    /// Get the names of the two coordinate axes defining the plane, in order.
    pub fn axes_names(self) -> (char, char) {
        match self {
            Self::XY => ('x', 'y'),
            Self::YZ => ('y', 'z'),
            Self::ZX => ('z', 'x'),
        }
    }
}

/// A point and the value of a function evaluated at that point.
pub type PointValue<T> = (Point, T);

/// Type storing a collection of evaluations, where values in each dimension (x, y, z, and value)
/// is stored in a separate vector. Each index, across the four vectors, corresponds to
/// a single point and its associated value.
///
/// It may be thought of as the transpose of `Vec<Evaluation<T>>`.
///
/// This type cannot be manually constructed and should instead be obtained from a
/// [`Vec<Evaluation<T>>`] via conversion traits.
///
/// # Safety
/// All four vectors must be the same length.
#[derive(Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct ComponentForm<T> {
    /// List of x-values.
    xs: Vec<f64>,
    /// List of y-values.
    ys: Vec<f64>,
    /// List of z-values.
    zs: Vec<f64>,
    /// List of wavefunction values evaluated at the corresponding (x, y, z) coordinate.
    vals: Vec<T>,
}

impl<T> ComponentForm<T> {
    /// Decompose `Self` into a four-tuple of its inner vectors,
    /// in the order (x, y, z, value).
    pub fn into_components(self) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<T>) {
        (self.xs, self.ys, self.zs, self.vals)
    }
}

/// Swizzling decomposition operators.
impl<T> ComponentForm<T> {
    /// Return the x and value components.
    pub fn into_xv(self) -> (Vec<f64>, Vec<T>) {
        (self.xs, self.vals)
    }
    /// Return the x, y, and value components.
    pub fn into_xyv(self) -> (Vec<f64>, Vec<f64>, Vec<T>) {
        (self.xs, self.ys, self.vals)
    }
    /// Return the y, z, and value components.
    pub fn into_yzv(self) -> (Vec<f64>, Vec<f64>, Vec<T>) {
        (self.ys, self.zs, self.vals)
    }
    /// Return the x, z, and value components.
    pub fn into_xzv(self) -> (Vec<f64>, Vec<f64>, Vec<T>) {
        (self.xs, self.zs, self.vals)
    }
}

impl<T> From<Vec<PointValue<T>>> for ComponentForm<T> {
    fn from(v: Vec<PointValue<T>>) -> Self {
        let len = v.len();
        let (mut xs, mut ys, mut zs, mut vals) = (
            Vec::with_capacity(len),
            Vec::with_capacity(len),
            Vec::with_capacity(len),
            Vec::with_capacity(len),
        );
        v.into_iter().for_each(|(pt, val)| {
            xs.push(pt.x());
            ys.push(pt.y());
            zs.push(pt.z());
            vals.push(val);
        });
        ComponentForm { xs, ys, zs, vals }
    }
}

/// A grid of points on a specified plane and a value associated with each point in the grid.
///
/// This type represents values in the manner expected by Plotly's "Surface" plot. That is,
/// the x coordinates (here `col_coords`) and y coordinates (here `row_coords`) of the points
/// in the grid are each represented as a one-dimensional list. The points in the grid are
/// then taken to be the Cartesian product of the x and y coordinates.
///
/// Values, in turn, are stored as a two-dimensional list: a column of rows, where a "column"
/// has constant `row_coord` value and a "row" has constant `col_coord` value.
///
/// Graphically, the layout is as follows:
///
/// ```text
///      [   3    6    9]  <---- `col_coords`
/// ⎴  [
/// 2    [v_00 v_01 v_02],
/// 4    [v_10 v_11 v_12],
/// 6    [v_20 v_21 v_22],
/// 8    [v_30 v_31 v_32],
/// ⎵  ]  <---- `vals`
/// ^---- `row_coords`
/// ```
///
/// Taking this `GridValues` to be on the xy-plane, the points would thus be found at the following
/// positions:
///
/// ```text
///  y
///  ^
///  |
///  8      v_30   v_31   v_32
///  |
///  6      v_20   v_21   v_22
///  |
///  4      v_10   v_11   v_12
///  |
///  2      v_00   v_01   v_02
///  |
/// -+------3------6------9------>x
///  |
///  ```
///
/// # Safety
/// `col_coords`, `row_coords`, and `vals` must have matching shapes (see [`Self::new`]).
#[derive(Getters)]
#[getset(get = "pub")]
pub struct GridValues<T> {
    /// The plane on which the grid is situated.
    plane: Plane,
    /// The "horizontal" coordinate of each column.
    col_coords: Vec<f64>,
    /// The "vertical" coordinate of each row.
    row_coords: Vec<f64>,
    /// The values associated with the grid, stored as a column of rows.
    vals: Vec<Vec<T>>,
}

impl<T> GridValues<T> {
    /// Create a new `GridValues` from components.
    ///
    /// # Panics
    /// This function will panic if the `Vec`s passed in do not have the correct shape:
    ///
    /// ```should_panic
    /// # use evanescence_core::geometry::{Plane, GridValues};
    /// GridValues::<f64>::new(
    ///     Plane::XY,
    ///     vec![0.0, 1.0], // There are two columns.
    ///     vec![0.0, 1.0, 2.0], // There are three rows.
    ///     vec![ // This is a column of rows.
    ///         vec![3.1, 4.1],
    ///         vec![1.5, 9.2],
    ///         // The third row is missing!!
    ///     ],
    /// );
    /// ```
    pub fn new(
        plane: Plane,
        col_coords: Vec<f64>,
        row_coords: Vec<f64>,
        vals: Vec<Vec<T>>,
    ) -> Self {
        // SAFETY: Verify that the passed `Vec`s have the correct shape.
        assert_eq!(row_coords.len(), vals.len());
        vals.iter()
            .for_each(|row| assert_eq!(col_coords.len(), row.len()));
        Self {
            plane,
            col_coords,
            row_coords,
            vals,
        }
    }

    /// Decompose `self` into a 3-tuple of column coordinates ("x coordinates"), row coordinates
    /// ("y coordinates"), and values, in that order.
    pub fn into_components(self) -> (Vec<f64>, Vec<f64>, Vec<Vec<T>>) {
        (self.col_coords, self.row_coords, self.vals)
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use crate::geometry::Point;

    /// This is very crude and only ensures that all pointsare at least inside
    /// the expected radius. It makes no attempt to verify the uniformity of
    /// the distribution produced.
    #[test]
    fn test_point_rng_max_radius() {
        let sampling_radius = 2_f64;
        Point::sample_from_ball_iter(sampling_radius)
            .take(10_000)
            .for_each(|pt| assert!(pt.r < sampling_radius));
    }

    #[test]
    fn test_spherical_coordinates() {
        let point = Point::new(1.0, 2.0, -3.0);
        assert_relative_eq!(point.r, 3.7416573867739413856);
        assert_relative_eq!(point.cos_theta.acos(), 2.5010703409103686643);
        assert_relative_eq!(point.phi, 1.1071487177940905030);
        let recomputed_point = Point::new_spherical(point.r, point.cos_theta.acos(), point.phi);
        assert_relative_eq!(point.x, recomputed_point.x);
        assert_relative_eq!(point.y, recomputed_point.y);
        assert_relative_eq!(point.z, recomputed_point.z);
    }

    #[test]
    fn test_rng_spherical_coordinates() {
        let rng_point = Point::sample_from_ball_iter(2.0).next().unwrap();
        let recomputed_point = Point::new(rng_point.x, rng_point.y, rng_point.z);
        assert_relative_eq!(rng_point.r, recomputed_point.r);
        assert_relative_eq!(rng_point.cos_theta, recomputed_point.cos_theta);
        assert_relative_eq!(rng_point.phi, recomputed_point.phi);
    }

    // #[test]
    // fn print_random_points() {
    //     Point::random_in_ball_iter(10.0)
    //         .take(10)
    //         .for_each(|pt| println!("Point::new({}, {}, {}),", pt.x, pt.y, pt.z));
    // }
}
