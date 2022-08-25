use std::ops::RangeInclusive;

use itertools::Itertools;
use na::{vector, Const, SVector, ToTypenum, Vector2};

use crate::geometry::point::IPoint;
use crate::geometry::region::{BallCenteredAtOrigin, BoundingRegion};
use crate::geometry::storage::grid_values::{CoordinatePlane3, GridValues, GridValues3};
use crate::geometry::storage::{PointValue, Soa};

pub trait Function<const N: usize, P: IPoint<N> = na::Point<f32, N>> {
    type Output: Copy;

    /// Evaluate `self` at a certain point, returning the value only.
    fn evaluate(&self, point: &P) -> Self::Output;

    /// Evaluate `self` at a certain point, returning the point *and* the value in the form of a
    /// [`PointValue`].
    #[inline]
    fn evaluate_at(&self, point: P) -> PointValue<N, P, Self::Output> {
        let value = self.evaluate(&point);
        PointValue(point, value)
    }

    /// Sample `self` in an `N`-dimensional hypercube centered at the origin, giving an
    /// evenly-spaced lattice.
    fn sample_from_origin_centered_hypercube(
        &self,
        edge_length: f32,
        num_points_per_dim: usize,
    ) -> Soa<N, Self::Output> {
        let coords = super::symmetric_linspace(edge_length / 2., num_points_per_dim).collect_vec();
        itertools::repeat_n(&coords, N)
            .multi_cartesian_product()
            .map(|pt_coords| pt_coords.into_iter().copied())
            .map(SVector::<_, N>::from_iterator)
            .map(|v| self.evaluate_at(v.into()))
            .collect()
    }

    /// Sample `self` on a line segment running across `interval` at a total of `num_points`
    /// evenly spaced points.
    fn sample_from_line_segment(
        &self,
        interval: RangeInclusive<SVector<f32, N>>,
        num_points: usize,
    ) -> Soa<N, Self::Output> {
        super::linspace(interval, num_points)
            .map(|pt| self.evaluate_at(pt.into()))
            .collect()
    }

    /// For `Function<2>` only.
    ///
    /// Sample `self` in a rectangular lattice given by `extent`, where the 'lower bound' is the
    /// 'lower left corner' and the 'upper bound' is the 'top right corner'.
    fn sample_from_plane(
        &self,
        extent: RangeInclusive<Vector2<f32>>,
        num_points: [usize; 2],
    ) -> GridValues<Self::Output>
    where
        Const<N>: ToTypenum,
        <Const<N> as ToTypenum>::Typenum: tn::Cmp<tn::U2, Output = tn::Equal>,
        P: From<Vector2<f32>>, // rustc cannot prove this from above.
    {
        let (bottom_left, top_right) = extent.into_inner();

        let xs = super::linspace(bottom_left.x..=top_right.x, num_points[0]).collect_vec();
        let ys = super::linspace(bottom_left.y..=top_right.y, num_points[1]).collect_vec();
        let mut vals = Vec::with_capacity(num_points[0]);

        for &y in &ys {
            let mut row = Vec::with_capacity(num_points[1]);
            for &x in &xs {
                row.push(self.evaluate(&vector![x, y].into()));
            }
            vals.push(row);
        }

        GridValues::new(xs, ys, vals).expect("rows and columns are equal in length by construction")
    }
}

impl<const N: usize, P: IPoint<N>, O: Copy> Function<N, P> for Box<dyn Function<N, P, Output = O>> {
    type Output = O;

    fn evaluate(&self, point: &P) -> Self::Output {
        self.as_ref().evaluate(point)
    }
}

pub trait Function3Ext<P: IPoint<3> = na::Point3<f32>>: Function<3, P> {
    /// Evaluate `Self` on a [`CoordinatePlane3`], producing a
    /// [grid](crate::geometry::storage::grid_values::GridValues3) of evenly spaced values.
    /// Specifically, the grid is a square centered at the origin with side length of 2 × `extent`,
    /// and `num_points` are sampled *in each dimension*.
    fn sample_in_plane(
        &self,
        plane: CoordinatePlane3,
        extent: f32,
        num_points: usize,
    ) -> GridValues3<Self::Output>;
}

impl<P: IPoint<3>, F> Function3Ext<P> for F
where
    F: Function<3, P>,
{
    fn sample_in_plane(
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

        GridValues3 {
            reference_plane: plane,
            grid_values: GridValues::new(
                points_in_row.iter().map(|p| p[component_idx.0]).collect(),
                points_in_col.iter().map(|p| p[component_idx.1]).collect(),
                vals,
            )
            .expect("rows and columns are equal in length by construction"),
        }
    }
}

/// Helper methods for evaluating Function<3>s that can be reasonably 'bounded' in a region
/// symmetric about the origin.
pub trait Function3InOriginCenteredRegionExt<P: IPoint<3>>: Function3Ext<P> {
    /// Sample the cross section of the function along a given `plane`.
    ///
    /// `num_points` points will be evaluated in a grid centered at the origin, extending to the
    /// bound of the function.
    fn bounded_sample_in_plane(
        &self,
        plane: CoordinatePlane3,
        num_points: usize,
    ) -> GridValues3<Self::Output>;

    /// Sample the function in a cube centered at the origin. `num_points` are sampled in each
    /// dimension, producing an evenly-spaced lattice of values the size of the function's bound.
    fn bounded_sample_in_cube(&self, num_points: usize) -> Soa<3, Self::Output>;
}

impl<P, F> Function3InOriginCenteredRegionExt<P> for F
where
    P: IPoint<3>,
    F: Function3Ext<P> + BoundingRegion<3, P, Geometry = BallCenteredAtOrigin>,
{
    fn bounded_sample_in_plane(
        &self,
        plane: CoordinatePlane3,
        num_points: usize,
    ) -> GridValues3<Self::Output> {
        self.sample_in_plane(plane, self.bounding_region().radius, num_points)
    }

    fn bounded_sample_in_cube(&self, num_points: usize) -> Soa<3, Self::Output> {
        self.sample_from_origin_centered_hypercube(self.bounding_region().radius * 2., num_points)
    }
}
