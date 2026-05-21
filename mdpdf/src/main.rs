// On Windows we do NOT set windows_subsystem = "windows" at compile time.
// Instead we detach from the console at runtime when launching in GUI mode.
// This allows CLI invocations to work naturally (the shell waits and output
// appears correctly) while the GUI still launches without a visible console.

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
mod gui;

use anyhow::Result;
use std::fs;
use std::path::Path;
use clap::Parser;
use cli::{Cli, Commands};

fn perform_validations(custom_metadata: &[String], scale: f64) {
    helpers::args_validator::validate_custom_metadata(custom_metadata);
    helpers::args_validator::validate_scale(scale);
}

pub(crate) fn get_default_title(input: &str) -> String {
    let path = Path::new(&input);

    let stem = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap()
        .to_string();

    return stem;
}

pub(crate) fn fix_output_filename(output: &str) -> String {
    if output.to_lowercase().ends_with(".pdf") {
        output.to_string()
    } else {
        format!("{}.pdf", output)
    }
}

fn get_message_provider() -> Box<dyn status_messages::MessageFetcher> {
    Box::new(status_messages::StatusMessageProvider)
}

/// Returns true when the invocation looks like a CLI call (first meaningful
/// argument is a known subcommand or a standard help/version flag rather than
/// a Tauri/OS-injected flag).
fn is_cli_invocation() -> bool {
    match std::env::args().nth(1).as_deref() {
        Some("render") | Some("extract") => true,
        // Help and version flags should be handled by the CLI parser
        Some("-h") | Some("--help") | Some("-V") | Some("--version") => true,
        _ => false,
    }
}

/// On Windows, hide and detach the console window when launching in GUI mode.
/// Since we no longer use `windows_subsystem = "windows"`, the process starts
/// as a console app. In GUI mode we free the console so no black window appears.
#[cfg(target_os = "windows")]
fn detach_console() {
    unsafe extern "system" {
        fn FreeConsole() -> i32;
    }
    unsafe {
        FreeConsole();
    }
}

#[cfg(not(target_os = "windows"))]
fn detach_console() {}

async fn run_cli() -> Result<()> {
    let cli = Cli::parse();
    let mut message_provider = get_message_provider();

    match cli.command {
        Commands::Render { 
            input, output,
            template, 
            custom_template_path,
            generate_html,
            custom_metadata,
            allow_html,
            one_page,
            manual_breaks,
            scale
        } => {
            if one_page && manual_breaks {
                eprintln!("ERROR: --one-page and --manual-breaks are mutually exclusive. Use one or the other.");
                std::process::exit(1);
            }

            perform_validations(&custom_metadata, scale);

            let title = get_default_title(&input);
            
            let output_filename = fix_output_filename(&output);

            println!("{}", message_provider.get_message(status_messages::StatusMessage::ReadingMarkdown));
            let md = fs::read_to_string(&input)?;

            println!("{}", message_provider.get_message(status_messages::StatusMessage::ConvertingToHtml));
            let html_body = markdown::to_html(&md, allow_html || manual_breaks)?;

            println!("{}", message_provider.get_message(status_messages::StatusMessage::ApplyingTemplate));
            let full_html = templates::wrap_html(&html_body, &title, template, custom_template_path);

            let temp_html = format!("{}{}", output_filename, ".html");
            fs::write(&temp_html, full_html)?;

            println!("{}", message_provider.get_message(status_messages::StatusMessage::ConvertingHtmlToPdf));
            pdf::html_to_pdf(&temp_html, &output_filename, &scale, one_page, manual_breaks).await?;

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

fn main() {
    if is_cli_invocation() {
        let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
        if let Err(e) = rt.block_on(run_cli()) {
            eprintln!("ERROR: {}", e);
            std::process::exit(1);
        }
    } else {
        detach_console();
        gui::run();
    }
}
