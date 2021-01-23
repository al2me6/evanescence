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
