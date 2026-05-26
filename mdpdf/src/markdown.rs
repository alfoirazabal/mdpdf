use anyhow::Result;
use comrak::{markdown_to_html_with_plugins, Options};
use comrak::adapters::SyntaxHighlighterAdapter;
use comrak::options::Plugins;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::html::{IncludeBackground, append_highlighted_html_for_styled_line};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

// ---------------------------------------------------------------------------
// CSS background-color detection
// ---------------------------------------------------------------------------

/// Returns `true` when the code-block background declared in `css` is dark
/// (perceived luminance below 0.5). Falls back to `false` (light) if no color
/// can be found.
fn code_bg_is_dark(css: &str) -> bool {
    extract_code_bg_color(css)
        .map(|c| perceived_luminance(c) < 0.5)
        .unwrap_or(false)
}

fn extract_code_bg_color(css: &str) -> Option<(u8, u8, u8)> {
    find_css_var(css, "--default-code-background")
        .or_else(|| find_pre_background(css))
}

/// Searches for `--name: #rrggbb` and parses the hex color.
fn find_css_var(css: &str, var_name: &str) -> Option<(u8, u8, u8)> {
    let needle = format!("{}:", var_name);
    let after = css.find(needle.as_str()).map(|p| css[p + needle.len()..].trim_start())?;
    parse_hex_color_prefix(after)
}

/// Searches the first `pre { … }` block for `background` / `background-color`.
fn find_pre_background(css: &str) -> Option<(u8, u8, u8)> {
    let mut rest = css;
    loop {
        let pre_pos = rest.find("pre")?;
        let after_pre = rest[pre_pos + 3..].trim_start();
        if after_pre.starts_with('{') {
            // `after_pre` is a suffix of `css`, so its offset is `css.len() - after_pre.len()`
            let block_start = css.len() - after_pre.len();
            let block_end = css[block_start..].find('}').map(|i| block_start + i)?;
            let block = &css[block_start..block_end];
            for prop in &["background-color:", "background:"] {
                if let Some(p) = block.find(prop) {
                    let val = block[p + prop.len()..].trim_start();
                    if let Some(rgb) = parse_hex_color_prefix(val) {
                        return Some(rgb);
                    }
                }
            }
            return None;
        }
        rest = &rest[pre_pos + 3..];
    }
}

/// Parses a hex color at the start of `s`: `#rgb` or `#rrggbb`.
fn parse_hex_color_prefix(s: &str) -> Option<(u8, u8, u8)> {
    let s = s.trim_start();
    if !s.starts_with('#') {
        return None;
    }
    let hex: String = s[1..].chars().take_while(|c| c.is_ascii_hexdigit()).collect();
    match hex.len() {
        6 => Some((
            u8::from_str_radix(&hex[0..2], 16).ok()?,
            u8::from_str_radix(&hex[2..4], 16).ok()?,
            u8::from_str_radix(&hex[4..6], 16).ok()?,
        )),
        3 => Some((
            u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?,
            u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?,
            u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?,
        )),
        _ => None,
    }
}

/// Perceived luminance in [0, 1].
fn perceived_luminance((r, g, b): (u8, u8, u8)) -> f32 {
    (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32) / 255.0
}

// ---------------------------------------------------------------------------
// Syntax highlighter adapter
// ---------------------------------------------------------------------------

/// A comrak `SyntaxHighlighterAdapter` backed by syntect.
///
/// Writes only foreground token colours (no background on `<pre>`), so the
/// code-block background is fully controlled by the user's CSS.
struct SyntaxHighlighter {
    theme_name: &'static str,
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}

impl SyntaxHighlighter {
    fn new(theme_name: &'static str) -> Self {
        SyntaxHighlighter {
            theme_name,
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
        }
    }
}

impl SyntaxHighlighterAdapter for SyntaxHighlighter {
    fn write_highlighted(
        &self,
        output: &mut dyn fmt::Write,
        lang: Option<&str>,
        code: &str,
    ) -> fmt::Result {
        let syntax = lang
            .map(|l| l.split_once(',').map(|(left, _)| left).unwrap_or(l))
            .and_then(|token| self.syntax_set.find_syntax_by_token(token))
            .or_else(|| self.syntax_set.find_syntax_by_first_line(code))
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        let theme = &self.theme_set.themes[self.theme_name];
        let mut highlighter = HighlightLines::new(syntax, theme);
        let mut buf = String::new();
        for line in LinesWithEndings::from(code) {
            let regions = highlighter
                .highlight_line(line, &self.syntax_set)
                .map_err(|_| fmt::Error)?;
            append_highlighted_html_for_styled_line(&regions, IncludeBackground::No, &mut buf)
                .map_err(|_| fmt::Error)?;
        }
        output.write_str(&buf)
    }

    fn write_pre_tag<'s>(
        &self,
        output: &mut dyn fmt::Write,
        _attributes: HashMap<&'static str, Cow<'s, str>>,
    ) -> fmt::Result {
        output.write_str("<pre>")
    }

    fn write_code_tag<'s>(
        &self,
        output: &mut dyn fmt::Write,
        attributes: HashMap<&'static str, Cow<'s, str>>,
    ) -> fmt::Result {
        if let Some(class) = attributes.get("class") {
            write!(output, r#"<code class="{}">"#, class)
        } else {
            output.write_str("<code>")
        }
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

pub fn to_html(md: &str, allow_html: bool, css: &str) -> Result<String> {
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

    let theme = if code_bg_is_dark(css) { "base16-ocean.dark" } else { "InspiredGitHub" };
    let adapter = SyntaxHighlighter::new(theme);
    let mut plugins = Plugins::default();
    plugins.render.codefence_syntax_highlighter = Some(&adapter);

    Ok(markdown_to_html_with_plugins(md, &options, &plugins))
}