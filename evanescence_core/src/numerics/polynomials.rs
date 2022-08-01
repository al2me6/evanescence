use std::ops;

use itertools::{EitherOrBoth, Itertools};
pub use smallvec::smallvec;
use smallvec::SmallVec;

pub type PolynomialStorage = SmallVec<[f32; 5]>; // Up to degree 4 inline.

#[derive(Clone, PartialEq, Debug)]
pub struct Polynomial(PolynomialStorage);

impl Polynomial {
    pub fn new(coefficients: PolynomialStorage) -> Self {
        let mut ret = Self(coefficients);
        ret.canonicalize();
        ret
    }

    pub fn zero() -> Self {
        Self(smallvec![0.])
    }

    pub fn zero_with_degree_capacity(n: usize) -> Self {
        let mut coeffs = SmallVec::with_capacity(n);
        coeffs.push(0.);
        Self(coeffs)
    }

    pub fn coefficients(&self) -> &[f32] {
        &self.0
    }

    pub fn degree(&self) -> usize {
        debug_assert!(self.is_canonical(), "got coefficients {:?}", self.0);
        self.last_nonzero_coefficient_index()
            .expect("coefficients were empty")
    }

    fn last_nonzero_coefficient_index(&self) -> Option<usize> {
        #[allow(clippy::float_cmp)]
        self.0.iter().rposition(|a_i| a_i != &0.)
    }

    fn is_canonical(&self) -> bool {
        !self.0.is_empty() && self.last_nonzero_coefficient_index() == Some(self.0.len() - 1)
    }

    fn canonicalize(&mut self) {
        if let Some(last_nonzero_idx) = self.last_nonzero_coefficient_index() {
            self.0.truncate(last_nonzero_idx + 1);
        } else if self.0.is_empty() {
            self.0.push(0.);
        }
    }
}

#[macro_export]
macro_rules! polynomial {
    ($($a_i:expr),+ $(,)?) => {
        $crate::numerics::polynomials::Polynomial::new(
            $crate::numerics::polynomials::smallvec![$($a_i),+])
    }
}

impl Polynomial {
    #[inline]
    pub fn evaluate_horner(&self, x: f32) -> f32 {
        self.iter().rev().fold(0., |res, &a_i| res * x + a_i)
    }
}

impl Polynomial {
    pub fn get(&self, i: usize) -> f32 {
        if i > self.degree() {
            return 0.;
        }
        self[i]
    }

    fn get_mut(&mut self, i: usize) -> &mut f32 {
        // Check len instead of degree, as this function may be called in a noncanonical state.
        if i >= self.0.len() {
            self.0.extend(itertools::repeat_n(0., i - self.0.len() + 1));
        }
        &mut self[i]
    }

    pub fn set(&mut self, i: usize, a_i: f32) {
        if i > self.degree() && a_i == 0. {
            return;
        }
        *self.get_mut(i) = a_i;
        self.canonicalize();
    }
}

impl Polynomial {
    pub fn iter(&self) -> <&Self as IntoIterator>::IntoIter {
        self.into_iter()
    }
}

impl IntoIterator for Polynomial {
    type IntoIter = <PolynomialStorage as IntoIterator>::IntoIter;
    type Item = f32;

    fn into_iter(self) -> Self::IntoIter {
        debug_assert!(self.is_canonical());
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Polynomial {
    type IntoIter = <&'a PolynomialStorage as IntoIterator>::IntoIter;
    type Item = &'a f32;

    fn into_iter(self) -> Self::IntoIter {
        debug_assert!(self.is_canonical());
        self.0.iter()
    }
}

impl FromIterator<f32> for Polynomial {
    fn from_iter<T: IntoIterator<Item = f32>>(iter: T) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

impl<'a> FromIterator<&'a f32> for Polynomial {
    fn from_iter<T: IntoIterator<Item = &'a f32>>(iter: T) -> Self {
        Self::new(iter.into_iter().copied().collect())
    }
}

impl ops::Index<usize> for Polynomial {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

impl ops::IndexMut<usize> for Polynomial {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

macro_rules! impl_add_sub_mul {
    ($Receiver:ty, $Rhs:ty) => {
        impl ops::Add<$Rhs> for $Receiver {
            type Output = Polynomial;

            fn add(self, rhs: $Rhs) -> Self::Output {
                self.iter()
                    .zip_longest(rhs.iter())
                    .map(|pair| match pair {
                        EitherOrBoth::Both(a_i, b_i) => a_i + b_i,
                        EitherOrBoth::Left(a_i) | EitherOrBoth::Right(a_i) => *a_i,
                    })
                    .collect()
            }
        }

        impl ops::Sub<$Rhs> for $Receiver {
            type Output = Polynomial;

            fn sub(self, rhs: $Rhs) -> Self::Output {
                self.iter()
                    .zip_longest(rhs.iter())
                    .map(|pair| match pair {
                        EitherOrBoth::Both(a_i, b_i) => a_i - b_i,
                        EitherOrBoth::Left(a_i) => *a_i,
                        EitherOrBoth::Right(b_i) => -b_i,
                    })
                    .collect()
            }
        }

        impl ops::Mul<$Rhs> for $Receiver {
            type Output = Polynomial;

            fn mul(self, rhs: $Rhs) -> Self::Output {
                let mut out = Polynomial::zero_with_degree_capacity(self.degree() + rhs.degree());
                for (i, a_i) in self.iter().enumerate() {
                    for (j, b_j) in rhs.iter().enumerate() {
                        *out.get_mut(i + j) += a_i * b_j;
                    }
                }
                debug_assert!(out.is_canonical());
                out
            }
        }
    };
}

impl_add_sub_mul!(Polynomial, Polynomial);
impl_add_sub_mul!(&Polynomial, Polynomial);
impl_add_sub_mul!(Polynomial, &Polynomial);
impl_add_sub_mul!(&Polynomial, &Polynomial);

macro_rules! impl_mul_div {
    ($Receiver:ty, $Rhs:ty) => {
        impl ops::Mul<$Rhs> for $Receiver {
            type Output = Polynomial;

            fn mul(self, rhs: $Rhs) -> Self::Output {
                self.iter().map(|a_i| a_i * rhs).collect()
            }
        }

        impl ops::Div<$Rhs> for $Receiver {
            type Output = Polynomial;

            fn div(self, rhs: $Rhs) -> Self::Output {
                self.iter().map(|a_i| a_i / rhs).collect()
            }
        }
    };
}

impl_mul_div!(Polynomial, f32);
impl_mul_div!(Polynomial, &f32);
impl_mul_div!(&Polynomial, f32);
impl_mul_div!(&Polynomial, &f32);

macro_rules! impl_add_sub_assign {
    ($Rhs:ty) => {
        impl ops::AddAssign<$Rhs> for Polynomial {
            fn add_assign(&mut self, rhs: $Rhs) {
                // Reverse to prevent excessive allocation.
                for (i, a_i) in rhs.iter().enumerate().rev() {
                    *self.get_mut(i) += a_i;
                }
                self.canonicalize();
            }
        }

        impl ops::SubAssign<$Rhs> for Polynomial {
            fn sub_assign(&mut self, rhs: $Rhs) {
                // Reverse to prevent excessive allocation.
                for (i, a_i) in rhs.iter().enumerate().rev() {
                    *self.get_mut(i) -= a_i;
                }
                self.canonicalize();
            }
        }
    };
}

impl_add_sub_assign!(Polynomial);
impl_add_sub_assign!(&Polynomial);

macro_rules! impl_mul_div_assign {
    ($Rhs:ty) => {
        impl ops::MulAssign<$Rhs> for Polynomial {
            fn mul_assign(&mut self, rhs: $Rhs) {
                self.0.iter_mut().for_each(|a_i| *a_i *= rhs);
                self.canonicalize();
            }
        }

        impl ops::DivAssign<$Rhs> for Polynomial {
            fn div_assign(&mut self, rhs: $Rhs) {
                self.0.iter_mut().for_each(|a_i| *a_i /= rhs);
                self.canonicalize();
            }
        }
    };
}

impl_mul_div_assign!(f32);
impl_mul_div_assign!(&f32);

#[cfg(test)]
mod tests {
    use approx::assert_ulps_eq;

    #[test]
    fn construction() {
        let a = polynomial![1., 0., 2., 3., 0., 0.];
        assert_eq!(a.degree(), 3);
        assert_iterable_approx_eq!(&a, &[1., 0., 2., 3.]);
    }

    #[test]
    fn ops() {
        let mut a = polynomial![0., 1., 2.];
        assert_eq!(a.degree(), 2);
        assert_iterable_approx_eq!(&a, &[0., 1., 2.]);
        a.set(1, -3.);
        assert_iterable_approx_eq!(&a, &[0., -3., 2.]);
        a.set(5, 5.);
        assert_eq!(a.degree(), 5);
        assert_iterable_approx_eq!(&a, &[0., -3., 2., 0., 0., 5.]);
        a.set(3, 0.5);
        a.set(5, 0.);
        assert_eq!(a.degree(), 3);
        assert_iterable_approx_eq!(&a, &[0., -3., 2., 0.5]);

        let mut b = polynomial![1.];
        b += polynomial![0., 0., 2.];
        assert_iterable_approx_eq!(&b, &[1., 0., 2.]);
        b /= 2.;
        assert_iterable_approx_eq!(&b, &[0.5, 0., 1.]);
        assert_iterable_approx_eq!(&b - polynomial![0., 1.], &[0.5, -1., 1.]);
        assert_iterable_approx_eq!(&b * 2., &[1., 0., 2.]);
        b -= polynomial![0., 0., 1.];
        assert_eq!(b.degree(), 0);
        assert_iterable_approx_eq!(&b, &[0.5]);
    }

    #[test]
    fn horner() {
        assert_ulps_eq!(
            polynomial![1.0, -3.5, 4.2, -0.3].evaluate_horner(-5.5),
            197.212_5,
            max_ulps = 1,
        );
        assert_ulps_eq!(
            polynomial![5.0, -4.6, 2.7, -18., -3.].evaluate_horner(9.3),
            -36_724.243_3,
            max_ulps = 1,
        );
    }
}
