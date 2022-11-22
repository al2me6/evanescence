use std::slice::SliceIndex;

use getset::Getters;
use na::SVector;

use crate::geometry::storage::{IPoint, PointValue};

/// Type storing a collection of [`PointValue<N, _, V>`]s as a struct-of-arrays.
///
/// This type cannot be manually constructed and should instead be obtained from an iterator of
/// `PointValue`s via conversion traits.
///
/// # Invariants
/// All vectors must be the same length.

// FIXME: Serde can't handle arbitrary-sized arrays...
// #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct Soa<const N: usize, V> {
    coords: [Vec<f32>; N],
    values: Vec<V>,
}

#[derive(Debug, PartialEq)]
pub struct SoaSlice<'a, const N: usize, V> {
    coords: [&'a [f32]; N],
    values: &'a [V],
}

impl<const N: usize, V> Soa<N, V> {
    pub fn new() -> Self {
        Self {
            coords: std::array::from_fn(|_| Vec::new()),
            values: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            coords: std::array::from_fn(|_| Vec::with_capacity(capacity)),
            values: Vec::with_capacity(capacity),
        }
    }

    pub fn len(&self) -> usize {
        debug_assert!(self.coords.iter().all(|c| c.len() == self.values.len()));
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn slice<I>(&self, idx: I) -> SoaSlice<'_, N, V>
    where
        I: SliceIndex<[f32], Output = [f32]> + SliceIndex<[V], Output = [V]> + Clone,
    {
        self.get(idx).expect("index out of bounds")
    }

    #[allow(clippy::needless_pass_by_value)] // Symmetry w/ `slice`.
    pub fn get<I>(&self, idx: I) -> Option<SoaSlice<'_, N, V>>
    where
        I: SliceIndex<[f32], Output = [f32]> + SliceIndex<[V], Output = [V]> + Clone,
    {
        let values = self.values.get(idx.clone())?;
        Some(SoaSlice {
            coords: std::array::from_fn(|i| &self.coords[i][idx.clone()]),
            values,
        })
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.values.iter_mut()
    }

    /// Return the inner vectors of `self`.
    pub fn into_components(self) -> ([Vec<f32>; N], Vec<V>) {
        (self.coords, self.values)
    }
}

impl<const N: usize, V> Default for Soa<N, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize, V> Extend<(SVector<f32, N>, V)> for Soa<N, V> {
    fn extend<T: IntoIterator<Item = (SVector<f32, N>, V)>>(&mut self, iter: T) {
        for (coords, val) in iter {
            for (self_coord, c) in itertools::zip_eq(self.coords.iter_mut(), &coords) {
                self_coord.push(*c);
            }
            self.values.push(val);
        }
    }
}

impl<const N: usize, V> FromIterator<(SVector<f32, N>, V)> for Soa<N, V> {
    fn from_iter<I: IntoIterator<Item = (SVector<f32, N>, V)>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let (lower, upper) = iter.size_hint();
        let len = upper.unwrap_or(lower);
        let mut this = Self::with_capacity(len);
        this.extend(iter);
        this
    }
}

impl<const N: usize, P: IPoint<N>, V> Extend<PointValue<N, P, V>> for Soa<N, V> {
    fn extend<T: IntoIterator<Item = PointValue<N, P, V>>>(&mut self, iter: T) {
        self.extend(iter.into_iter().map(PointValue::into_raw));
    }
}

impl<const N: usize, P: IPoint<N>, V> FromIterator<PointValue<N, P, V>> for Soa<N, V> {
    fn from_iter<I: IntoIterator<Item = PointValue<N, P, V>>>(iter: I) -> Self {
        iter.into_iter().map(PointValue::into_raw).collect()
    }
}

impl<'a, const N: usize, V> SoaSlice<'a, N, V> {
    pub fn coords(&self) -> [&'a [f32]; N] {
        self.coords
    }

    pub fn values(&self) -> &'a [V] {
        self.values
    }
}

impl<'a, const N: usize, V> SoaSlice<'a, N, V>
where
    V: Clone,
{
    pub fn cloned(&self) -> Soa<N, V> {
        Soa {
            coords: self.coords.map(ToOwned::to_owned),
            values: self.values.to_owned(),
        }
    }
}

/// Conversion into a struct-of-arrays.
pub trait IntoSoa<const N: usize> {
    type Value: Clone;

    /// Convert a collection of [`PointValue`]s to a `Soa`.
    fn into_soa(self) -> Soa<N, Self::Value>;

    /// Convert to a `Soa` and then immediately into its components.
    fn into_soa_components(self) -> ([Vec<f32>; N], Vec<Self::Value>)
    where
        Self: Sized,
    {
        self.into_soa().into_components()
    }
}

impl<const N: usize, P, V, I> IntoSoa<N> for I
where
    P: IPoint<N>,
    V: Clone,
    I: IntoIterator<Item = PointValue<N, P, V>>,
{
    type Value = V;

    fn into_soa(self) -> Soa<N, Self::Value> {
        self.into_iter().collect()
    }
}
