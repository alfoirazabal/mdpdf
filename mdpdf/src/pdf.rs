use anyhow::Result;
use headless_chrome::{Browser, LaunchOptions};
use url::Url;

// Chrome renders at 96 dpi by default. PrintToPdfOptions expects paper
// dimensions in inches, so we divide pixel measurements by 96.
const CHROME_DPI: f64 = 96.0;

pub async fn html_to_pdf(input: &str, output: &str, scale: &f64, one_page: bool) -> Result<()> {

    let path = std::fs::canonicalize(input)?;
    let url = Url::from_file_path(&path)
        .map_err(|_| anyhow::anyhow!("Invalid file path"))?;

    let browser = Browser::new(LaunchOptions::default())?;
    let tab = browser.new_tab()?;

    tab.navigate_to(&url.as_str())?;
    tab.wait_until_navigated()?;

    // When --one-page is set, measure the rendered content dimensions via JS
    // and use them as the paper size. Scale is applied first by Chrome during
    // rendering, so the measured dimensions already reflect it — we do NOT
    // divide by scale here; that would double-compensate.
    let (paper_width, paper_height) = if one_page {
        let result = tab.evaluate(
            "JSON.stringify({
                w: document.body.scrollWidth,
                h: document.body.scrollHeight
            })",
            false,
        )?;

        let raw = result.value
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .ok_or_else(|| anyhow::anyhow!("Failed to read content dimensions from Chrome"))?;

        let parsed: serde_json::Value = serde_json::from_str(&raw)?;

        let px_w = parsed["w"].as_f64()
            .ok_or_else(|| anyhow::anyhow!("Invalid scrollWidth value"))?;
        let px_h = parsed["h"].as_f64()
            .ok_or_else(|| anyhow::anyhow!("Invalid scrollHeight value"))?;

        (Some(px_w / CHROME_DPI), Some(px_h / CHROME_DPI))
    } else {
        (None, None)
    };

    let options = headless_chrome::types::PrintToPdfOptions {
        margin_top: Some(0.0),
        margin_bottom: Some(0.0),
        margin_left: Some(0.0),
        margin_right: Some(0.0),
        generate_document_outline: Some(true),
        prefer_css_page_size: Some(true),
        display_header_footer: Some(false),
        print_background: Some(true),
        scale: Some(*scale),
        paper_width,
        paper_height,
        ..Default::default()
    };

    let pdf = tab.print_to_pdf(Option::Some(options))?;

    std::fs::write(output, pdf)?;

    Ok(())
}