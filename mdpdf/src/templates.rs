const KATEX_CSS: &str = include_str!("../assets/katex.min.css");
const KATEX_JS: &str = include_str!("../assets/katex.min.js");
const AUTO_RENDER_JS: &str = include_str!("../assets/auto-render.min.js");
const RENDER_MATH_IN_ELEMENT_CALLER: &str = include_str!("../assets/calls/render-math-in-element-caller.js");

pub fn wrap_html_mobile_template(body: &str) -> String {
    format!(
r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">

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
@page {{
  size: 4.5in 10in;
}}

body {{
  font-family: system-ui;
  line-height: 1.6;
  background: #121212;
  color: #eaeaea;
  margin: 24px;
}}

pre {{
  background: #1e1e1e;
  padding: 12px;
  border-radius: 8px;
  overflow-x: auto;
}}
</style>

</head>
<body>
{}
</body>
</html>"#,
        KATEX_CSS,
        KATEX_JS,
        AUTO_RENDER_JS,
        RENDER_MATH_IN_ELEMENT_CALLER,
        body
    )
}