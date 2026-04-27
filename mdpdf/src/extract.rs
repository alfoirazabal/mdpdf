use lopdf::{Document};
use std::fs::File;
use std::io::Write;

use crate::embed;

pub fn extract_file(
    pdf_path: &str,
    output_path: &str,   // where you want to save it
) -> anyhow::Result<()> {
    let doc = Document::load(pdf_path)?;

    let catalog = doc.catalog()?;
    let names = catalog.get(b"Names")?.as_reference()?;
    let names_dict = doc.get_object(names)?.as_dict()?;

    let embedded_files = names_dict
        .get(b"EmbeddedFiles")?
        .as_reference()?;
    let embedded_dict = doc.get_object(embedded_files)?.as_dict()?;

    let files = embedded_dict.get(b"Names")?.as_array()?;

    for i in (0..files.len()).step_by(2) {
        let name_obj = &files[i];
        let file_spec_ref = files[i + 1].as_reference()?;

        let name = std::str::from_utf8(name_obj.as_str()?)?;

        println!("{}", name);

        if name == embed::SOURCE_MD_FILE_NAME {
            let file_spec = doc.get_object(file_spec_ref)?.as_dict()?;

            let ef_dict = file_spec.get(b"EF")?.as_dict()?;
            let file_stream_ref = ef_dict.get(b"F")?.as_reference()?;

            let stream = doc.get_object(file_stream_ref)?.as_stream()?;

            let data = match stream.decompressed_content() {
                Ok(data) => data,
                Err(_) => stream.content.clone(),
            };

            let mut file = File::create(output_path)?;
            file.write_all(&data)?;

            println!("Extracted {} → {}", name, output_path);
            return Ok(());
        }
    }

    anyhow::bail!("File not found in PDF: {}", embed::SOURCE_MD_FILE_NAME);
}