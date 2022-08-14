use getset::Getters;
use na::Vector3;
use strum::Display;
use thiserror::Error;

/// A grid of points on a plane and a value associated with each point in the grid.
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
/// Calling the plane xy, the points would thus be found at the following positions:
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
/// # Invariants
/// `col_coords`, `row_coords`, and `vals` must have matching shapes.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Getters, Debug, PartialEq)]
#[getset(get = "pub")]
pub struct GridValues<V> {
    /// The "horizontal" coordinate of each column.
    col_coords: Vec<f32>,
    /// The "vertical" coordinate of each row.
    row_coords: Vec<f32>,
    /// The values associated with the grid, stored as a column of rows.
    vals: Vec<Vec<V>>,
}

/// Error type describing invalid [`GridValues3`] geometries.
#[derive(PartialEq, Eq, Debug, Error)]
pub enum InvalidGridValuesError {
    /// Incorrect number of row coordinates.
    #[error("number of row coordinates does not match number of value rows")]
    Row,
    /// Incorrect number of column coordinates.
    #[error("number of column coordinates does not match length of value row")]
    Column,
}

impl<V> GridValues<V> {
    /// Create a new `GridValues` from components.
    ///
    /// # Errors
    /// This function errors if the `Vec`s passed in do not have the correct shape:
    ///
    /// ```
    /// # use evanescence_core::geometry::storage::grid_values::{GridValues, InvalidGridValuesError};
    /// let wrong_rows = GridValues::<f32>::new(
    ///     vec![0.0, 1.0],      // There are two columns.
    ///     vec![0.0, 1.0, 2.0], // There are three rows.
    ///     vec![
    ///         // This is a column of rows.
    ///         vec![3.1, 4.1], // Each row has two values (corresponding to the two columns).
    ///         vec![1.5, 9.2],
    ///         // The third row is missing!!
    ///     ],
    /// );
    /// assert_eq!(Err(InvalidGridValuesError::Row), wrong_rows);
    /// ```
    pub fn new(
        col_coords: Vec<f32>,
        row_coords: Vec<f32>,
        vals: Vec<Vec<V>>,
    ) -> Result<Self, InvalidGridValuesError> {
        // INVARIANT: Verify that the passed `Vec`s have the correct shape.
        if row_coords.len() != vals.len() {
            return Err(InvalidGridValuesError::Row);
        }
        for row in &vals {
            if col_coords.len() != row.len() {
                return Err(InvalidGridValuesError::Column);
            }
        }
        Ok(Self {
            col_coords,
            row_coords,
            vals,
        })
    }

    /// Decompose `self` into a 3-tuple of column coordinates ("x coordinates"), row coordinates
    /// ("y coordinates"), and values, in that order.
    pub fn decompose(self) -> (Vec<f32>, Vec<f32>, Vec<Vec<V>>) {
        (self.col_coords, self.row_coords, self.vals)
    }
}

/// Type representing one of the three coordinate planes in `R^3`.
#[allow(clippy::upper_case_acronyms)] // "XY", etc. are not acronyms.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Display)]
pub enum CoordinatePlane3 {
    XY,
    YZ,
    ZX,
}

impl CoordinatePlane3 {
    /// Get the basis vectors for the coordinate plane, in terms of the standard basis for `R^3`.
    ///
    /// The basis is oriented such that `e_1 × e_2` is the third standard basis vector for `R^3`.
    pub fn basis_vectors(self) -> (Vector3<f32>, Vector3<f32>) {
        match self {
            Self::XY => (Vector3::x(), Vector3::y()),
            Self::YZ => (Vector3::y(), Vector3::z()),
            Self::ZX => (Vector3::z(), Vector3::x()),
        }
    }

    /// Get the names of the two coordinate axes defining the plane, in order.
    pub fn axes_names(self) -> (&'static str, &'static str) {
        match self {
            Self::XY => ("x", "y"),
            Self::YZ => ("y", "z"),
            Self::ZX => ("z", "x"),
        }
    }

    /// Give an ordered triple describing which coordinate axis takes on the value of zero.
    /// (Ex. `(x, y, 0)`.)
    pub fn ordered_triple_form(self) -> &'static str {
        match self {
            Self::XY => "(x, y, 0)",
            Self::YZ => "(0, y, z)",
            Self::ZX => "(x, 0, z)",
        }
    }

    /// Produce coordinates corresponding to a square in the plane, *with respect to the xy-plane*.
    /// That is, the values are always represented as (x, y, z), regardless of whether the plane
    /// itself is the xy-plane.
    ///
    /// Specifically, the square has side length `2 * extent` and is centered at the origin.
    pub fn square_wrt_xy_plane(self, extent: f32) -> GridValues<f32> {
        match self {
            Self::XY => GridValues {
                col_coords: vec![-extent, extent],
                row_coords: vec![-extent, extent],
                vals: vec![vec![0.0, 0.0]; 2],
            },
            Self::YZ => GridValues {
                col_coords: vec![0.0, 0.0],
                row_coords: vec![-extent, extent],
                vals: vec![vec![-extent, extent]; 2],
            },
            Self::ZX => GridValues {
                col_coords: vec![-extent, extent],
                row_coords: vec![0.0, 0.0],
                vals: vec![vec![-extent; 2], vec![extent; 2]],
            },
        }
    }
}

/// A 2D [`GridValues`] embedded in `R^3`, with a marker indicating the reference plane.
pub struct GridValues3<V> {
    /// The plane on which the grid is situated.
    pub reference_plane: CoordinatePlane3,
    pub grid_values: GridValues<V>,
}
