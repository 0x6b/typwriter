use std::path::PathBuf;

use typwriter::{FormatParams, format};

fn main() {
    let params = FormatParams {
        input: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("sample.typ"),
        column: 80,
        tab_spaces: 2,
    };

    println!("{}", format(&params).unwrap_or_else(|why| why.to_string()));
}
