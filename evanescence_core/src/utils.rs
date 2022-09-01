/// Verify that two iterables containing float values are approximately equal.
#[macro_export]
macro_rules! assert_iterable_approx_eq {
    ($lhs:expr, $rhs: expr $(, $opt:ident = $val:expr)* $(,)?) => {
        assert_iterable_approx_eq!(relative_eq, $lhs, $rhs $(, $opt = $val)*)
    };
    ($method:ident, $lhs:expr, $rhs: expr $(, $opt:ident = $val:expr)* $(,)?) => {{
        use itertools::Itertools;
        assert!(
            $lhs.iter()
                .zip_eq($rhs.iter())
                .all(|(l, r)| approx::$method!(l, r $(, $opt = $val)*)),
            "assertion failed: `(left ≈ right)` via {}\n\
                left: `{:?}`\n\
                right: `{:?}`",
            stringify!($method),
            $lhs,
            $rhs
        );
    }};
}

#[macro_use]
pub mod sup_sub_string;

use phf::phf_map;

pub const UNICODE_SUPERSCRIPTS: phf::Map<char, char> = phf_map! {
    '0' => '⁰',
    '1' => '¹',
    '2' => '²',
    '3' => '³',
    '4' => '⁴',
    '5' => '⁵',
    '6' => '⁶',
    '7' => '⁷',
    '8' => '⁸',
    '9' => '⁹',
    '+' => '⁺',
    '-' => '⁻',
    '=' => '⁼',
    '(' => '⁽',
    ')' => '⁾',
};

pub const UNICODE_SUBSCRIPTS: phf::Map<char, char> = phf_map! {
    '0' => '₀',
    '1' => '₁',
    '2' => '₂',
    '3' => '₃',
    '4' => '₄',
    '5' => '₅',
    '6' => '₆',
    '7' => '₇',
    '8' => '₈',
    '9' => '₉',
    '+' => '₊',
    '-' => '₋',
    '=' => '₌',
    '(' => '₍',
    ')' => '₎',
};
