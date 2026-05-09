use anyhow::Result;
use comrak::{markdown_to_html, Options};

pub fn to_html(md: &str, allow_html: bool) -> Result<String> {
    let mut options = Options::default();

    options.extension.table = true;
    options.extension.footnotes = true;
    options.extension.strikethrough = true;

    // Enables raw HTML passthrough. Note: comrak does not separate raw HTML
    // from unsafe link schemes (javascript:, data:, etc.) — both are controlled
    // by this single flag. Use with awareness of your content source.
    if allow_html {
        options.render.r#unsafe = true;
    }

    Ok(markdown_to_html(md, &options))
}