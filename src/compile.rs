use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use ecow::eco_format;
use fs::write;
use output_template::{format, has_indexable_template};
use typst::{
    diag::{At, SourceResult, Warned},
    foundations::Smart,
    utils::Scalar,
};
use typst_layout::PagedDocument;
use typst_pdf::{PdfOptions, PdfStandard as TypstPdfStandard, PdfStandards, pdf};
use typst_render::{RenderOptions, render};
use typst_syntax::Span;

use crate::world::SystemWorld;

/// PDF standard that can be enforced during export.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PdfStandard {
    /// PDF 1.4.
    V1_4,
    /// PDF 1.5.
    V1_5,
    /// PDF 1.6.
    V1_6,
    /// PDF 1.7.
    V1_7,
    /// PDF 2.0.
    V2_0,
    /// PDF/A-1b.
    A1b,
    /// PDF/A-1a.
    A1a,
    /// PDF/A-2b.
    A2b,
    /// PDF/A-2u.
    A2u,
    /// PDF/A-2a.
    A2a,
    /// PDF/A-3b.
    A3b,
    /// PDF/A-3u.
    A3u,
    /// PDF/A-3a.
    A3a,
    /// PDF/A-4.
    A4,
    /// PDF/A-4f.
    A4f,
    /// PDF/A-4e.
    A4e,
    /// PDF/UA-1.
    Ua1,
}

impl From<PdfStandard> for TypstPdfStandard {
    fn from(standard: PdfStandard) -> Self {
        match standard {
            PdfStandard::V1_4 => TypstPdfStandard::V_1_4,
            PdfStandard::V1_5 => TypstPdfStandard::V_1_5,
            PdfStandard::V1_6 => TypstPdfStandard::V_1_6,
            PdfStandard::V1_7 => TypstPdfStandard::V_1_7,
            PdfStandard::V2_0 => TypstPdfStandard::V_2_0,
            PdfStandard::A1b => TypstPdfStandard::A_1b,
            PdfStandard::A1a => TypstPdfStandard::A_1a,
            PdfStandard::A2b => TypstPdfStandard::A_2b,
            PdfStandard::A2u => TypstPdfStandard::A_2u,
            PdfStandard::A2a => TypstPdfStandard::A_2a,
            PdfStandard::A3b => TypstPdfStandard::A_3b,
            PdfStandard::A3u => TypstPdfStandard::A_3u,
            PdfStandard::A3a => TypstPdfStandard::A_3a,
            PdfStandard::A4 => TypstPdfStandard::A_4,
            PdfStandard::A4f => TypstPdfStandard::A_4f,
            PdfStandard::A4e => TypstPdfStandard::A_4e,
            PdfStandard::Ua1 => TypstPdfStandard::Ua_1,
        }
    }
}

/// Parameters for Typst document compilation.
///
/// See also [`compile()`].
#[derive(Debug, Clone, Default)]
pub struct CompileParams {
    /// Path to the input Typst file.
    pub input: PathBuf,

    /// String key-value pairs visible through `sys.inputs` [dictionary](https://typst.app/docs/reference/foundations/dictionary/) in the `input` document.
    pub dict: Vec<(String, String)>,

    /// Path to the output file (PDF, PNG). Output format is determined by extension, and only PNG
    /// and PDF are supported.
    pub output: PathBuf,

    /// Adds additional directories to search for fonts.
    pub font_paths: Vec<PathBuf>,

    /// The PPI (pixels per inch) to use for PNG export. [`None`] means 144.
    pub ppi: Option<f32>,

    /// Custom path to local packages, defaults to system-dependent location
    pub package_path: Option<PathBuf>,

    /// Custom path to package cache, defaults to system-dependent location
    pub package_cache_path: Option<PathBuf>,

    /// PDF standards to enforce conformance with. When [`None`], no specific standard is enforced.
    /// The list is validated for compatibility (e.g., PDF/A-2b requires PDF 1.7 or later).
    /// See [`PdfStandard`] for available options.
    pub pdf_standards: Option<Vec<PdfStandard>>,
}

/// Compiles an input file into a supported output format.
///
/// # Arguments
///
/// - `params` - [`CompileParams`] struct.
///
/// # Returns
///
/// Result containing the [`Duration`] of the compilation.
///
/// # Example
///
/// Following is an example of how to use the `compile` function:
///
/// ```rust
/// let params = typwriter::CompileParams {
///     input: std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
///         .join("examples")
///         .join("sample.typ"),
///     output: std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
///         .join("examples")
///         .join("sample.pdf"),
///     font_paths: vec!["assets".into()],
///     dict: vec![("input".to_string(), "value".to_string())],
///     ppi: None,
///     package_path: None,
///     package_cache_path: None,
///     pdf_standards: None,
/// };
/// match typwriter::compile(&params) {
///     Ok(duration) => println!("Compilation succeeded in {duration:?}"),
///     Err(why) => eprintln!("{why}"),
/// }
/// ```
///
/// which is equivalent to running:
///
/// ```console
/// $ typst compile examples/sample.typ examples/sample.pdf
/// ```
pub fn compile(params: &CompileParams) -> Result<Duration, Box<dyn Error>> {
    let world = SystemWorld::new(
        &params.input,
        &params.font_paths,
        &params.dict.clone(),
        &params.package_path,
        &params.package_cache_path,
    )
    .map_err(|err| err.to_string())?;
    let start = Instant::now();

    let Warned { output, warnings } = typst::compile::<PagedDocument>(&world);
    let result = output.and_then(|document| export(&document, params));

    match result {
        Ok(()) => Ok(start.elapsed()),
        Err(errors) => Err(warnings
            .into_iter()
            .chain(errors)
            .map(|diagnostic| {
                format!(
                    "{:?}: {}\n{}",
                    diagnostic.severity,
                    diagnostic.message.clone(),
                    diagnostic
                        .hints
                        .iter()
                        .map(|e| format!("hint: {}", e.v))
                        .collect::<Vec<String>>()
                        .join("\n")
                )
            })
            .collect::<Vec<String>>()
            .join("\n")
            .into()),
    }
}

/// Export into the target format.
fn export(document: &PagedDocument, params: &CompileParams) -> SourceResult<()> {
    match params.output.extension() {
        Some(ext) if ext.eq_ignore_ascii_case("png") => export_image(document, params),
        _ => export_pdf(document, params),
    }
}

/// Export to one or multiple PNGs.
fn export_image(document: &PagedDocument, params: &CompileParams) -> SourceResult<()> {
    let output = &params.output.to_str().unwrap_or_default();
    let can_handle_multiple = has_indexable_template(output);

    if !can_handle_multiple && document.pages().len() > 1 {
        panic!("{}", "cannot export multiple images without `{{n}}` in output path");
    }

    document.pages().iter().enumerate().for_each(|(i, page)| {
        let storage;
        let path = if can_handle_multiple {
            storage = format(output, i + 1, document.pages().len());
            Path::new(&storage)
        } else {
            params.output.as_path()
        };
        let pixmap = render(
            page,
            &RenderOptions {
                pixel_per_pt: Scalar::new(f64::from(params.ppi.unwrap_or(144.0) / 72.0)),
                render_bleed: false,
            },
        );
        let buf = pixmap.encode_png().unwrap();
        write(path, buf).unwrap();
    });

    Ok(())
}

/// Export to a PDF.
fn export_pdf(document: &PagedDocument, params: &CompileParams) -> SourceResult<()> {
    let standards = match &params.pdf_standards {
        Some(list) => {
            let typst_standards: Vec<TypstPdfStandard> = list.iter().map(|s| (*s).into()).collect();
            PdfStandards::new(&typst_standards)
                .map_err(|err| eco_format!("invalid PDF standards: {}", err.message()))
                .at(Span::detached())?
        }
        None => PdfStandards::default(),
    };
    let options = PdfOptions {
        ident: Smart::Auto,
        creator: Smart::Auto,
        timestamp: None,
        page_ranges: None,
        standards,
        tagged: true,
        pretty: false,
    };
    write(&params.output, pdf(document, &options)?)
        .map_err(|err| eco_format!("failed to write PDF: {err}"))
        .at(Span::detached())?;
    Ok(())
}

mod output_template {
    const INDEXABLE: [&str; 3] = ["{p}", "{0p}", "{n}"];

    pub fn has_indexable_template(output: &str) -> bool {
        INDEXABLE.iter().any(|template| output.contains(template))
    }

    pub fn format(output: &str, this_page: usize, total_pages: usize) -> String {
        // Find the base 10 width of number `i`
        fn width(i: usize) -> usize {
            1 + i.checked_ilog10().unwrap_or(0) as usize
        }

        let other_templates = ["{t}"];
        INDEXABLE
            .iter()
            .chain(other_templates.iter())
            .fold(output.to_string(), |out, template| {
                let replacement = match *template {
                    "{p}" => format!("{this_page}"),
                    "{0p}" | "{n}" => format!("{:01$}", this_page, width(total_pages)),
                    "{t}" => format!("{total_pages}"),
                    _ => unreachable!("unhandled template placeholder {template}"),
                };
                out.replace(template, replacement.as_str())
            })
    }
}
