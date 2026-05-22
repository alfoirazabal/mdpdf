use lopdf::Document;
use std::fs::File;
use std::io::Write;

use crate::embed;

pub struct ExtractResult {
    pub css_extracted: bool,
}

pub fn extract_file(
    pdf_path: &str,
    md_output_path: &str,
    css_output_path: &str,
) -> anyhow::Result<ExtractResult> {
    let doc = Document::load(pdf_path)?;

    let catalog = doc.catalog()?;
    let names = catalog.get(b"Names")?.as_reference()?;
    let names_dict = doc.get_object(names)?.as_dict()?;

    let embedded_files = names_dict
        .get(b"EmbeddedFiles")?
        .as_reference()?;
    let embedded_dict = doc.get_object(embedded_files)?.as_dict()?;

    let files = embedded_dict.get(b"Names")?.as_array()?;

    let mut md_found = false;
    let mut css_extracted = false;

    for i in (0..files.len()).step_by(2) {
        let name_obj = &files[i];
        let file_spec_ref = files[i + 1].as_reference()?;

        let name = std::str::from_utf8(name_obj.as_str()?)?;

        if name == embed::SOURCE_MD_FILE_NAME {
            let file_spec = doc.get_object(file_spec_ref)?.as_dict()?;
            let ef_dict = file_spec.get(b"EF")?.as_dict()?;
            let file_stream_ref = ef_dict.get(b"F")?.as_reference()?;
            let stream = doc.get_object(file_stream_ref)?.as_stream()?;
            let data = match stream.decompressed_content() {
                Ok(data) => data,
                Err(_) => stream.content.clone(),
            };
            let mut file = File::create(md_output_path)?;
            file.write_all(&data)?;
            md_found = true;
        } else if name == embed::SOURCE_CSS_FILE_NAME {
            let file_spec = doc.get_object(file_spec_ref)?.as_dict()?;
            let ef_dict = file_spec.get(b"EF")?.as_dict()?;
            let file_stream_ref = ef_dict.get(b"F")?.as_reference()?;
            let stream = doc.get_object(file_stream_ref)?.as_stream()?;
            let data = match stream.decompressed_content() {
                Ok(data) => data,
                Err(_) => stream.content.clone(),
            };
            let mut file = File::create(css_output_path)?;
            file.write_all(&data)?;
            css_extracted = true;
        }
    }

    if !md_found {
        anyhow::bail!("File not found in PDF: {}", embed::SOURCE_MD_FILE_NAME);
    }

    Ok(ExtractResult { css_extracted })
}