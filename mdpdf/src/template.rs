const KATEX_CSS: &str = include_str!("../assets/katex.min.css");
const KATEX_JS: &str = include_str!("../assets/katex.min.js");
const AUTO_RENDER_JS: &str = include_str!("../assets/auto-render.min.js");

pub fn wrap_html(body: &str) -> String {
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
renderMathInElement(document.body);
</script>

<style>
@page {{
  size: 6in 9in;
}}

body {{
  font-family: system-ui;
  line-height: 1.6;
  background: #121212;
  color: #eaeaea;
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
        body
    )
}