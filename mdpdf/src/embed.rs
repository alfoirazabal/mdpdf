use anyhow::Result;
use lopdf::{Document, Object};
use std::fs;

pub fn attach_file(pdf_path: &str, file_path: &str) -> Result<()> {
    let mut doc = Document::load(pdf_path)?;
    let data = fs::read(file_path)?;

    // Very minimal embedding (can be expanded later)
    let file_stream = lopdf::Stream::new(
        lopdf::Dictionary::new(),
        data,
    );

    let file_id = doc.add_object(file_stream);

    // NOTE: Proper embedding requires FileSpec + Names dictionary
    // This is a placeholder minimal step — extend later

    doc.save(pdf_path)?;
    Ok(())
}