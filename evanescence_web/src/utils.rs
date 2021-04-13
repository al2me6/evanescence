use std::fmt::LowerExp;

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
