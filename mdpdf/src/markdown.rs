use anyhow::Result;
use comrak::{markdown_to_html, Options};

pub fn to_html(md: &str) -> Result<String> {
    let mut options = Options::default();

    options.extension.table = true;
    options.extension.footnotes = true;
    options.extension.strikethrough = true;

    Ok(markdown_to_html(md, &options))
}