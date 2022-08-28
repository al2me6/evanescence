use std::collections::BinaryHeap;
use std::marker::PhantomData;
use std::rc::Rc;
use std::{cmp, mem};

use itertools::Itertools;
use na::allocator::Allocator as MatAllocator;
use na::{vector, Const, DimAdd, DimName, DimSum, OMatrix, OVector, SVector, U1};

use crate::geometry::point::IPoint;
use crate::geometry::region::{BallCenteredAtOrigin, BoundingRegion};
use crate::geometry::storage::PointValue;
use crate::numerics::Function;

type Vector<const N: usize> = SVector<f32, N>;
type Np1<const N: usize> = DimSum<Const<N>, U1>;
type VectorNp1<const N: usize> = OVector<f32, Np1<N>>;
type MatrixNxNp1<const N: usize> = OMatrix<f32, Const<N>, Np1<N>>;

/// An `N`-dimensional simplex, containing information about the objective function inside its
/// volume.
#[derive(Clone, Debug)]
struct Simplex<const N: usize>
where
    Const<N>: DimAdd<U1>,
    na::DefaultAllocator: MatAllocator<f32, Np1<N>, U1>,
{
    /// The vertices of this `N`-simplex.
    vertices: Vec<Rc<Vector<N>>>,
    /// The value of the objective at each vertex.
    values: VectorNp1<N>,
    /// The fraction of the entire search space contained in `self`.
    content_fraction_of_root: f32,
    /// Position of the vertex used to subdivide this simplex into `N` sub-simplices.
    subdivision_point: Rc<Vector<N>>,
    /// The fraction of `self`'s content that each subdivided sub-simplex would contain.
    /// Sums to unity.
    content_fractions_of_subdivisions: VectorNp1<N>,
    /// The value of the objective at the subdivision point, as estimated by inverse distance
    /// interpolation.
    interpolated_value: f32,
    /// A measure of the degree to which the content of `self` has already been explored.
    ///
    /// `delta * log(content_fraction_of_root, base = NUM_VERTICES)`.
    ///
    /// For practical reasons, this actually stores the fixed (_i.e._, non-`delta`) parts of the
    /// `acquisition_value` computation: `exploration_preference * log(..)`. This allows updating
    /// `acquisition_value` with fewer operations.
    opportunity_cost: f32,
    /// The delta value (_i.e._, `max - min`) used to compute the current acquisition value.
    last_known_delta: f32,
    /// Number quantifying the 'quality' of this simplex.
    ///
    /// `interpolated_value + exploration_preference * opportunity_cost`.
    acquisition_value: f32,
}

impl<const N: usize> Simplex<N>
where
    Const<N>: DimAdd<U1>,
    Np1<N>: DimName,
    na::DefaultAllocator: MatAllocator<f32, Np1<N>, U1>
        + MatAllocator<f32, Const<N>, Np1<N>>
        + MatAllocator<f32, Np1<N>, Const<N>>,
{
    const NUM_VERTICES: usize = N + 1;

    /// Construct a new simplex with the given vertices and values. Compute the location of the
    /// subdivision point and the interpolated objective value at that point.
    fn new(
        vertices: Vec<Rc<Vector<N>>>,
        values: VectorNp1<N>,
        content_fraction_of_root: f32,
        exploration_preference: f32,
        delta: f32,
    ) -> Self {
        assert!((0.0..=1.0).contains(&content_fraction_of_root));

        let vertices_mat =
            MatrixNxNp1::<N>::from_iterator(vertices.iter().flat_map(|v| v.iter()).copied());

        let barycenter: Vector<N> = vertices_mat.column_sum() / (vertices_mat.ncols() as f32);
        let dists_from_barycenter = VectorNp1::<N>::from_iterator(
            vertices_mat
                .column_iter()
                .map(|vert| vert.metric_distance(&barycenter)),
        );

        let subdivision_pt_barycentric: VectorNp1<N> =
            (&dists_from_barycenter) / dists_from_barycenter.sum();
        approx::assert_abs_diff_eq!(subdivision_pt_barycentric.sum(), 1.0, epsilon = 1E-3);

        let subdivision_pt_cartesian: Vector<N> = (&vertices_mat) * (&subdivision_pt_barycentric);

        let inverse_dists_from_subdivision_pt = VectorNp1::<N>::from_iterator(
            vertices_mat
                .column_iter()
                .map(|vert| vert.metric_distance(&subdivision_pt_cartesian).recip()),
        );

        let interpolated_value = inverse_dists_from_subdivision_pt.dot(&values)
            / inverse_dists_from_subdivision_pt.sum();

        let opportunity_cost =
            exploration_preference * content_fraction_of_root.log(Simplex::<N>::NUM_VERTICES as _);

        let mut this = Self {
            vertices,
            values,
            content_fraction_of_root,
            subdivision_point: Rc::new(subdivision_pt_cartesian),
            content_fractions_of_subdivisions: subdivision_pt_barycentric,
            interpolated_value,
            opportunity_cost,
            last_known_delta: 0.,
            acquisition_value: 0.,
        };
        this.update_acquisition_value(delta);
        this
    }

    /// Recompute the acquisition value based on a new delta.
    fn update_acquisition_value(&mut self, delta: f32) {
        self.acquisition_value = self.interpolated_value + self.opportunity_cost * delta;
        self.last_known_delta = delta;
    }
}

impl<const N: usize> PartialEq for Simplex<N>
where
    Const<N>: DimAdd<U1>,
    na::DefaultAllocator: MatAllocator<f32, Np1<N>, U1>,
{
    fn eq(&self, other: &Self) -> bool {
        self.acquisition_value == other.acquisition_value
    }
}

impl<const N: usize> Eq for Simplex<N>
where
    Const<N>: DimAdd<U1>,
    na::DefaultAllocator: MatAllocator<f32, Np1<N>, U1>,
{
}

impl<const N: usize> PartialOrd for Simplex<N>
where
    Const<N>: DimAdd<U1>,
    na::DefaultAllocator: MatAllocator<f32, Np1<N>, U1>,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<const N: usize> Ord for Simplex<N>
where
    Const<N>: DimAdd<U1>,
    na::DefaultAllocator: MatAllocator<f32, Np1<N>, U1>,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.acquisition_value.total_cmp(&other.acquisition_value)
    }
}

/// The Simple(x) global optimization algorithm, based on the reference Python implementation
/// <https://github.com/chrisstroemel/Simple>.
pub struct Simple<const N: usize, P, Obj>
where
    P: IPoint<N>,
    Obj: FnMut(&P) -> f32,
    Const<N>: DimAdd<U1>,
    na::DefaultAllocator: MatAllocator<f32, Np1<N>, U1>,
{
    queue: BinaryHeap<Simplex<N>>,
    objective: Obj,
    max_value: f32,
    min_value: f32,
    best_point: Rc<Vector<N>>,
    exploration_preference: f32,
    phantom: PhantomData<P>,
}

impl<const N: usize, P, Obj> Simple<N, P, Obj>
where
    P: IPoint<N>,
    Obj: FnMut(&P) -> f32,
    Const<N>: DimAdd<U1>,
    Np1<N>: DimName,
    na::DefaultAllocator: MatAllocator<f32, Np1<N>, U1>
        + MatAllocator<f32, Const<N>, Np1<N>>
        + MatAllocator<f32, Np1<N>, Const<N>>,
{
    /// Construct a new Simple(x) optimizer for the given `objective` function.
    ///
    /// `search_space` contains the vertices of an `N`-simplex, and `exploration_preference` is
    /// a parameter controlling the algorithm's preference to explore other extrema (as opposed
    /// to pursuing the first one found).
    pub fn new(search_space: Vec<Vector<N>>, objective: Obj, exploration_preference: f32) -> Self {
        let vertices = search_space.into_iter().map(Rc::new).collect_vec();

        let mut this = Self {
            queue: BinaryHeap::new(),
            objective,
            max_value: f32::MIN,
            min_value: f32::MAX,
            best_point: Rc::new(SVector::zeros()),
            exploration_preference,
            phantom: PhantomData,
        };
        let values =
            VectorNp1::<N>::from_iterator(vertices.iter().map(|v| this.evaluate_point(v.clone())));
        this.enqueue_simplex(vertices, values, 1.);
        this
    }

    /// Give the currently known range of the objective within the search space.
    ///
    /// `max_value - min_value`.
    fn current_delta(&self) -> f32 {
        self.max_value - self.min_value
    }

    fn enqueue_simplex(
        &mut self,
        vertices: Vec<Rc<Vector<N>>>,
        values: VectorNp1<N>,
        content_fraction_of_root: f32,
    ) {
        self.queue.push(Simplex::new(
            vertices,
            values,
            content_fraction_of_root,
            self.exploration_preference,
            self.current_delta(),
        ));
    }

    /// Select the simplex that gives the most information for subdivision.
    fn get_next_simplex(&mut self) -> Simplex<N> {
        let mut candidate = self.queue.pop().expect("queue is never empty");
        loop {
            // Note that the acquisition value was computed based on the delta known at the time of
            // creation of the simplex. Thus, it may need to be updated.
            debug_assert!(
                self.current_delta() >= candidate.last_known_delta,
                "delta is monotonically increasing"
            );
            if self.current_delta() > candidate.last_known_delta {
                candidate.update_acquisition_value(self.current_delta());
                // Check to see if it has become (comparatively) worse due to the delta change.
                let mut next_best = self.queue.peek_mut().unwrap();
                if candidate.acquisition_value < next_best.acquisition_value {
                    // If so, put it back in the queue and try the next one.
                    mem::swap(&mut candidate, &mut next_best);
                    continue;
                }
            }
            return candidate;
        }
    }

    /// Evaluate the objective at the given point and update the known max/min values.
    fn evaluate_point(&mut self, point: Rc<Vector<N>>) -> f32 {
        let value = (self.objective)(&(*point).into());
        if value > self.max_value {
            self.max_value = value;
            self.best_point = point;
        } else if value < self.min_value {
            self.min_value = value;
        }
        value
    }

    /// Build all subdivisions of the `parent` simplex and enqueue them.
    ///
    /// (_i.e._, every simplex obtained by replacing one of the parent's vertices with the
    /// subdivision point.)
    fn construct_and_enqueue_subdivisions(
        &mut self,
        parent: &Simplex<N>,
        value_at_subdivision_point: f32,
    ) {
        for i in 0..Simplex::<N>::NUM_VERTICES {
            let mut vertices = parent.vertices.clone();
            let mut values = parent.values.clone();
            vertices[i] = parent.subdivision_point.clone();
            values[i] = value_at_subdivision_point;
            self.enqueue_simplex(
                vertices,
                values,
                parent.content_fraction_of_root * parent.content_fractions_of_subdivisions[i],
            );
        }
    }

    pub fn maximize(&mut self, iterations: usize) -> PointValue<N, P, f32> {
        for _ in 0..iterations {
            let target = self.get_next_simplex();
            let value_at_subdiv_pt = self.evaluate_point(target.subdivision_point.clone());
            self.construct_and_enqueue_subdivisions(&target, value_at_subdiv_pt);
        }
        PointValue((*self.best_point).into(), self.max_value)
    }
}

pub trait BoundingSimplex<const N: usize, P: IPoint<N>>: BoundingRegion<N, P> {
    fn bounding_simplex(&self) -> Vec<Vector<N>>;
}

impl<P, F> BoundingSimplex<3, P> for F
where
    P: IPoint<3>,
    F: Function<3, P> + BoundingRegion<3, P, Geometry = BallCenteredAtOrigin>,
{
    fn bounding_simplex(&self) -> Vec<Vector<3>> {
        const FRAC_1_3: f32 = 0.333_333_33;
        const SQRT_FRAC_2_9: f32 = 0.471_404_52;
        const SQRT_FRAC_2_3: f32 = 0.816_496_58;

        let radius = self.bounding_region().radius;

        let mut tetrahedron = vec![
            vector![0.942_809_04, 0., -FRAC_1_3],
            vector![-SQRT_FRAC_2_9, SQRT_FRAC_2_3, -FRAC_1_3],
            vector![-SQRT_FRAC_2_9, -SQRT_FRAC_2_3, -FRAC_1_3],
            vector![0., 0., 1.],
        ];
        for vert in &mut tetrahedron {
            vert.scale_mut(radius);
        }
        tetrahedron
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use na::{vector, Point1, Point2};

    use crate::geometry::storage::PointValue;
    use crate::numerics::optimization::simple_x::Simple;

    #[test]
    fn quadratic() {
        let quadratic = |p: &Point1<f32>| {
            let x = p.coords.x;
            -x * x + 2. * x + 1.
        };

        let mut simple = Simple::new(vec![vector![-5.], vector![5.]], quadratic, 0.15);
        let PointValue(pt, val) = simple.maximize(200);
        assert_relative_eq!(pt.x, 1.0, max_relative = 1E-3);
        assert_relative_eq!(val, 2.0, max_relative = 1E-6);
    }

    #[test]
    fn modulated_sinusoid() {
        let sinusoid = |p: &Point1<f32>| {
            let x = p.coords.x;
            (x / 5.0).cos() * (5.0 * x).sin()
        };

        let mut simple = Simple::new(vec![vector![-8.], vector![8.]], sinusoid, 0.15);
        let PointValue(pt, val) = simple.maximize(100);
        assert_relative_eq!(pt.x, 0.313_66, max_relative = 5E-3);
        assert_relative_eq!(val, 0.998_03, max_relative = 5E-5);
    }

    /// <https://github.com/chrisstroemel/Simple/blob/0d172eb504002caf8880e1f9a573e55b7b0fa423/README.md#usage>
    #[test]
    fn reference_example() {
        let objective =
            |p: &Point2<f32>| -((p.coords.x - 0.2).powi(2) + (p.coords.y - 0.3).powi(2)).sqrt();

        let mut simple = Simple::new(
            vec![vector![0.0, 0.0], vector![0.0, 1.0], vector![1.0, 0.0]],
            objective,
            0.05,
        );
        let PointValue(pt, val) = simple.maximize(30);
        assert_relative_eq!(pt.coords, vector![0.192_852_89, 0.295_910_33]);
        assert_relative_eq!(val, -0.008_234_477);
    }
}
