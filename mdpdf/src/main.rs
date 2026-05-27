// In release mode on Windows the binary is built as a Windows (GUI) subsystem
// application.  This prevents a console window from appearing when the user
// double-clicks the executable, and eliminates the conhost Job Object that
// would otherwise restrict Chrome process creation (os error 50).
//
// For CLI invocations from cmd / PowerShell we call AttachConsole so that
// stdout/stderr output is still visible.  Trade-off: because the process is a
// Windows-subsystem app, cmd/PowerShell return the prompt immediately instead
// of waiting for the process to finish.  This is standard Windows behaviour
// for GUI-subsystem executables.
//
// Debug builds keep the console subsystem so that cargo run / the debugger
// show output normally.
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

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
        Some("render") | Some("extract") | Some("_html-to-pdf") => true,
        // Help and version flags should be handled by the CLI parser
        Some("-h") | Some("--help") | Some("-V") | Some("--version") => true,
        _ => false,
    }
}

/// In release mode on Windows, attach to the parent process's console so that
/// stdout/stderr are visible when the CLI is invoked from cmd or PowerShell.
/// In debug builds the process is already a console-subsystem app, so this
/// is a no-op.  On non-Windows platforms this is always a no-op.
fn attach_console() {
    #[cfg(all(not(debug_assertions), target_os = "windows"))]
    {
        unsafe extern "system" {
            fn AttachConsole(dwProcessId: u32) -> i32;
        }
        const ATTACH_PARENT_PROCESS: u32 = 0xFFFF_FFFF;
        unsafe { AttachConsole(ATTACH_PARENT_PROCESS); }
    }
}

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
            scale,
            no_embed_css,
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
            let css_content = templates::get_css_content(template.clone(), custom_template_path.clone());
            let html_body = markdown::to_html(&md, allow_html || manual_breaks, &css_content)?;

            println!("{}", message_provider.get_message(status_messages::StatusMessage::ApplyingTemplate));
            let full_html = templates::wrap_html(&html_body, &title, template, custom_template_path);

            let temp_html = format!("{}{}", output_filename, ".html");
            fs::write(&temp_html, full_html)?;

            println!("{}", message_provider.get_message(status_messages::StatusMessage::ConvertingHtmlToPdf));
            pdf::html_to_pdf(&temp_html, &output_filename, &scale, one_page, manual_breaks).await?;

            println!("{}", message_provider.get_message(status_messages::StatusMessage::AttachingMdToPdf));
            let css_to_embed = if no_embed_css { None } else { Some(css_content.as_str()) };
            embed::attach_file_and_embed_metadata(&output_filename, &input, &custom_metadata, css_to_embed)?;

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
        Commands::HtmlToPdf { input, output, scale, one_page, manual_breaks } => {
            pdf::html_to_pdf(&input, &output, &scale, one_page, manual_breaks).await?;
        }
        Commands::Extract { input, output, css_output } => {
            let css_output_path = css_output.unwrap_or_else(|| {
                let stem = Path::new(&output)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("output");
                let parent = Path::new(&output).parent()
                    .filter(|p| *p != Path::new(""))
                    .map(|p| p.to_string_lossy().to_string() + "/")
                    .unwrap_or_default();
                format!("{}{}.css", parent, stem)
            });

            match extract::extract_file(&input, &output, &css_output_path) {
                Ok(result) => {
                    println!("Extracted Markdown → {}", output);
                    if result.css_extracted {
                        println!("Extracted CSS template → {}", css_output_path);
                    } else {
                        println!("Note: No CSS template was embedded in this PDF.");
                    }
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
        attach_console();
        let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
        if let Err(e) = rt.block_on(run_cli()) {
            eprintln!("ERROR: {}", e);
            std::process::exit(1);
        }
    } else {
        gui::run();
    }
}
