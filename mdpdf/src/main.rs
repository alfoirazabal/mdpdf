mod markdown;
mod templates;
mod pdf;
mod embed;
mod cli;
mod extract;
mod enums;

use anyhow::Result;
use std::fs;
use std::path::Path;
use clap::Parser;
use cli::{Cli, Commands};

fn get_default_title(input: &str) -> String {
    let path = Path::new(&input);

    let stem = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap()
        .to_string();

    return stem;
}

#[tokio::main]
async fn main() -> Result<()> {

    let cli = Cli::parse();

    match cli.command {
        Commands::Render { input, output, template } => {
            let title = get_default_title(&input);

            println!("Reading Markdown...");
            let md = fs::read_to_string(&input)?;

            println!("Converting to HTML...");
            let html_body = markdown::to_html(&md)?;

            println!("Applying template...");
            let full_html = templates::wrap_html(&html_body, &title, template);

            let temp_html = "temp.html";
            fs::write(temp_html, full_html)?;

            println!("Converting HTML to PDF...");
            pdf::html_to_pdf(temp_html, &output).await?;

            println!("Attaching MD to PDF...");
            embed::attach_file(&output, &input)?;

            match fs::remove_file(&temp_html) {
                Ok(()) => { }
                Err(e) => {
                    eprintln!("Cannot delete temporary `temp.html` file: {}", e);
                    std::process::exit(1);
                }
            }

            println!("Done: {}", output);
        }
        Commands::Extract { input, output } => {
            match extract::extract_file(&input, &output) {
                Ok(()) => {
                    std::process::exit(0);
                },
                Err(err) => {
                    eprintln!("ERROR: {}", err);
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}