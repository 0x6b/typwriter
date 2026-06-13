use std::path::PathBuf;

use typwriter::{CompileParams, compile};

fn main() {
    // equivalent to:
    //     typst compile examples/sample.typ examples/sample.pdf
    let params = CompileParams {
        input: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("sample.typ"),
        output: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("sample.pdf"),
        font_paths: vec!["assets".into()],
        dict: vec![("input".to_string(), "value".to_string())],
        ppi: None,
        package_path: None,
        package_cache_path: None,
        pdf_standards: None,
    };
    match compile(&params) {
        Ok(duration) => println!("Compilation succeeded in {duration:?}"),
        Err(why) => eprintln!("{why}"),
    }
}
