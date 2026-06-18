use std::{error::Error, fs::read_to_string, path::PathBuf};

use typstyle_core::{Config, Typstyle};

/// Parameters for a formatting operation.
///
/// See also [`format()`].
#[derive(Debug, Clone, Default)]
pub struct FormatParams {
    /// Path to the input Typst file.
    pub input: PathBuf,

    /// The width of the output.
    pub column: usize,

    /// The number of spaces to use for a tab character.
    pub tab_spaces: usize,
}

/// Formats a Typst file with [Enter-tainer/typstyle](https://github.com/Enter-tainer/typstyle/).
///
/// # Arguments
///
/// - `params` - [`FormatParams`] struct.
///
/// # Returns
///
/// String containing the formatted Typst file.
///
/// # Example
///
/// Following is an example of how to use the `format` function:
///
/// ```rust
/// let params = typwriter::FormatParams {
///     input: std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
///         .join("examples")
///         .join("sample.typ"),
///     column: 80,
///     tab_spaces: 2,
/// };
///
/// println!("{}", typwriter::format(&params).map_or_else(|why| why.to_string(), |s| s));
/// ```
pub fn format(params: &FormatParams) -> Result<String, Box<dyn Error>> {
    let config = Config::new()
        .with_width(params.column)
        .with_tab_spaces(params.tab_spaces);
    let result = Typstyle::new(config)
        .format_text(&read_to_string(&params.input)?)
        .render()
        .map_err(|why| why.to_string())?;
    Ok(result)
}
