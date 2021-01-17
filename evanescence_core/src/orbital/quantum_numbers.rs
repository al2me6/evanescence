//! Types for working with and validating quantum numbers.
use std::ops::{Range, RangeInclusive};

use getset::CopyGetters;

/// Type representing the quantum numbers `n`, `l`, and `m`.
///
/// # Safety
/// `Qn` must satisfy that `n > 0`, `n > l` and `l >= |m|`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Qn {
    /// The principal quantum number.
    n: u32,
    /// The azimuthal quantum number.
    l: u32,
    /// The magnetic quantum number.
    m: i32,
}

impl Qn {
    /// Create a new `Qn`, verifying that the passed values are valid. Returns `None` if that
    /// is not the case.
    pub const fn new(n: u32, l: u32, m: i32) -> Option<Self> {
        if n > l && l >= m.abs() as u32 {
            Some(Self { n, l, m })
        } else {
            None
        }
    }

    /// List all possible values of `l` for a given value of `n`.
    ///
    /// # Panics
    /// This function will panic if the passed value is zero.
    pub fn enumerate_l_for_n(n: u32) -> Range<u32> {
        assert!(n != 0);
        0..n
    }

    /// List all possible values of `m` for a given value of `l`.
    pub fn enumerate_m_for_l(l: u32) -> RangeInclusive<i32> {
        -(l as i32)..=(l as i32)
    }

    /// List all possible quantum number sets with `n` less than or equal to the value passed.
    ///
    /// # Panics
    /// This function will panic if the passed value is zero.
    pub fn enumerate_up_to_n(n: u32) -> impl Iterator<Item = Self> {
        assert!(n != 0);
        (1..=n).flat_map(|n| {
            Self::enumerate_l_for_n(n).flat_map(move |l| {
                Self::enumerate_m_for_l(l).map(move |m| Self::new(n, l, m).unwrap())
            })
        })
    }

    /// List all possible quantum number sets with both `n` and `l` less than or equal to
    /// the values passed.
    ///
    /// # Panics
    /// This function will panic if the passed value is zero.
    #[allow(clippy::filter_map)] // Stylistic.
    pub fn enumerate_up_to_n_l(n: u32, l: u32) -> impl Iterator<Item = Self> {
        assert!(n != 0);
        (1..=n).flat_map(move |n| {
            Self::enumerate_l_for_n(n)
                // Check if the value of l is within the limit requested.
                .filter(move |&possible_l| possible_l <= l)
                .flat_map(move |l| {
                    Self::enumerate_m_for_l(l).map(move |m| Self::new(n, l, m).unwrap())
                })
        })
    }

    /// Set `n`, the principal quantum number, clamping `l` and `m` as necessary.
    ///
    /// # Panics
    /// This function will panic if the passed value is zero.
    pub fn set_n_clamping(&mut self, n: u32) {
        assert!(n != 0);
        if self.l >= n {
            self.set_l_clamping(n - 1);
        }
        self.n = n;
    }

    /// Set `l`, the azimuthal quantum number, clamping `m` as necessary.
    ///
    /// # Panics
    /// This function will panic if the passed value `l` does not satisfy `self.n > l`.
    pub fn set_l_clamping(&mut self, l: u32) {
        assert!(self.n > l);
        if self.m.abs() as u32 > l {
            self.set_m(self.m.signum() * l as i32)
        }
        self.l = l;
    }

    /// Set `m`, the magnetic quantum number.
    ///
    /// # Panics
    /// The passed value `m` must satisfy `self.l >= |m|`. Otherwise, this function will panic.
    pub fn set_m(&mut self, m: i32) {
        assert!(self.l >= m.abs() as _);
        self.m = m;
    }
}

impl Default for Qn {
    fn default() -> Self {
        Self::new(1, 0, 0).unwrap()
    }
}

impl std::fmt::Display for Qn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{},{}", self.n, self.l, self.m)
    }
}

/// Type representing the quantum numbers `n` and `l`.
///
/// # Safety
/// `Nl` must satisfy that `n > 0` and `n > l`.
#[derive(Clone, Copy, Debug, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Nl {
    /// The principal quantum number.
    n: u32,
    /// The azimuthal quantum number.
    l: u32,
}

impl Nl {
    /// Create a new `Nm`, verifying that the passed values are valid. Returns `None`
    /// if that is not the case.
    pub const fn new(n: u32, l: u32) -> Option<Self> {
        if n != 0 && n > l {
            Some(Self { n, l })
        } else {
            None
        }
    }
}

impl From<Qn> for Nl {
    fn from(Qn { n, l, m: _ }: Qn) -> Self {
        Self { n, l }
    }
}

/// Type representing the quantum numbers `l` and `m`.
///
/// # Safety
/// `Lm` must satisfy that `l >= |m|`.
#[derive(Clone, Copy, Debug, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Lm {
    /// The azimuthal quantum number.
    l: u32,
    /// The magnetic quantum number.
    m: i32,
}

impl Lm {
    /// Create a new `Lm`, verifying that the passed values are valid. Returns `None`
    /// if that is not the case.
    pub const fn new(l: u32, m: i32) -> Option<Self> {
        if l >= m.abs() as u32 {
            Some(Self { l, m })
        } else {
            None
        }
    }
}

impl From<Qn> for Lm {
    fn from(Qn { n: _, l, m }: Qn) -> Self {
        Self { l, m }
    }
}

#[cfg(test)]
mod tests {
    use super::Qn;

    macro_rules! test_invalid {
        ($($fn:ident, $n:literal, $l:literal, $m:literal);+ $(;)?) => {
            $(
                #[test]
                #[should_panic]
                fn $fn() {
                    Qn::new($n, $l, $m).unwrap();
                }
            )+
        };
    }

    test_invalid!(
        test_000, 0, 0, 0;
        test_21n2, 2, 1, -2;
        test_253, 2, 5, 3;
        test_443, 4, 4, 3;
    );

    #[test]
    fn test_clamping_setters() {
        let mut qn = Qn::new(5, 4, -3).unwrap();
        qn.set_n_clamping(3);
        assert_eq!(Qn::new(3, 2, -2).unwrap(), qn);
        qn.set_l_clamping(0);
        assert_eq!(Qn::new(3, 0, 0).unwrap(), qn);
        qn = Qn::new(4, 2, 1).unwrap();
        qn.set_n_clamping(1);
        assert_eq!(Qn::new(1, 0, 0).unwrap(), qn);
    }

    #[test]
    fn test_m_setter() {
        let mut qn = Qn::new(5, 4, -3).unwrap();
        qn.set_m(2);
        assert_eq!(Qn::new(5, 4, 2).unwrap(), qn);
        qn.set_m(-4);
        assert_eq!(Qn::new(5, 4, -4).unwrap(), qn);
    }

    #[test]
    #[should_panic(expected = "assertion failed: n != 0")]
    fn test_invalid_n_setter() {
        let mut qn = Qn::new(5, 4, -3).unwrap();
        qn.set_n_clamping(0);
    }

    #[test]
    #[should_panic(expected = "assertion failed: n != 0")]
    fn test_invalid_enumerate() {
        let _ = Qn::enumerate_up_to_n(0);
    }

    #[test]
    #[should_panic(expected = "assertion failed: self.n > l")]
    fn test_invalid_l_setter() {
        let mut qn = Qn::new(5, 4, -3).unwrap();
        qn.set_l_clamping(5);
    }

    #[test]
    #[should_panic(expected = "assertion failed: self.l >= m.abs() as _")]
    fn test_invalid_m_setter() {
        let mut qn = Qn::new(5, 4, -3).unwrap();
        qn.set_m(-5);
    }

    #[test]
    fn test_enumerate_l_m() {
        assert_eq!(vec![0], Qn::enumerate_l_for_n(1).collect::<Vec<_>>());
        assert_eq!(vec![0, 1, 2], Qn::enumerate_l_for_n(3).collect::<Vec<_>>());
        assert_eq!(vec![0], Qn::enumerate_m_for_l(0).collect::<Vec<_>>());
        assert_eq!(
            vec![-2, -1, 0, 1, 2],
            Qn::enumerate_m_for_l(2).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_enumerate_qn() {
        let expected = &[
            Qn::new(1, 0, 0),
            Qn::new(2, 0, 0),
            Qn::new(2, 1, -1),
            Qn::new(2, 1, 0),
            Qn::new(2, 1, 1),
            Qn::new(3, 0, 0),
            Qn::new(3, 1, -1),
            Qn::new(3, 1, 0),
            Qn::new(3, 1, 1),
            // ^^ There are 9 quantum numbers through n=3, l=2. ^^
            Qn::new(3, 2, -2),
            Qn::new(3, 2, -1),
            Qn::new(3, 2, 0),
            Qn::new(3, 2, 1),
            Qn::new(3, 2, 2),
        ];
        for (exp, test) in expected.iter().zip(Qn::enumerate_up_to_n(3)) {
            assert_eq!(exp, &Some(test));
        }
        for (exp, test) in expected[..9].iter().zip(Qn::enumerate_up_to_n_l(3, 1)) {
            assert_eq!(exp, &Some(test));
        }
    }
}
