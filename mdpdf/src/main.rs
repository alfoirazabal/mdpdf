mod markdown;
mod templates;
mod pdf;
mod embed;
mod cli;
mod extract;
mod enums;
mod status_messages;
mod constants;
mod helpers;

use anyhow::Result;
use std::fs;
use std::path::Path;
use clap::Parser;
use cli::{Cli, Commands};

fn perform_validations(custom_metadata: &[String]) {
    helpers::args_validator::validate_custom_metadata(custom_metadata);
}

fn get_default_title(input: &str) -> String {
    let path = Path::new(&input);

    let stem = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap()
        .to_string();

    return stem;
}

fn fix_output_filename(output: &str) -> String {
    if output.to_lowercase().ends_with(".pdf") {
        output.to_string()
    } else {
        format!("{}.pdf", output)
    }
}

fn get_message_provider() -> Box<dyn status_messages::MessageFetcher> {
    Box::new(status_messages::StatusMessageProvider)
}

#[tokio::main]
async fn main() -> Result<()> {

    let cli = Cli::parse();
    let mut message_provider = get_message_provider();

    match cli.command {
        Commands::Render { 
            input, output,
            template, 
            custom_template_path,
            generate_html ,
            custom_metadata,
            scale
        } => {
            perform_validations(&custom_metadata);

            let title = get_default_title(&input);
            
            let output_filename = fix_output_filename(&output);

            println!("{}", message_provider.get_message(status_messages::StatusMessage::ReadingMarkdown));
            let md = fs::read_to_string(&input)?;

            println!("{}", message_provider.get_message(status_messages::StatusMessage::ConvertingToHtml));
            let html_body = markdown::to_html(&md)?;

            println!("{}", message_provider.get_message(status_messages::StatusMessage::ApplyingTemplate));
            let full_html = templates::wrap_html(&html_body, &title, template, custom_template_path);

            let temp_html = format!("{}{}", output_filename, ".html");
            fs::write(&temp_html, full_html)?;

            println!("{}", message_provider.get_message(status_messages::StatusMessage::ConvertingHtmlToPdf));
            pdf::html_to_pdf(&temp_html, &output_filename, &scale).await?;

            println!("{}", message_provider.get_message(status_messages::StatusMessage::AttachingMdToPdf));
            embed::attach_file_and_embed_metadata(&output_filename, &input, &custom_metadata)?;

            if !generate_html {
                match fs::remove_file(&temp_html) {
                    Ok(()) => { }
                    Err(e) => {
                        eprintln!("Cannot delete temporary `temp.html` file: {}", e);
                        std::process::exit(1);
                    }
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