pub(crate) fn capitalize_words(source: &str) -> String {
    let mut prev_is_space = true;
    source
        .chars()
        .map(|mut c| {
            if prev_is_space {
                c = c.to_ascii_uppercase();
            }
            prev_is_space = " -".contains(c);
            c
        })
        .collect()
}

pub(crate) fn min_max<'a, I: Iterator<Item = &'a f32>>(values: I) -> (f32, f32) {
    values.fold((0.0_f32, 0.0_f32), |(curr_min, curr_max), &v| {
        (curr_min.min(v), curr_max.max(v))
    })
}

pub(crate) fn abs_max<'a, I: Iterator<Item = &'a f32>>(values: I) -> f32 {
    let (min, max) = min_max(values);
    max.max(min.abs())
}

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
    pub(crate) const BASE0203: &str = "#676a6c";
    pub(crate) const BASE0304: &str = "#a5a8a5";
}

pub(crate) fn fire_resize_event() {
    web_sys::window()
        .unwrap()
        .dispatch_event(&web_sys::Event::new("resize").unwrap())
        .unwrap();
}
