use std::borrow::Cow;

use super::{UNICODE_SUBSCRIPTS, UNICODE_SUPERSCRIPTS};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Format {
    Unicode,
    Html,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SupSubSegment {
    Normal(Cow<'static, str>),
    Superscript(Cow<'static, str>),
    Subscript(Cow<'static, str>),
}

impl SupSubSegment {
    pub fn format_with(&self, format: Format) -> Option<String> {
        match format {
            Format::Unicode => match self {
                Self::Normal(s) => Some(s.clone().into_owned()),
                Self::Superscript(s) => s.chars().map(|c| UNICODE_SUPERSCRIPTS.get(&c)).collect(),
                Self::Subscript(s) => s.chars().map(|c| UNICODE_SUBSCRIPTS.get(&c)).collect(),
            },
            Format::Html => Some(match self {
                Self::Normal(s) => s.clone().into_owned(),
                Self::Superscript(s) => format!("<sup>{s}</sup>"),
                Self::Subscript(s) => format!("<sub>{s}</sub>"),
            }),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SupSubString(Vec<SupSubSegment>);

impl SupSubString {
    pub fn new(segments: Vec<SupSubSegment>) -> Self {
        Self(segments)
    }
}

impl SupSubString {
    pub fn format_with(&self, format: Format) -> Option<String> {
        // Fixme: Try to reduce allocations?
        self.0
            .iter()
            .map(|segment| segment.format_with(format))
            .collect()
    }
}

#[macro_export]
macro_rules! sup_sub_string {
    (
        $(@[$($head:expr),+])?
        $str:literal
        $($tail:tt)*
    ) => {
        sup_sub_string!(
            @[
                $($($head,)+)?
                $crate::utils::sup_sub_string::SupSubSegment::Normal(
                    ::std::borrow::Cow::Borrowed($str)
                )
            ]
            $($tail)*
        )
    };
    (
        $(@[$($head:expr),+])?
        $case:ident ($inner:expr)
        $($tail:tt)*
    ) => {
        sup_sub_string!(
            @[
                $($($head,)+)?
                sup_sub_string!(@case $case) (
                    ::std::borrow::Cow::from($inner)
                )
            ]
            $($tail)*
        )
    };
    (@[$($seg:expr),+]) => {
        $crate::utils::sup_sub_string::SupSubString::new(
            ::std::vec![$($seg),+]
        )
    };
    (@case nrm) => {
        $crate::utils::sup_sub_string::SupSubSegment::Normal
    };
    (@case sup) => {
        $crate::utils::sup_sub_string::SupSubSegment::Superscript
    };
    (@case sub) => {
        $crate::utils::sup_sub_string::SupSubSegment::Subscript
    };
}

#[cfg(test)]
mod tests {
    use super::Format;

    #[test]
    fn sup_sub_string() {
        let all_normal = sup_sub_string!("hello" " " nrm("world".to_owned()));
        assert_eq!(
            all_normal.format_with(Format::Html),
            Some("hello world".to_owned())
        );
        assert_eq!(
            all_normal.format_with(Format::Unicode),
            Some("hello world".to_owned())
        );

        let sulfate = sup_sub_string!["SO" sub("4") sup("2-")];
        assert_eq!(
            sulfate.format_with(Format::Html),
            Some("SO<sub>4</sub><sup>2-</sup>".to_owned())
        );
        assert_eq!(
            sulfate.format_with(Format::Unicode),
            Some("SO₄²⁻".to_owned())
        );

        let x_n = sup_sub_string!("x" sup("n"));
        assert_eq!(
            x_n.format_with(Format::Html),
            Some("x<sup>n</sup>".to_owned())
        );
        assert_eq!(x_n.format_with(Format::Unicode), None);
    }
}
