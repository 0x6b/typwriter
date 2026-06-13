use std::path::PathBuf;

use tokio::runtime::Runtime;
use typwriter::{CompileParams, FittingType, watch};

fn main() {
    let rt = Runtime::new().unwrap();
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

    rt.block_on(async {
        if let Err(error) =
            watch(&params, true, Some("Google Chrome.app"), Some(FittingType::Width)).await
        {
            eprintln!("Server error: {}", error)
        }
    });
}
