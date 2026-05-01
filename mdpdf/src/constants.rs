pub const DEV_NAME: &str = "Alfonso Irazabal Levy";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn about() -> String {
    format!(
        "MDPDF - Convert Markdown to PDF and vice versa - By {} - v{}",
        DEV_NAME, VERSION
    )
}

pub fn generate_pdf_metadata_creator_value() -> String {
    format!("MDPDF v{}", VERSION)
}