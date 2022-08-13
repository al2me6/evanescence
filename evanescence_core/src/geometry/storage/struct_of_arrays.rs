use getset::Getters;

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
    vals: Vec<V>,
}

impl<const N: usize, V> Soa<N, V> {
    /// Return the inner vectors of `self`.
    pub fn decompose(self) -> ([Vec<f32>; N], Vec<V>) {
        (self.coords, self.vals)
    }
}

macro_rules! impl_iter_traits {
    ($($a:lifetime)?) => {
        impl<$($a,)? const N: usize, P, V> Extend<$(&$a)? PointValue<N, P, V>> for Soa<N, V>
        where
            P: IPoint<N>,
            V: Clone,
        {
            fn extend<T: IntoIterator<Item = $(&$a)? PointValue<N, P, V>>>(&mut self, iter: T) {
                for PointValue(pt, val) in iter {
                    for (v, coord) in itertools::zip_eq(self.coords.iter_mut(), pt.coordinates()) {
                        v.push(*coord);
                    }
                    self.vals.push(val.clone());
                }
            }
        }

        impl<$($a,)? const N: usize, P, V> FromIterator<$(&$a)? PointValue<N, P, V>> for Soa<N, V>
        where
            P: IPoint<N>,
            V: Clone,
        {
            fn from_iter<I: IntoIterator<Item = $(&$a)? PointValue<N, P, V>>>(iter: I) -> Self {
                let iter = iter.into_iter();
                let (lower, upper) = iter.size_hint();
                let len = upper.unwrap_or(lower);
                let mut this = Self {
                    coords: std::array::from_fn(|_| Vec::with_capacity(len)),
                    vals: Vec::with_capacity(len),
                };
                this.extend(iter);
                this
            }
        }
    };
}

impl_iter_traits!();
impl_iter_traits!('a);
