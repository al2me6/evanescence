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
