use anyhow::Result;
use headless_chrome::{Browser, LaunchOptions};
use std::path::Path;

pub async fn html_to_pdf(input: &str, output: &str) -> Result<()> {

    let path = std::fs::canonicalize(Path::new(input))?;
    let url = format!("file://{}", path.to_string_lossy());

    let browser = Browser::new(LaunchOptions::default())?;
    let tab = browser.new_tab()?;

    tab.navigate_to(&url)?;
    tab.wait_until_navigated()?;

    let options = headless_chrome::types::PrintToPdfOptions {
        margin_top: Some(0.0),
        margin_bottom: Some(0.0),
        margin_left: Some(0.0),
        margin_right: Some(0.0),
        generate_document_outline: Some(true),
        prefer_css_page_size: Some(true),
        display_header_footer: Some(false),
        print_background: Some(true),
        ..Default::default()
    };

    let pdf = tab.print_to_pdf(Option::Some(options))?;

    std::fs::write(output, pdf)?;

    Ok(())
}