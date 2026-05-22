use std::fs;
use std::path::{Path, PathBuf};
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

/// If output_path is relative (no directory component), resolve it relative to
/// the input file's parent directory instead of the process cwd.
fn resolve_output_path(input_path: &str, output_path: &str) -> String {
    let out = Path::new(output_path);
    if out.is_absolute() || out.parent().is_some_and(|p| p != Path::new("")) {
        return output_path.to_string();
    }
    // Bare filename — place next to the input file
    if let Some(parent) = Path::new(input_path).parent() {
        parent.join(output_path).to_string_lossy().to_string()
    } else {
        output_path.to_string()
    }
}

// ---------------------------------------------------------------------------
// User CSS template persistence
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone)]
pub struct UserTemplate {
    name: String,
    description: String,
    css: String,
}

fn templates_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("mdpdf").join("templates"))
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

#[tauri::command]
pub fn list_user_templates() -> Result<Vec<UserTemplate>, String> {
    let dir = match templates_dir() {
        Some(d) => d,
        None => return Ok(Vec::new()),
    };
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut templates = Vec::new();
    let entries = fs::read_dir(&dir)
        .map_err(|e| format!("Cannot read templates directory: {}", e))?;
    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let tmpl: UserTemplate = match serde_json::from_str(&content) {
            Ok(t) => t,
            Err(_) => continue,
        };
        templates.push(tmpl);
    }
    templates.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(templates)
}

#[tauri::command]
pub fn save_user_template(template: UserTemplate) -> Result<(), String> {
    let dir = templates_dir()
        .ok_or_else(|| "Cannot determine templates directory".to_string())?;
    fs::create_dir_all(&dir)
        .map_err(|e| format!("Cannot create templates directory: {}", e))?;
    let filename = format!("{}.json", sanitize_filename(&template.name));
    let path = dir.join(&filename);
    if path.exists() {
        return Err(format!("A template named '{}' already exists.", template.name));
    }
    let json = serde_json::to_string_pretty(&template)
        .map_err(|e| format!("Cannot serialize template: {}", e))?;
    fs::write(&path, json)
        .map_err(|e| format!("Cannot write template file: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn update_user_template(old_name: String, template: UserTemplate) -> Result<(), String> {
    let dir = templates_dir()
        .ok_or_else(|| "Cannot determine templates directory".to_string())?;
    // Remove old file
    let old_filename = format!("{}.json", sanitize_filename(&old_name));
    let old_path = dir.join(&old_filename);
    if old_path.exists() {
        fs::remove_file(&old_path)
            .map_err(|e| format!("Cannot remove old template file: {}", e))?;
    }
    // Write new file
    let new_filename = format!("{}.json", sanitize_filename(&template.name));
    let new_path = dir.join(&new_filename);
    let json = serde_json::to_string_pretty(&template)
        .map_err(|e| format!("Cannot serialize template: {}", e))?;
    fs::write(&new_path, json)
        .map_err(|e| format!("Cannot write template file: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn delete_user_template(name: String) -> Result<(), String> {
    let dir = templates_dir()
        .ok_or_else(|| "Cannot determine templates directory".to_string())?;
    let filename = format!("{}.json", sanitize_filename(&name));
    let path = dir.join(&filename);
    if !path.exists() {
        return Err(format!("Template '{}' not found.", name));
    }
    fs::remove_file(&path)
        .map_err(|e| format!("Cannot delete template: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn export_user_template(template: UserTemplate, path: String) -> Result<(), String> {
    let json = serde_json::to_string_pretty(&template)
        .map_err(|e| format!("Cannot serialize template: {}", e))?;
    fs::write(&path, json)
        .map_err(|e| format!("Cannot write template file: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn import_user_template(path: String) -> Result<UserTemplate, String> {
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Cannot read template file: {}", e))?;
    let template: UserTemplate = serde_json::from_str(&content)
        .map_err(|e| format!("Cannot parse template file: {}", e))?;
    Ok(template)
}

// ---------------------------------------------------------------------------
// Configuration persistence
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    template: Option<String>,
    css: Option<String>,
    scale: Option<f64>,
    generate_html: Option<bool>,
    allow_html: Option<bool>,
    one_page: Option<bool>,
    manual_breaks: Option<bool>,
    metadata: Option<Vec<MetadataEntry>>,
    embed_css: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MetadataEntry {
    key: String,
    value: String,
}

fn config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("mdpdf"))
}

fn config_file_path() -> Option<PathBuf> {
    config_dir().map(|d| d.join("config.json"))
}

#[tauri::command]
pub fn save_config(config: AppConfig) -> Result<(), String> {
    let path = config_file_path()
        .ok_or_else(|| "Cannot determine config directory".to_string())?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Cannot create config directory: {}", e))?;
    }
    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Cannot serialize config: {}", e))?;
    fs::write(&path, json)
        .map_err(|e| format!("Cannot write config file: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn load_config() -> Result<Option<AppConfig>, String> {
    let path = match config_file_path() {
        Some(p) => p,
        None => return Ok(None),
    };
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Cannot read config file: {}", e))?;
    let config: AppConfig = serde_json::from_str(&content)
        .map_err(|e| format!("Cannot parse config file: {}", e))?;
    Ok(Some(config))
}

#[tauri::command]
pub fn export_config(config: AppConfig, path: String) -> Result<(), String> {
    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Cannot serialize config: {}", e))?;
    fs::write(&path, json)
        .map_err(|e| format!("Cannot write config file: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn import_config(path: String) -> Result<AppConfig, String> {
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Cannot read config file: {}", e))?;
    let config: AppConfig = serde_json::from_str(&content)
        .map_err(|e| format!("Cannot parse config file: {}", e))?;
    Ok(config)
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
    embed_css: bool,
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

    let output_filename = crate::fix_output_filename(
        &resolve_output_path(&params.input_path, &params.output_path),
    );

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
    let css_to_embed = if params.embed_css { Some(params.css.as_str()) } else { None };
    crate::embed::attach_file_and_embed_metadata(
        &output_filename,
        &params.input_path,
        &params.custom_metadata,
        css_to_embed,
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
    css_output_path: String,
}

#[tauri::command]
pub async fn extract_markdown(app: AppHandle, params: ExtractParams) -> Result<(), String> {
    let output_path = resolve_output_path(&params.input_path, &params.output_path);
    let css_output_path = resolve_output_path(&params.input_path, &params.css_output_path);

    emit_progress(&app, "extract_progress", "Extracting from PDF…", 50, false, None);

    let result = crate::extract::extract_file(&params.input_path, &output_path, &css_output_path)
        .map_err(|e| e.to_string())?;

    emit_progress(
        &app,
        "extract_progress",
        &format!("Extracted Markdown → {}", output_path),
        80,
        false,
        None,
    );

    if result.css_extracted {
        emit_progress(
            &app,
            "extract_progress",
            &format!("Extracted CSS template → {}", css_output_path),
            95,
            false,
            None,
        );
    } else {
        emit_progress(
            &app,
            "extract_progress",
            "Note: No CSS template was embedded in this PDF.",
            95,
            false,
            None,
        );
    }

    emit_progress(
        &app,
        "extract_progress",
        "Done!",
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
            save_config,
            load_config,
            export_config,
            import_config,
            list_user_templates,
            save_user_template,
            update_user_template,
            delete_user_template,
            export_user_template,
            import_user_template,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
