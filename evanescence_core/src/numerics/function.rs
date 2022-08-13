use std::ops::RangeInclusive;

use itertools::Itertools;
use na::{Const, SVector, ToTypenum, Vector3};

use crate::geometry::point::IPoint;
use crate::geometry::region::{BallCenteredAtOrigin, BoundingRegion};
use crate::geometry::storage::grid_values::CoordinatePlane3;
use crate::geometry::storage::{GridValues3, PointValue, Soa};

pub trait Function<const N: usize, P: IPoint<N> = na::Point<f32, N>> {
    type Output: Copy;

    /// Evaluate `Self` at a certain point, returning the value only.
    fn evaluate(&self, point: &P) -> Self::Output;

    /// Evaluate `Self` at a certain point, returning the point *and* the value in the form of a
    /// [`PointValue`], or a `(Point, Self::Output)`.
    #[inline]
    fn evaluate_at(&self, point: P) -> PointValue<N, P, Self::Output> {
        let value = self.evaluate(&point);
        PointValue(point, value)
    }

    /// Evaluate `Self` on a line segment running across `range` at a total of `num_points`
    /// different points, all evenly spaced (à la [`linspace`](super::linspace) operation).
    fn evaluate_on_line_segment(
        &self,
        range: RangeInclusive<SVector<f32, N>>,
        num_points: usize,
    ) -> Vec<PointValue<N, P, Self::Output>>
    where
        Const<N>: ToTypenum,
        <Const<N> as ToTypenum>::Typenum: typenum::Cmp<typenum::U0, Output = typenum::Greater>,
    {
        super::linspace(range, num_points)
            .map(|pt| self.evaluate_at(pt.into()))
            .collect()
    }
}

impl<const N: usize, P: IPoint<N>, O: Copy> Function<N, P> for Box<dyn Function<N, P, Output = O>> {
    type Output = O;

    fn evaluate(&self, point: &P) -> Self::Output {
        self.as_ref().evaluate(point)
    }
}

pub trait Function3Ext<P: IPoint<3> = na::Point3<f32>>: Function<3, P> {
    /// Evaluate `Self` on a [`CoordinatePlane3`], producing a [grid](crate::geometry::storage::GridValues3)
    /// of evenly spaced values. Specifically, the grid is a square centered at the origin with
    /// side length of 2 × `extent`, and `num_points` are sampled *in each dimension*.
    fn evaluate_on_plane(
        &self,
        plane: CoordinatePlane3,
        extent: f32,
        num_points: usize,
    ) -> GridValues3<Self::Output>;

    /// Evaluate `Self` on a cube of side length 2 × `extent`, centered at the origin, producing
    /// a list of evenly spaced points arranged as a flattened 3D array, with the first index
    /// being x, second index being y, and third index being z.
    ///
    /// That is, values are each of the form (x, y, z, val), sorted by increasing x, then y, and
    /// finally z.
    fn evaluate_in_region(&self, extent: f32, num_points: usize) -> Soa<3, Self::Output>;
}

impl<P: IPoint<3>, F> Function3Ext<P> for F
where
    F: Function<3, P>,
{
    fn evaluate_on_plane(
        &self,
        plane: CoordinatePlane3,
        extent: f32,
        num_points: usize,
    ) -> GridValues3<Self::Output> {
        // Indices cooresponding to the components of the `Vector3`.
        let component_idx: (usize, usize) = match plane {
            CoordinatePlane3::XY => (0, 1),
            CoordinatePlane3::YZ => (1, 2),
            CoordinatePlane3::ZX => (2, 0),
        };

        // The midpoints of the grid's "right" and "top" edges.
        let midpoints = {
            let basis = plane.basis_vectors();
            (basis.0 * extent, basis.1 * extent)
        };

        // Points linearly dependent on `e_0`, i.e., the center row.
        let points_in_row = super::symmetric_linspace(midpoints.0, num_points).collect_vec();
        // Points linearly dependent on `e_1`, i.e., the center column.
        let points_in_col = super::symmetric_linspace(midpoints.1, num_points).collect_vec();

        let mut vals = Vec::with_capacity(num_points);

        for &col_pt in &points_in_col {
            let mut row = Vec::with_capacity(num_points);
            for &row_pt in &points_in_row {
                row.push(self.evaluate(&(row_pt + col_pt).into()));
            }
            vals.push(row);
        }

        GridValues3::new(
            plane,
            points_in_row.iter().map(|p| p[component_idx.0]).collect(),
            points_in_col.iter().map(|p| p[component_idx.1]).collect(),
            vals,
        )
        .expect("rows and columns are equal in length by construction")
    }

    fn evaluate_in_region(&self, extent: f32, num_points: usize) -> Soa<3, Self::Output> {
        super::symmetric_linspace(Vector3::x() * extent, num_points)
            .flat_map(|x_pt| {
                super::symmetric_linspace(Vector3::y() * extent, num_points).flat_map(move |y_pt| {
                    super::symmetric_linspace(Vector3::z() * extent, num_points)
                        .map(move |z_pt| self.evaluate_at((x_pt + y_pt + z_pt).into()))
                })
            })
            .collect()
    }
}

/// Helper methods for evaluating Function<3>s that can be reasonably 'bounded' in a region
/// symmetric about the origin.
pub trait Function3InOriginCenteredRegionExt<P: IPoint<3>>: Function3Ext<P> {
    /// Sample the cross section of the function along a given `plane`.
    ///
    /// `num_points` points will be evaluated in a grid centered at the origin, extending to the
    /// bound of the function.
    fn sample_plane(&self, plane: CoordinatePlane3, num_points: usize)
        -> GridValues3<Self::Output>;

    /// Sample the function in a cube centered at the origin. `num_points` are sampled in each
    /// dimension, producing an evenly-spaced lattice of values the size of the function's bound.
    fn sample_region(&self, num_points: usize) -> Soa<3, Self::Output>;
}

impl<P, F> Function3InOriginCenteredRegionExt<P> for F
where
    P: IPoint<3>,
    F: Function3Ext<P> + BoundingRegion<3, P, Geometry = BallCenteredAtOrigin>,
{
    fn sample_plane(
        &self,
        plane: CoordinatePlane3,
        num_points: usize,
    ) -> GridValues3<Self::Output> {
        self.evaluate_on_plane(plane, self.bounding_region().radius, num_points)
    }

    fn sample_region(&self, num_points: usize) -> Soa<3, Self::Output> {
        self.evaluate_in_region(self.bounding_region().radius, num_points)
    }
}
