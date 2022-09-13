use std::borrow::Cow;

use super::{UNICODE_SUBSCRIPTS, UNICODE_SUPERSCRIPTS};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SupSubFormat {
    /// Format using the `<sup>` and `<sub>` tags. This is infalliable.
    Html,
    /// Format using Unicode superscripts and subscripts. Segments must contain only the characters
    /// `[0-9+\-=\(\)]`.
    Unicode,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SupSubSegment {
    Normal(Cow<'static, str>),
    Superscript(Cow<'static, str>),
    Subscript(Cow<'static, str>),
}

impl SupSubSegment {
    pub fn with_case_of(other: &Self, inner: impl Into<Cow<'static, str>>) -> Self {
        (match other {
            Self::Normal(_) => Self::Normal,
            Self::Superscript(_) => Self::Superscript,
            Self::Subscript(_) => Self::Subscript,
        })(inner.into())
    }

    pub fn inner(&self) -> &str {
        match self {
            Self::Normal(s) | Self::Superscript(s) | Self::Subscript(s) => s,
        }
    }

    pub fn inner_mut(&mut self) -> &mut String {
        match self {
            Self::Normal(s) | Self::Superscript(s) | Self::Subscript(s) => s.to_mut(),
        }
    }

    pub fn format(&self, format: SupSubFormat) -> Option<String> {
        match format {
            SupSubFormat::Unicode => match self {
                Self::Normal(s) => Some(s.clone().into_owned()),
                Self::Superscript(s) => s.chars().map(|c| UNICODE_SUPERSCRIPTS.get(&c)).collect(),
                Self::Subscript(s) => s.chars().map(|c| UNICODE_SUBSCRIPTS.get(&c)).collect(),
            },
            SupSubFormat::Html => Some(match self {
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

    pub fn push_nrm(&mut self, segment: impl Into<Cow<'static, str>>) {
        self.0.push(SupSubSegment::Normal(segment.into()));
    }

    pub fn push_sup(&mut self, segment: impl Into<Cow<'static, str>>) {
        self.0.push(SupSubSegment::Superscript(segment.into()));
    }

    pub fn push_sub(&mut self, segment: impl Into<Cow<'static, str>>) {
        self.0.push(SupSubSegment::Subscript(segment.into()));
    }

    pub fn is_empty(&self) -> bool {
        self.0.iter().all(|segment| segment.inner().is_empty())
    }

    pub fn format_with(
        &self,
        formatter: impl Fn(&SupSubSegment) -> Option<String>,
    ) -> Option<String> {
        // Fixme: Try to reduce allocations?
        self.0.iter().map(formatter).collect()
    }

    pub fn format(&self, format: SupSubFormat) -> Option<String> {
        self.format_with(|segment| segment.format(format))
    }

    /// Try to format all segments in the requested format, falling back to normal case for
    /// segments that cannot be formatted (due to unsupported characters).
    pub fn format_or_normal(&self, format: SupSubFormat) -> String {
        self.0
            .iter()
            .map(|seg| {
                seg.format(format).unwrap_or_else(|| {
                    SupSubSegment::Normal(Cow::Owned(seg.inner().to_owned()))
                        .format(format)
                        .unwrap()
                })
            })
            .collect()
    }
}

impl From<Vec<SupSubSegment>> for SupSubString {
    fn from(segments: Vec<SupSubSegment>) -> Self {
        Self(segments)
    }
}

impl IntoIterator for SupSubString {
    type IntoIter = <Vec<SupSubSegment> as IntoIterator>::IntoIter;
    type Item = SupSubSegment;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Extend<String> for SupSubString {
    fn extend<T: IntoIterator<Item = String>>(&mut self, iter: T) {
        self.0
            .extend(iter.into_iter().map(Cow::Owned).map(SupSubSegment::Normal));
    }
}

impl Extend<SupSubSegment> for SupSubString {
    fn extend<T: IntoIterator<Item = SupSubSegment>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}

impl Extend<SupSubString> for SupSubString {
    fn extend<T: IntoIterator<Item = SupSubString>>(&mut self, iter: T) {
        self.0
            .extend(iter.into_iter().flat_map(IntoIterator::into_iter));
    }
}

impl<I> FromIterator<I> for SupSubString
where
    Self: Extend<I>,
{
    fn from_iter<T: IntoIterator<Item = I>>(iter: T) -> Self {
        let iter = iter.into_iter();
        let (lower, upper) = iter.size_hint();
        let mut this = Self::new(Vec::with_capacity(upper.unwrap_or(lower)));
        this.extend(iter);
        this
    }
}

#[macro_export]
macro_rules! sup_sub_string {
    () => { $crate::utils::sup_sub_string::SupSubString::new(::std::vec![]) };
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
    use super::SupSubFormat;

    #[test]
    fn sup_sub_string() {
        let all_normal = sup_sub_string!["hello" " " nrm("world".to_owned())];
        assert_eq!(
            all_normal.format(SupSubFormat::Html),
            Some("hello world".to_owned())
        );
        assert_eq!(
            all_normal.format(SupSubFormat::Unicode),
            Some("hello world".to_owned())
        );

        let sulfate = sup_sub_string!["SO" sub("4") sup("2-")];
        assert_eq!(
            sulfate.format(SupSubFormat::Html),
            Some("SO<sub>4</sub><sup>2-</sup>".to_owned())
        );
        assert_eq!(
            sulfate.format(SupSubFormat::Unicode),
            Some("SO₄²⁻".to_owned())
        );

        let x_n = sup_sub_string!["x" sup("n")];
        assert_eq!(
            x_n.format(SupSubFormat::Html),
            Some("x<sup>n</sup>".to_owned())
        );
        assert_eq!(x_n.format(SupSubFormat::Unicode), None);
    }
}
