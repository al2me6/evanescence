use getset::Getters;

use crate::geometry::storage::{IPoint, PointValue};

/// Type storing a collection of [`PointValue`]s, where values in each dimension
/// (x, y, z, and value) are stored in a separate vector. Each index, across the four vectors,
/// corresponds to a single point and its associated value.
///
/// It may be thought of as the transpose of `Vec<PointValue<T>>`.
///
/// This type cannot be manually constructed and should instead be obtained from a
/// [`Vec<PointValue<T>>`] via conversion traits.
///
/// # Invariants
/// All four vectors must be the same length.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct ComponentForm3<T> {
    /// List of x values.
    xs: Vec<f32>,
    /// List of y values.
    ys: Vec<f32>,
    /// List of z values.
    zs: Vec<f32>,
    /// List of values evaluated at the corresponding (x, y, z) coordinate.
    vals: Vec<T>,
}

impl<T> ComponentForm3<T> {
    /// Decompose `self` into a four-tuple of its inner vectors, in the order (x, y, z, value).
    pub fn into_components(self) -> (Vec<f32>, Vec<f32>, Vec<f32>, Vec<T>) {
        (self.xs, self.ys, self.zs, self.vals)
    }
}

impl<P, V> FromIterator<PointValue<3, P, V>> for ComponentForm3<V>
where
    P: IPoint<3>,
    V: Clone,
{
    fn from_iter<I: IntoIterator<Item = PointValue<3, P, V>>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let (lower, upper) = iter.size_hint();
        let len = upper.unwrap_or(lower);
        let (mut xs, mut ys, mut zs, mut vals) = (
            Vec::with_capacity(len),
            Vec::with_capacity(len),
            Vec::with_capacity(len),
            Vec::with_capacity(len),
        );
        for PointValue(pt, val) in iter {
            let coords = pt.coordinates();
            xs.push(coords.x);
            ys.push(coords.y);
            zs.push(coords.z);
            vals.push(val.clone());
        }
        // INVARIANT: The four vectors, by nature, have the same length.
        ComponentForm3 { xs, ys, zs, vals }
    }
}

impl<'a, P, V> FromIterator<&'a PointValue<3, P, V>> for ComponentForm3<V>
where
    P: IPoint<3>,
    V: Clone,
{
    fn from_iter<I: IntoIterator<Item = &'a PointValue<3, P, V>>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let (lower, upper) = iter.size_hint();
        let len = upper.unwrap_or(lower);
        let (mut xs, mut ys, mut zs, mut vals) = (
            Vec::with_capacity(len),
            Vec::with_capacity(len),
            Vec::with_capacity(len),
            Vec::with_capacity(len),
        );
        for PointValue(pt, val) in iter {
            let coords = pt.coordinates();
            xs.push(coords.x);
            ys.push(coords.y);
            zs.push(coords.z);
            vals.push(val.clone());
        }
        // INVARIANT: The four vectors, by nature, have the same length.
        ComponentForm3 { xs, ys, zs, vals }
    }
}
