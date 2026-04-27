const KATEX_CSS: &str = include_str!("../assets/katex.min.css");
const KATEX_JS: &str = include_str!("../assets/katex.min.js");
const AUTO_RENDER_JS: &str = include_str!("../assets/auto-render.min.js");
const RENDER_MATH_IN_ELEMENT_CALLER: &str = include_str!("../assets/calls/render-math-in-element-caller.js");

const CSS_STYLE_MOBILE_DARK: &str = include_str!("../assets/styles/style-mobile-dark.css");
const CSS_STYLE_MOBILE_LIGHT: &str = include_str!("../assets/styles/style-mobile-light.css");

pub enum TemplateType {
  MobileDark,
  MobileLight
}

fn get_css_style(template_type: TemplateType) -> &'static str {
  match template_type {
    TemplateType::MobileDark => CSS_STYLE_MOBILE_DARK,
    TemplateType::MobileLight => CSS_STYLE_MOBILE_LIGHT
  }
}

pub fn wrap_html_mobile_template(body: &str, title: &str, template_type: TemplateType) -> String {

  let template = get_css_style(template_type);

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