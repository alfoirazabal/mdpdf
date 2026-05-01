use anyhow::Result;
use lopdf::{Document, Object};
use std::fs;

pub const SOURCE_MD_FILE_NAME: &str = "SOURCE_MD_FILE.md";

fn create_embedded_file(
    doc: &mut lopdf::Document,
    data: Vec<u8>,
) -> lopdf::ObjectId {
    let mut ef_dict = lopdf::Dictionary::new();
    ef_dict.set("Type", "EmbeddedFile");

    let ef_stream = lopdf::Stream::new(ef_dict, data);
    doc.add_object(ef_stream)
}

fn create_filespec(
    doc: &mut lopdf::Document,
    ef_ref: lopdf::ObjectId,
) -> lopdf::ObjectId {
    let mut filespec = lopdf::Dictionary::new();
    filespec.set("Type", "Filespec");
    filespec.set("F", lopdf::Object::string_literal(SOURCE_MD_FILE_NAME));
    filespec.set("UF", lopdf::Object::string_literal(SOURCE_MD_FILE_NAME));

    let mut ef_entry = lopdf::Dictionary::new();
    ef_entry.set("F", ef_ref);

    filespec.set("EF", ef_entry);

    doc.add_object(filespec)
}

fn create_embedded_files_tree(
    doc: &mut lopdf::Document,
    file_name: &str,
    filespec_ref: lopdf::ObjectId,
) -> lopdf::ObjectId {
    let mut names_array = Vec::new();
    names_array.push(lopdf::Object::string_literal(file_name));
    names_array.push(lopdf::Object::Reference(filespec_ref));

    let mut embedded_files = lopdf::Dictionary::new();
    embedded_files.set("Names", lopdf::Object::Array(names_array));

    doc.add_object(embedded_files)
}

fn attach_to_catalog(
    doc: &mut lopdf::Document,
    embedded_files_ref: lopdf::ObjectId,
) -> lopdf::Result<()> {
    let catalog_id = doc.trailer.get(b"Root")?.as_reference()?;

    // Step 1: check if Names exists WITHOUT holding borrow
    let names_ref_opt = {
        let catalog = doc.get_object(catalog_id)?.as_dict()?;
        catalog.get(b"Names").ok().and_then(|n| n.as_reference().ok())
    };

    // Step 2: create Names if needed
    let names_ref = if let Some(r) = names_ref_opt {
        r
    } else {
        let new_names_ref = doc.add_object(lopdf::Dictionary::new());

        // now re-borrow catalog to set it
        let catalog = doc.get_object_mut(catalog_id)?.as_dict_mut()?;
        catalog.set("Names", new_names_ref);

        new_names_ref
    };

    // Step 3: modify Names dict
    let names_dict = doc.get_object_mut(names_ref)?.as_dict_mut()?;
    names_dict.set("EmbeddedFiles", embedded_files_ref);

    Ok(())
}

fn make_pdf_utf16_string(s: &str) -> Object {
    // UTF-16BE + BOM
    let mut bytes = vec![0xFE, 0xFF]; // BOM

    for unit in s.encode_utf16() {
        bytes.push((unit >> 8) as u8);
        bytes.push((unit & 0xFF) as u8);
    }

    Object::String(bytes, lopdf::StringFormat::Literal)
}

pub fn attach_file_and_embed_metadata(output_path: &str, input_path: &str, custom_metadata: &[String]) -> Result<()> {
    let mut doc = Document::load(output_path)?;
    let data = fs::read(input_path)?;

    let ef_ref = create_embedded_file(&mut doc, data);
    let filespec_ref = create_filespec(&mut doc, ef_ref);
    let tree_ref = create_embedded_files_tree(&mut doc, SOURCE_MD_FILE_NAME, filespec_ref);

    let info_id = doc.trailer.get(b"Info")
        .and_then(|obj| obj.as_reference())
        .unwrap_or_else(|_| {
            let id = doc.new_object_id();
            doc.trailer.set("Info", Object::Reference(id));
            id
        });
    
    let info_dict = doc.get_object_mut(info_id)?.as_dict_mut()?;

    let new_creator_value = crate::constants::generate_pdf_metadata_creator_value();
    info_dict.set("Creator", lopdf::Object::string_literal(new_creator_value));

    for key in custom_metadata {
        let (k, v) = key.split_once('=').unwrap_or_else(|| {
            eprintln!("Invalid custom metadata format: `{}`. Expected `key=value`.", key);
            std::process::exit(1);
        });
        info_dict.set(k, make_pdf_utf16_string(v));
    }

    attach_to_catalog(&mut doc, tree_ref)?;

    doc.compress();
    doc.save(output_path)?;
    Ok(())
}