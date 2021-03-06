use std::fmt::LowerExp;

use instant::Instant;
use itertools::Itertools;
use log::{Level, Record};

pub(crate) fn capitalize_words<T: AsRef<str>>(source: T) -> String {
    let mut prev_is_word_separator = true;
    source
        .as_ref()
        .chars()
        .map(|mut c| {
            if prev_is_word_separator {
                c = c.to_ascii_uppercase();
            }
            prev_is_word_separator = " -".contains(c);
            c
        })
        .collect()
}

pub(crate) fn fmt_scientific_notation<T: LowerExp>(source: T, precision: usize) -> String {
    format!("{:.*e}</sup>", precision, source)
        .replace("-", "−") // "hyphen" -> "minus".
        .replace("e", " × 10<sup>")
}

/// Italicize the parts of an orbital name that should be italicized (i.e., the alpha characters).
/// It is probably wiser to outsource this kind of work to Latex...
pub(crate) fn fmt_orbital_name_html<T: AsRef<str>>(source: T) -> String {
    source
        .as_ref()
        .chars()
        .group_by(char::is_ascii_alphabetic)
        .into_iter()
        .map(|(is_letter, chars)| {
            let group = chars.collect::<String>();
            // HACK: In this context "sub" is an HTML tag.
            if is_letter && group != "sub" {
                format!("<i>{}</i>", group)
            } else {
                group
            }
        })
        .collect()
}

pub(crate) fn fmt_thousands_separated(n: usize) -> String {
    let string = n.to_string();
    // Comparing `len` directly is fine because the string should only contain ASCII (1-byte) chars.
    if string.len() <= 4 {
        return string;
    }
    Iterator::intersperse(string.chars().collect_vec().rchunks(3).rev(), &[' '])
        .flatten()
        .collect()
}

pub(crate) fn partial_max<I>(values: I) -> Option<<I as IntoIterator>::Item>
where
    I: IntoIterator,
    <I as IntoIterator>::Item: PartialOrd,
{
    values.into_iter().max_by(|a, b| a.partial_cmp(b).unwrap())
}

/// [Base16 Tomorrow Night](https://github.com/chriskempson/base16-tomorrow-scheme/blob/master/tomorrow-night.yaml)
/// colors.
pub(crate) mod b16_colors {
    pub(crate) const BASE: &[&str; 16] = &[
        "#1d1f21", // 00
        "#282a2e", // 01
        "#373b41", // 02
        "#969896", // 03
        "#b4b7b4", // 04
        "#c5c8c6", // 05
        "#e0e0e0", // 06
        "#ffffff", // 07
        "#cc6666", // 08
        "#de935f", // 09
        "#f0c674", // 0a
        "#b5bd68", // 0b
        "#8abeb7", // 0c
        "#81a2be", // 0d
        "#b294bb", // 0e
        "#a3685a", // 0f
    ];
    pub(crate) const BASE0102: &str = "#303338";
    pub(crate) const BASE0203: &str = "#676a6c";
    pub(crate) const BASE0304: &str = "#a5a8a5";
}

pub(crate) fn fire_resize_event() {
    web_sys::window()
        .unwrap()
        .dispatch_event(&web_sys::Event::new("resize").unwrap())
        .unwrap();
}

#[must_use = "timer is useless if dropped immediately"]
pub(crate) struct ScopeTimer {
    action_description: String,
    begin: Instant,
    file: &'static str,
    line: u32,
}

impl ScopeTimer {
    pub(crate) fn new(action_description: String, file: &'static str, line: u32) -> Self {
        Self {
            action_description,
            begin: Instant::now(),
            file,
            line,
        }
    }
}

impl Drop for ScopeTimer {
    fn drop(&mut self) {
        let time = self.begin.elapsed().as_millis();
        log::logger().log(
            &Record::builder()
                .args(format_args!("{}: {}ms", self.action_description, time))
                .level(Level::Info)
                .file(Some(self.file))
                .line(Some(self.line))
                .build(),
        );
    }
}

macro_rules! time_scope {
    ($($arg:tt)+) => {
        $crate::utils::ScopeTimer::new(format!($($arg)+), file!(), line!())
    };
}
