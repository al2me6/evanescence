use std::ops::RangeInclusive;

use crate::geometry::region::{BallCenteredAtOrigin, BoundingRegion};
use crate::geometry::vec3::Vec3;
use crate::geometry::{
    self, ComponentForm, CoordinatePlane, GridValues, PointValue, SphericalPoint3,
};

/// Trait for mathematical functions that can be evaluated at a point in `R^3`.
///
/// Utilities are provided for sampling the function on a line or plane.
pub trait Evaluate {
    type Output: Copy;

    /// Evaluate `Self` at a certain point, returning the value only.
    fn evaluate(&self, point: &SphericalPoint3) -> Self::Output;

    /// Evaluate `Self` at a certain point, returning the point *and* the value in the form of a
    /// [`PointValue`], or a `(Point, Self::Output)`.
    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn evaluate_at(&self, point: &SphericalPoint3) -> PointValue<Self::Output> {
        PointValue(*point, self.evaluate(point))
    }

    /// Evaluate `Self` on a line segment running across `range` at a total of `num_points`
    /// different points, all evenly spaced (à la "`linspace`" operation).
    fn evaluate_on_line_segment(
        &self,
        range: RangeInclusive<Vec3>,
        num_points: usize,
    ) -> Vec<PointValue<Self::Output>> {
        geometry::linspace(range, num_points)
            .map(|pt| self.evaluate_at(&pt.into()))
            .collect()
    }

    /// Evaluate `Self` on a [`CoordinatePlane`], producing a [grid](crate::geometry::GridValues)
    /// of evenly spaced values. Specifically, the grid is a square centered at the origin with
    /// side length of 2 × `extent`, and `num_points` are sampled *in each dimension*.
    fn evaluate_on_plane(
        &self,
        plane: CoordinatePlane,
        extent: f32,
        num_points: usize,
    ) -> GridValues<Self::Output> {
        type ComponentGetter = fn(&Vec3) -> f32;
        // Functions to extract the correct component of the `Vec3`.
        let extract_component: (ComponentGetter, ComponentGetter) = match plane {
            CoordinatePlane::XY => (Vec3::get_x, Vec3::get_y),
            CoordinatePlane::YZ => (Vec3::get_y, Vec3::get_z),
            CoordinatePlane::ZX => (Vec3::get_z, Vec3::get_x),
        };

        // The midpoints of the grid's "right" and "top" edges.
        let midpoints = {
            let basis = plane.basis_vectors();
            (basis.0 * extent, basis.1 * extent)
        };

        // Points linearly dependent on `e_0`, i.e., the center row.
        let points_in_row: Vec<_> = Vec3::symmetric_linspace(midpoints.0, num_points).collect();
        // Points linearly dependent on `e_1`, i.e., the center column.
        let points_in_col: Vec<_> = Vec3::symmetric_linspace(midpoints.1, num_points).collect();

        let mut vals = Vec::with_capacity(num_points);

        for &col_pt in &points_in_col {
            let mut row = Vec::with_capacity(num_points);
            for &row_pt in &points_in_row {
                row.push(self.evaluate(&(row_pt + col_pt).into()));
            }
            vals.push(row);
        }

        GridValues::new(
            plane,
            points_in_row.iter().map(extract_component.0).collect(),
            points_in_col.iter().map(extract_component.1).collect(),
            vals,
        )
        .expect("rows and columns are equal in length by construction")
    }

    /// Evaluate `Self` on a cube of side length 2 × `extent`, centered at the origin, producing
    /// a list of evenly spaced points arranged as a flattened 3D array, with the first index
    /// being x, second index being y, and third index being z.
    ///
    /// That is, values are each of the form (x, y, z, val), sorted by increasing x, then y, and
    /// finally z.
    fn evaluate_in_region(&self, extent: f32, num_points: usize) -> ComponentForm<Self::Output> {
        Vec3::symmetric_linspace(Vec3::I * extent, num_points)
            .flat_map(|x_pt| {
                Vec3::symmetric_linspace(Vec3::J * extent, num_points).flat_map(move |y_pt| {
                    Vec3::symmetric_linspace(Vec3::K * extent, num_points)
                        .map(move |z_pt| self.evaluate_at(&(x_pt + y_pt + z_pt).into()))
                })
            })
            .collect::<Vec<_>>()
            .into()
    }
}

/// Helper methods for evaluating functions that can be reasonably 'bounded' in a region symmetric
/// about the origin.
pub trait EvaluateInOriginCenteredRegionExt: Evaluate {
    /// Sample the cross section of the function along a given `plane`.
    ///
    /// `num_points` points will be evaluated in a grid centered at the origin, extending to the
    /// bound of the function.
    fn sample_plane(&self, plane: CoordinatePlane, num_points: usize) -> GridValues<Self::Output>;

    /// Sample the function in a cube centered at the origin. `num_points` are sampled in each
    /// dimension, producing an evenly-spaced lattice of values the size of the function's bound.
    fn sample_region(&self, num_points: usize) -> ComponentForm<Self::Output>;
}

impl<E> EvaluateInOriginCenteredRegionExt for E
where
    E: Evaluate + BoundingRegion<Geometry = BallCenteredAtOrigin>,
{
    fn sample_plane(&self, plane: CoordinatePlane, num_points: usize) -> GridValues<Self::Output> {
        self.evaluate_on_plane(plane, self.bounding_region().radius, num_points)
    }

    fn sample_region(&self, num_points: usize) -> ComponentForm<Self::Output> {
        self.evaluate_in_region(self.bounding_region().radius, num_points)
    }
}
