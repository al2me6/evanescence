use std::env;
use std::error::Error;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

use pulldown_cmark::{
    html as html_renderer, CowStr, Event, HeadingLevel, LinkType, Options, Parser, Tag,
};

const INPUT: &str = "res/help_window.md";
const OUTPUT: &str = "help.html"; // In the `OUT_DIR` folder.

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed={INPUT}");

    let md = fs::read_to_string(INPUT)?;

    let out_dir = env::var("OUT_DIR")?;
    let html = File::create(Path::new(&out_dir).join(OUTPUT))?;
    let mut writer = BufWriter::new(html);

    let parser = Parser::new_ext(&md, Options::ENABLE_SMART_PUNCTUATION);

    // Insert heading anchors. See https://github.com/raphlinus/pulldown-cmark/issues/407.
    let mut heading_level: Option<HeadingLevel> = None;
    let parser = parser.filter_map(|event| match event {
        Event::Start(Tag::Heading(level, ..)) => {
            heading_level = Some(level);
            None
        }
        Event::Text(heading_text) if heading_level.is_some() => {
            Some(Event::Html(CowStr::from(format!(
                r#"<{} id="{}">{heading_text}"#,
                heading_level.take().unwrap(),
                // WARNING: This is potentially fragile if used with non-ASCII heading text!
                heading_text.trim().to_lowercase().replace(' ', "-"),
            ))))
        }
        rest => Some(rest),
    });

    // Open all inline links in a new tab.
    let parser = parser.map(|event| match event {
        Event::Start(Tag::Link(LinkType::Autolink | LinkType::Inline, dest, title))
            if dest.starts_with("http") =>
        {
            Event::Html(CowStr::from(format!(
                r#"<a href="{dest}" target="_blank">{title}"#,
            )))
        }
        rest => rest,
    });

    html_renderer::write_html(&mut writer, parser)?;
    writer.flush()?;

    Ok(())
}
