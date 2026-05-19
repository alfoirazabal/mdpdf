use std::fs;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

// ---------------------------------------------------------------------------
// Shared progress event payload
// ---------------------------------------------------------------------------

#[derive(Serialize, Clone)]
struct ProgressPayload {
    message: String,
    percent: u8,
    done: bool,
    error: Option<String>,
}

fn emit_progress(app: &AppHandle, event: &str, msg: &str, pct: u8, done: bool, err: Option<String>) {
    let _ = app.emit(event, ProgressPayload {
        message: msg.to_string(),
        percent: pct,
        done,
        error: err,
    });
}

// ---------------------------------------------------------------------------
// render_pdf command
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderParams {
    input_path: String,
    output_path: String,
    /// The full CSS text from the editor — always treated as an inline custom template.
    css: String,
    scale: f64,
    generate_html: bool,
    allow_html: bool,
    one_page: bool,
    manual_breaks: bool,
    custom_metadata: Vec<String>,
}

#[tauri::command]
pub async fn render_pdf(app: AppHandle, params: RenderParams) -> Result<(), String> {
    if params.one_page && params.manual_breaks {
        return Err("--one-page and --manual-breaks are mutually exclusive.".to_string());
    }

    // Validate scale
    if params.scale < 0.1 || params.scale > 2.0 {
        return Err(format!(
            "Invalid scale {}: must be between 0.1 and 2.0.",
            params.scale
        ));
    }

    // Validate metadata format
    for entry in &params.custom_metadata {
        if !entry.contains('=') {
            return Err(format!(
                "Invalid metadata `{}` — expected Key=Value format.",
                entry
            ));
        }
    }

    let output_filename = crate::fix_output_filename(&params.output_path);

    emit_progress(&app, "render_progress", "Reading Markdown…", 10, false, None);
    let md = fs::read_to_string(&params.input_path)
        .map_err(|e| format!("Cannot read input file: {}", e))?;

    emit_progress(&app, "render_progress", "Converting to HTML…", 25, false, None);
    let html_body = crate::markdown::to_html(&md, params.allow_html || params.manual_breaks)
        .map_err(|e| e.to_string())?;

    emit_progress(&app, "render_progress", "Applying template…", 40, false, None);
    let title = crate::get_default_title(&params.input_path);
    let full_html = crate::templates::wrap_html_with_inline_css(&html_body, &title, &params.css);

    let temp_html = format!("{}.html", output_filename);
    fs::write(&temp_html, &full_html)
        .map_err(|e| format!("Cannot write temporary HTML: {}", e))?;

    emit_progress(
        &app,
        "render_progress",
        "Converting HTML to PDF — this may take a moment…",
        55,
        false,
        None,
    );
    crate::pdf::html_to_pdf(
        &temp_html,
        &output_filename,
        &params.scale,
        params.one_page,
        params.manual_breaks,
    )
    .await
    .map_err(|e| format!("PDF conversion failed: {}", e))?;

    emit_progress(&app, "render_progress", "Attaching Markdown to PDF…", 85, false, None);
    crate::embed::attach_file_and_embed_metadata(
        &output_filename,
        &params.input_path,
        &params.custom_metadata,
    )
    .map_err(|e| format!("Failed to embed source: {}", e))?;

    if !params.generate_html {
        let _ = fs::remove_file(&temp_html);
    }

    emit_progress(
        &app,
        "render_progress",
        &format!("Done! Saved to {}", output_filename),
        100,
        true,
        None,
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// extract_markdown command
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractParams {
    input_path: String,
    output_path: String,
}

#[tauri::command]
pub async fn extract_markdown(app: AppHandle, params: ExtractParams) -> Result<(), String> {
    emit_progress(&app, "extract_progress", "Extracting Markdown from PDF…", 50, false, None);

    crate::extract::extract_file(&params.input_path, &params.output_path)
        .map_err(|e| e.to_string())?;

    emit_progress(
        &app,
        "extract_progress",
        &format!("Done! Saved to {}", params.output_path),
        100,
        true,
        None,
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// get_template_css command
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn get_template_css(template_name: String) -> String {
    crate::templates::get_template_css_by_name(&template_name).to_string()
}

// ---------------------------------------------------------------------------
// App entry point for GUI mode
// ---------------------------------------------------------------------------

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            render_pdf,
            extract_markdown,
            get_template_css,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
