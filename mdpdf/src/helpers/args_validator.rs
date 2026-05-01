pub fn validate_custom_metadata(metadata: &[String]) {
    for entry in metadata {
        if !entry.contains('=') {
            eprintln!("Invalid custom metadata format: `{}`. Expected `key=value`.", entry);
            std::process::exit(1);
        }
    }
}

pub fn validate_scale(scale: f64) {
    if scale < 0.1 || scale > 2.0 {
        eprintln!("Invalid scale value: {}. The scale must be within the range [0.1 – 2].", scale);
        std::process::exit(1);
    }
}