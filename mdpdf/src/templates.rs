use crate::enums::template_enum::Template;

const KATEX_CSS: &str = include_str!("../assets/katex.min.css");
const KATEX_JS: &str = include_str!("../assets/katex.min.js");
const AUTO_RENDER_JS: &str = include_str!("../assets/auto-render.min.js");
const RENDER_MATH_IN_ELEMENT_CALLER: &str = include_str!("../assets/calls/render-math-in-element-caller.js");

const CSS_STYLE_MOBILE_DARK: &str = include_str!("../assets/styles/style-mobile-dark.css");
const CSS_STYLE_MOBILE_LIGHT: &str = include_str!("../assets/styles/style-mobile-light.css");
const CSS_STYLE_TABLET_DARK: &str = include_str!("../assets/styles/style-tablet-dark.css");
const CSS_STYLE_TABLET_LIGHT: &str = include_str!("../assets/styles/style-tablet-light.css");
const CSS_STYLE_WIDESCREEN_DARK: &str = include_str!("../assets/styles/style-widescreen-dark.css");
const CSS_STYLE_WIDESCREEN_LIGHT: &str = include_str!("../assets/styles/style-widescreen-light.css");
const CSS_STYLE_WIDESCREEN_CUT_DARK: &str = include_str!("../assets/styles/style-widescreen-dark-cut.css");
const CSS_STYLE_WIDESCREEN_CUT_LIGHT: &str = include_str!("../assets/styles/style-widescreen-light-cut.css");
const CSS_STYLE_PRINT_A4_DARK: &str = include_str!("../assets/styles/style-print-a4-dark.css");
const CSS_STYLE_PRINT_A4_LIGHT: &str = include_str!("../assets/styles/style-print-a4-light.css");
const CSS_STYLE_PRINT_A4: &str = include_str!("../assets/styles/style-print-a4.css");

fn read_template_from_file(path: &str) -> String {
  std::fs::read_to_string(path).unwrap_or_else(|_| {
    panic!("Failed to read template file at path: {}", path);
  })
}

fn get_css_style(template_type: Option<Template>) -> &'static str {
  match template_type {
    Some(Template::MobileDark) => CSS_STYLE_MOBILE_DARK,
    Some(Template::MobileLight) => CSS_STYLE_MOBILE_LIGHT,
    Some(Template::TabletDark) => CSS_STYLE_TABLET_DARK,
    Some(Template::TabletLight) => CSS_STYLE_TABLET_LIGHT,
    Some(Template::WidescreenDark) => CSS_STYLE_WIDESCREEN_DARK,
    Some(Template::WidescreenLight) => CSS_STYLE_WIDESCREEN_LIGHT,
    Some(Template::WidescreenCutDark) => CSS_STYLE_WIDESCREEN_CUT_DARK,
    Some(Template::WidescreenCutLight) => CSS_STYLE_WIDESCREEN_CUT_LIGHT,
    Some(Template::PrintA4Dark) => CSS_STYLE_PRINT_A4_DARK,
    Some(Template::PrintA4Light) => CSS_STYLE_PRINT_A4_LIGHT,
    Some(Template::PrintA4) => CSS_STYLE_PRINT_A4,
    None => CSS_STYLE_PRINT_A4
  }
}

/// Returns the built-in CSS for the given template name string (kebab-case).
/// Falls back to the default `print-a4` template for unknown names.
pub fn get_template_css_by_name(name: &str) -> &'static str {
  match name {
    "mobile-dark"          => CSS_STYLE_MOBILE_DARK,
    "mobile-light"         => CSS_STYLE_MOBILE_LIGHT,
    "tablet-dark"          => CSS_STYLE_TABLET_DARK,
    "tablet-light"         => CSS_STYLE_TABLET_LIGHT,
    "widescreen-dark"      => CSS_STYLE_WIDESCREEN_DARK,
    "widescreen-light"     => CSS_STYLE_WIDESCREEN_LIGHT,
    "widescreen-cut-dark"  => CSS_STYLE_WIDESCREEN_CUT_DARK,
    "widescreen-cut-light" => CSS_STYLE_WIDESCREEN_CUT_LIGHT,
    "print-a4-dark"        => CSS_STYLE_PRINT_A4_DARK,
    "print-a4-light"       => CSS_STYLE_PRINT_A4_LIGHT,
    _                      => CSS_STYLE_PRINT_A4,
  }
}

/// Wraps HTML body with the given CSS inline — used by the GUI where the CSS
/// comes directly from the editor (never from a file path).
pub fn wrap_html_with_inline_css(body: &str, title: &str, css: &str) -> String {
  format!(
r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<title>{}</title>

<style>
{}
</style>

<script>
{}
</script>

<script>
{}
{}
</script>

<style>
{}
</style>

</head>
<body>
{}
</body>
</html>"#,
    title,
    KATEX_CSS,
    KATEX_JS,
    AUTO_RENDER_JS,
    RENDER_MATH_IN_ELEMENT_CALLER,
    css,
    body
  )
}

pub fn wrap_html(body: &str, title: &str, template_type: Option<Template>, template_path: Option<String>) -> String {

  let template = match template_path {
    Some(path) => read_template_from_file(&path),
    None => get_css_style(template_type).to_string()
  };

  format!(
r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<title>{}</title>

<style>
{}
</style>

<script>
{}
</script>

<script>
{}
{}
</script>

<style>
{}
</style>

</head>
<body>
{}
</body>
</html>"#,
    title,
    KATEX_CSS,
    KATEX_JS,
    AUTO_RENDER_JS,
    RENDER_MATH_IN_ELEMENT_CALLER,
    template,
    body
  )
}