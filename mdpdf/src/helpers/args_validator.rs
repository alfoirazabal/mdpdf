pub fn validate_custom_metadata(metadata: &[String]) {
    for entry in metadata {
        if !entry.contains('=') {
            eprintln!("Invalid custom metadata format: `{}`. Expected `key=value`.", entry);
            std::process::exit(1);
        }
    }
}