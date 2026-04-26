mod markdown;
mod template;
mod pdf;
mod embed;

use anyhow::Result;
use std::fs;

#[tokio::main]
async fn main() -> Result<()> {
    let input_path = "examples/sample.md";
    let output_pdf = "output.pdf";

    let md = fs::read_to_string(input_path)?;

    let html_body = markdown::to_html(&md)?;
    let full_html = template::wrap_html_mobile_template(&html_body);

    let temp_html = "temp.html";

    println!("Converting to Html...");

    fs::write(temp_html, full_html)?;

    println!("Converting Html to PDF...");

    pdf::html_to_pdf(temp_html, output_pdf).await?;

    println!("Attaching MD to PDF...");

    embed::attach_file(output_pdf, input_path)?;

    println!("Done: {}", output_pdf);

    Ok(())
}