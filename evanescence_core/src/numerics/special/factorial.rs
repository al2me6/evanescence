use num::{one, zero, PrimInt};

pub trait Factorial {
    #[must_use]
    /// `x!`
    fn factorial(self) -> Self;
}

impl<T> Factorial for T
where
    T: PrimInt,
{
    #[inline]
    fn factorial(self) -> Self {
        if self < zero() {
            panic!("cannot compute factorial of negative value");
        }
        if self == zero() {
            return one();
        }
        num::range_inclusive(one(), self).fold(one(), |acc, n| acc * n)
    }
}

/// Compute the [double factorial](https://en.wikipedia.org/wiki/Double_factorial).
#[allow(clippy::module_name_repetitions)]
pub trait DoubleFactorial {
    /// `x!!`
    #[must_use]
    fn double_factorial(self) -> Self;
}

impl<T> DoubleFactorial for T
where
    T: PrimInt,
{
    #[inline]
    fn double_factorial(self) -> Self {
        if self < zero() {
            panic!("cannot compute double factorial of negative value");
        }
        if self == zero() || self == one() {
            return one();
        }
        let mut acc = self;
        let delta = T::one() + T::one();
        let mut mul = acc - delta;
        while mul >= delta {
            acc = acc * mul;
            mul = mul - delta;
        }
        acc
    }
}

#[cfg(test)]
mod tests {
    use super::{DoubleFactorial, Factorial};

    #[test]
    fn factorial() {
        assert_eq!(
            &[1, 1, 2, 6, 24, 120, 720, 5_040, 40_320, 362_880, 3_628_800],
            &(0_u32..=10).map(Factorial::factorial).collect::<Vec<_>>()[..]
        );
    }

    #[test]
    fn double_factorial() {
        assert_eq!(
            &[1, 1, 2, 3, 8, 15, 48, 105, 384, 945, 3840],
            &(0_u32..=10)
                .map(DoubleFactorial::double_factorial)
                .collect::<Vec<_>>()[..]
        );
    }
}
