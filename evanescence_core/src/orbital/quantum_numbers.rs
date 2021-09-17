//! Types for working with and validating quantum numbers.
use std::ops::{Range, RangeInclusive};

use getset::CopyGetters;
use thiserror::Error;

/// Error type describing an invalid [`Qn`].
#[derive(PartialEq, Eq, Debug, Error)]
pub enum InvalidQnError {
    /// `n` is zero.
    #[error("must satisfy n > 0")]
    N,
    /// `l` is too large.
    #[error("must satisfy n > l; got n={n}, l={l}")]
    L { n: u32, l: u32 },
    /// `|m|` is too large.
    #[error("must satisfy l >= |m|; got l={l}, m={m}")]
    M { l: u32, m: i32 },
}

type Result<T, E = InvalidQnError> = std::result::Result<T, E>;

/// Type representing the quantum numbers `n`, `l`, and `m`.
///
/// # Invariants
/// `Qn` must satisfy that `n > 0`, `n > l` and `l >= |m|`.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    /// Create a new `Qn`, verifying that the passed values are valid.
    ///
    /// # Errors
    /// This function will return [`Err`] if [`Qn`]'s invariants are not satisfied.
    pub const fn new(n: u32, l: u32, m: i32) -> Result<Self> {
        if n == 0 {
            return Err(InvalidQnError::N);
        }
        if n <= l {
            return Err(InvalidQnError::L { n, l });
        }
        if l < m.unsigned_abs() {
            return Err(InvalidQnError::M { l, m });
        }
        Ok(Self { n, l, m })
    }

    /// List all possible values of `l` for a given value of `n`.
    ///
    /// # Errors
    /// This function will return an [`Err`] if the passed value of n is zero.
    pub fn enumerate_l_for_n(n: u32) -> Result<Range<u32>> {
        if n == 0 {
            return Err(InvalidQnError::N);
        }
        Ok(0..n)
    }

    /// List all possible values of `m` for a given value of `l`.
    pub fn enumerate_m_for_l(l: u32) -> RangeInclusive<i32> {
        -(l as i32)..=(l as i32)
    }

    /// List all possible quantum number sets with `n` less than or equal to the value passed.
    ///
    /// # Errors
    /// This function will return an [`Err`] if the passed value of n is zero.
    pub fn enumerate_up_to_n(n: u32) -> Result<impl Iterator<Item = Self>> {
        if n == 0 {
            return Err(InvalidQnError::N);
        }
        Ok((1..=n).flat_map(|n| {
            Self::enumerate_l_for_n(n)
                .expect("n is nonzero by construction")
                .flat_map(move |l| {
                    Self::enumerate_m_for_l(l)
                        .map(move |m| Self::new(n, l, m).expect("generated `Qn` is invalid"))
                })
        }))
    }

    /// List all possible quantum number sets with both `n` and `l` less than or equal to
    /// the values passed.
    ///
    /// # Errors
    /// This function will return an [`Err`] if the passed value of n is zero.
    #[allow(clippy::manual_filter_map)] // Stylistic.
    pub fn enumerate_up_to_n_l(n: u32, l: u32) -> Result<impl Iterator<Item = Self>> {
        if n == 0 {
            return Err(InvalidQnError::N);
        }
        Ok((1..=n).flat_map(move |n| {
            Self::enumerate_l_for_n(n)
                .expect("n is nonzero by construction")
                // Check if the value of l is within the limit requested.
                .filter(move |&possible_l| possible_l <= l)
                .flat_map(move |l| {
                    Self::enumerate_m_for_l(l)
                        .map(move |m| Self::new(n, l, m).expect("generated `Qn` is invalid"))
                })
        }))
    }

    /// Set `n`, the principal quantum number, clamping `l` and `m` as necessary.
    ///
    /// # Errors
    /// This function will return an [`Err`] if the passed value of n is zero.
    pub fn set_n_clamping(&mut self, n: u32) -> Result<()> {
        if n == 0 {
            return Err(InvalidQnError::N);
        }
        if self.l >= n {
            self.set_l_clamping(n - 1)?;
        }
        self.n = n;
        Ok(())
    }

    /// Set `l`, the azimuthal quantum number, clamping `m` as necessary.
    ///
    /// # Errors
    /// This function will return an [`Err`] if `l` does not satisfy `self.n > l`.
    pub fn set_l_clamping(&mut self, l: u32) -> Result<()> {
        if self.n <= l {
            return Err(InvalidQnError::L { n: self.n, l });
        }
        if self.m.unsigned_abs() > l {
            self.set_m(self.m.signum() * l as i32)?;
        }
        self.l = l;
        Ok(())
    }

    /// Set `m`, the magnetic quantum number.
    ///
    /// # Errors
    /// This function will return an [`Err`] if `m` does not satisfy `self.l >= |m|`.
    pub fn set_m(&mut self, m: i32) -> Result<()> {
        if self.l < m.unsigned_abs() {
            return Err(InvalidQnError::M { l: self.l, m });
        }
        self.m = m;
        Ok(())
    }
}

impl Default for Qn {
    fn default() -> Self {
        // INVARIANT: This is a valid quantum number.
        Self { n: 1, l: 0, m: 0 }
    }
}

impl std::fmt::Display for Qn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ψ{}{}{}", self.n, self.l, self.m)
    }
}

impl From<&Qn> for Qn {
    fn from(qn: &Qn) -> Self {
        *qn
    }
}

/// Type representing the quantum numbers `n` and `l`.
///
/// # Invariants
/// `Nl` must satisfy that `n > 0` and `n > l`.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Nl {
    /// The principal quantum number.
    n: u32,
    /// The azimuthal quantum number.
    l: u32,
}

impl Nl {
    /// Create a new `Nm`, verifying that the passed values are valid.
    ///
    /// # Errors
    /// This function will return [`Err`] if [`Nl`]'s invariants are not satisfied.
    pub const fn new(n: u32, l: u32) -> Result<Self> {
        if n == 0 {
            return Err(InvalidQnError::N);
        }
        if n <= l {
            return Err(InvalidQnError::L { n, l });
        }
        Ok(Self { n, l })
    }
}

impl From<Qn> for Nl {
    fn from(Qn { n, l, m: _ }: Qn) -> Self {
        // INVARIANTS: Assume that the passed `Qn` is valid.
        Self { n, l }
    }
}

impl From<&Qn> for Nl {
    fn from(&Qn { n, l, m: _ }: &Qn) -> Self {
        // INVARIANTS: Assume that the passed `Qn` is valid.
        Self { n, l }
    }
}

/// Type representing the quantum numbers `l` and `m`.
///
/// # Invariants
/// `Lm` must satisfy that `l >= |m|`.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Lm {
    /// The azimuthal quantum number.
    l: u32,
    /// The magnetic quantum number.
    m: i32,
}

impl Lm {
    /// Create a new `Lm`, verifying that the passed values are valid.
    ///
    /// # Errors
    /// This function will return [`Err`] if [`Lm`]'s invariants are not satisfied.
    pub const fn new(l: u32, m: i32) -> Result<Self> {
        if l < m.unsigned_abs() {
            return Err(InvalidQnError::M { l, m });
        }
        Ok(Self { l, m })
    }
}

impl From<Qn> for Lm {
    fn from(Qn { n: _, l, m }: Qn) -> Self {
        // INVARIANTS: Assume that the passed `Qn` is valid.
        Self { l, m }
    }
}

impl From<&Qn> for Lm {
    fn from(&Qn { n: _, l, m }: &Qn) -> Self {
        // INVARIANTS: Assume that the passed `Qn` is valid.
        Self { l, m }
    }
}

#[cfg(test)]
mod tests {
    use super::{InvalidQnError, Qn};

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
        qn.set_n_clamping(3).unwrap();
        assert_eq!(Qn::new(3, 2, -2).unwrap(), qn);
        qn.set_l_clamping(0).unwrap();
        assert_eq!(Qn::new(3, 0, 0).unwrap(), qn);
        qn = Qn::new(4, 2, 1).unwrap();
        qn.set_n_clamping(1).unwrap();
        assert_eq!(Qn::new(1, 0, 0).unwrap(), qn);
    }

    #[test]
    fn test_m_setter() {
        let mut qn = Qn::new(5, 4, -3).unwrap();
        qn.set_m(2).unwrap();
        assert_eq!(Qn::new(5, 4, 2).unwrap(), qn);
        qn.set_m(-4).unwrap();
        assert_eq!(Qn::new(5, 4, -4).unwrap(), qn);
    }

    #[test]
    fn test_invalid_n_setter() {
        let mut qn = Qn::new(5, 4, -3).unwrap();
        assert_eq!(Err(InvalidQnError::N), qn.set_n_clamping(0));
    }

    #[test]
    fn test_invalid_enumerate() {
        assert!(Qn::enumerate_up_to_n(0).is_err());
    }

    #[test]
    fn test_invalid_l_setter() {
        let mut qn = Qn::new(5, 4, -3).unwrap();
        assert_eq!(Err(InvalidQnError::L { n: 5, l: 5 }), qn.set_l_clamping(5));
    }

    #[test]
    fn test_invalid_m_setter() {
        let mut qn = Qn::new(5, 4, -3).unwrap();
        assert_eq!(Err(InvalidQnError::M { l: 4, m: 5 }), qn.set_m(5));
    }

    #[test]
    fn test_enumerate_l_m() {
        assert_eq!(
            vec![0],
            Qn::enumerate_l_for_n(1).unwrap().collect::<Vec<_>>()
        );
        assert_eq!(
            vec![0, 1, 2],
            Qn::enumerate_l_for_n(3).unwrap().collect::<Vec<_>>()
        );
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
        for (exp, test) in expected.iter().zip(Qn::enumerate_up_to_n(3).unwrap()) {
            assert_eq!(exp, &Ok(test));
        }
        for (exp, test) in expected[..9]
            .iter()
            .zip(Qn::enumerate_up_to_n_l(3, 1).unwrap())
        {
            assert_eq!(exp, &Ok(test));
        }
    }
}
