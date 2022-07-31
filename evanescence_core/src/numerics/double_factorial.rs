/// Compute the [double factorial](https://en.wikipedia.org/wiki/Double_factorial).
pub trait DoubleFactorial {
    /// `x!!`
    #[must_use]
    fn double_factorial(self) -> Self;
}

macro_rules! impl_double_factorial {
    ($($T:ty),+) => {
        $(impl DoubleFactorial for $T {
            #[inline]
            fn double_factorial(self) -> Self {
                if self <= 1 {
                    return 1;
                }
                let mut acc = self;
                let delta = 2;
                let mut mul = acc - delta;
                while mul >= delta {
                    acc *= mul;
                    mul -= delta;
                }
                acc
            }
        })+
    }
}
impl_double_factorial!(u8, u16, u32, u64, usize);

#[cfg(test)]
mod tests {
    use super::DoubleFactorial;

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
