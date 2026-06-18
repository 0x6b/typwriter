use std::{collections::HashMap, error::Error, path::Path};

use chrono::Local;
use lopdf::{Dictionary, Document, Object, Stream, text_string};
use serde::{Deserialize, Serialize};
use xmp_writer::{DateTime, LangId, XmpWriter};

/// PDF, Dublin Core, and [Extensible Metadata Platform (XMP)](https://www.adobe.com/devnet/xmp.html) metadata for a PDF document.
///
/// See also [`update_metadata()`] and [Extensible Metadata Platform (XMP) Specification: Part 1, Data Model, Serialization, and Core Properties](https://github.com/adobe/XMP-Toolkit-SDK/blob/main/docs/XMPSpecificationPart1.pdf) for detail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfMetadata {
    /// Title of the document.
    /// - Acrobat Reader: Title
    /// - Apple Preview: Title
    pub title: String,

    /// Author of the document.
    /// - Acrobat Reader: Author
    /// - Apple Preview: Author
    pub author: String,

    /// Application.
    /// - Acrobat Reader: Application
    /// - Apple Preview: PDF Producer _and_ Content creator
    pub application: String,

    /// Subject of the document.
    /// - Acrobat Reader: Subject _and_ Description
    /// - Apple Preview: Subject
    pub subject: String,

    /// Copyright status. `true` means `Marked`.
    /// - Acrobat Reader: Copyright Status
    /// - Apple Preview: (None)
    pub copyright_status: bool,

    /// Copyright notice.
    /// - Acrobat Reader: Copyright Notice
    /// - Apple Preview: (None)
    pub copyright_notice: String,

    /// Keywords, which should be set as an array, but will be concatenated and set as a single
    /// property.
    /// - Acrobat Reader: Keywords
    /// - Apple Preview: (None)
    pub keywords: Vec<String>,

    /// Language (RFC 3066)
    pub language: String,

    /// Custom properties.
    /// - Acrobat Reader: Custom properties
    /// - Apple Preview: (None)
    pub custom_properties: HashMap<String, String>,
}

impl Default for PdfMetadata {
    fn default() -> Self {
        Self {
            title: String::new(),
            author: String::new(),
            application: "typwriter".to_string(),
            subject: String::new(),
            copyright_status: true,
            copyright_notice: format!(
                "© {} Author. All rights reserved.",
                Local::now().format("%Y")
            ),
            keywords: vec![],
            language: "en".to_string(),
            custom_properties: HashMap::new(),
        }
    }
}

/// Updates the metadata of a PDF file.
///
/// Note that:
///
/// - All metadata will be overwritten, not merged.
/// - The creation date is set automatically to the current date without time information (time is
///   always 0:00 UTC) to avoid exposing precise timestamps.
/// - PDF/UA conformance (`pdfuaid:part`) is preserved if present in the original document.
///
/// # Arguments
///
/// - `path` - Path to the PDF file.
/// - `metadata` - [`PdfMetadata`] to set.
///
/// # Example
///
/// Following is an example of how to use the `update_metadata` function:
///
/// ```rust
/// let output = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
///     .join("examples")
///     .join("sample.pdf");
///
/// // Compile a document first
/// let params = typwriter::CompileParams {
///     input: std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
///         .join("examples")
///         .join("sample.typ"),
///     output: output.clone(),
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
///
/// // Then update metadata
/// let mut custom_properties = std::collections::HashMap::new();
/// custom_properties.insert("robots".to_string(), "noindex".to_string());
/// custom_properties.insert("custom".to_string(), "properties".to_string());
///
/// let metadata = typwriter::PdfMetadata {
///     title: "Title (typwriter)".to_string(),
///     author: "Author (typwriter)".to_string(),
///     application: "Application (typwriter)".to_string(),
///     subject: "Subject (typwriter)".to_string(),
///     copyright_status: true,
///     copyright_notice: "Copyright notice (typwriter)".to_string(),
///     keywords: vec!["typwriter".to_string(), "rust".to_string(), "pdf".to_string()],
///     language: "en".to_string(),
///     custom_properties,
/// };
///
/// typwriter::update_metadata(&output, &metadata).unwrap();
/// ```
pub fn update_metadata(path: &Path, metadata: &PdfMetadata) -> Result<(), Box<dyn Error>> {
    let mut doc = Document::load(path)?;

    // Extract existing PDF/UA conformance before overwriting
    let pdfua_part = extract_pdfua_part(&doc);

    // Generate XMP metadata using xmp-writer
    let xmp_string = generate_xmp(metadata, pdfua_part);

    // Find and update the XMP metadata stream in the PDF
    update_xmp_stream(&mut doc, &xmp_string)?;

    // Update PDF Info dictionary
    update_info_dict(&mut doc, metadata);

    doc.save(path)?;

    Ok(())
}

/// Extract the PDF/UA part number from existing XMP metadata
fn extract_pdfua_part(doc: &Document) -> Option<i32> {
    let catalog = doc.catalog().ok()?;
    let metadata_ref = catalog.get(b"Metadata").ok()?;
    let metadata_id = metadata_ref.as_reference().ok()?;

    if let Ok(Object::Stream(stream)) = doc.get_object(metadata_id) {
        // Try decompressed first, fall back to raw content (XMP is often uncompressed)
        let content = stream
            .decompressed_content()
            .unwrap_or_else(|_| stream.content.clone());
        let xmp_str = String::from_utf8_lossy(&content);

        // Look for pdfuaid:part in the XMP content
        // Pattern: <pdfuaid:part>1</pdfuaid:part>
        if let Some(start) = xmp_str.find("<pdfuaid:part>") {
            let after_tag = &xmp_str[start + 14..];
            if let Some(end) = after_tag.find("</pdfuaid:part>") {
                let part_str = &after_tag[..end];
                return part_str.trim().parse().ok();
            }
        }
    }
    None
}

/// Generate XMP metadata string using xmp-writer
fn generate_xmp(metadata: &PdfMetadata, pdfua_part: Option<i32>) -> String {
    let mut xmp = XmpWriter::new();

    // Dublin Core properties
    xmp.title([(Some(LangId("x-default")), metadata.title.as_str())]);
    xmp.description([(Some(LangId("x-default")), metadata.subject.as_str())]);
    xmp.creator([metadata.author.as_str()]);
    xmp.language([LangId(&metadata.language)]);

    // XMP Rights Management
    xmp.marked(metadata.copyright_status);
    xmp.rights([(Some(LangId("x-default")), metadata.copyright_notice.as_str())]);

    // XMP Basic
    xmp.creator_tool(&metadata.application);
    let now = Local::now();
    let date = DateTime::date(
        now.format("%Y").to_string().parse().unwrap_or(2024),
        now.format("%m").to_string().parse().unwrap_or(1),
        now.format("%d").to_string().parse().unwrap_or(1),
    );
    xmp.create_date(date);
    xmp.modify_date(date);

    // Keywords: dc:subject (RDF Bag) is displayed by Adobe Reader without quotes,
    // while pdf:Keywords (simple text) is displayed with quotes.
    if !metadata.keywords.is_empty() {
        xmp.subject(metadata.keywords.iter().map(String::as_str));
        xmp.pdf_keywords(&metadata.keywords.join(", "));
    }

    // PDF/UA conformance (preserve if present in original document)
    if let Some(part) = pdfua_part {
        xmp.pdfua_part(part);
    }

    xmp.finish(None)
}

/// Find and update the XMP metadata stream in the PDF document
fn update_xmp_stream(doc: &mut Document, xmp_string: &str) -> Result<(), Box<dyn Error>> {
    // Get the catalog ObjectId from the trailer
    let catalog_id = doc.trailer.get(b"Root")?.as_reference()?;

    // Check if there's already a Metadata entry in the catalog
    {
        let catalog = doc.catalog()?;
        if let Ok(metadata_ref) = catalog.get(b"Metadata")
            && let Ok(metadata_id) = metadata_ref.as_reference()
        {
            // Update existing metadata stream
            if let Ok(Object::Stream(stream)) = doc.get_object_mut(metadata_id) {
                stream.set_plain_content(xmp_string.as_bytes().to_vec());
                stream.dict.set("Length", xmp_string.len() as i64);
                // Remove any compression filter for XMP
                stream.dict.remove(b"Filter");
                return Ok(());
            }
        }
    }

    // No existing metadata stream found, create a new one
    let mut stream_dict = Dictionary::new();
    stream_dict.set("Type", Object::Name(b"Metadata".to_vec()));
    stream_dict.set("Subtype", Object::Name(b"XML".to_vec()));
    stream_dict.set("Length", xmp_string.len() as i64);

    let stream = Stream::new(stream_dict, xmp_string.as_bytes().to_vec());
    let metadata_id = doc.add_object(Object::Stream(stream));

    // Add the Metadata reference to the catalog
    let catalog_mut = doc.get_object_mut(catalog_id)?;
    if let Object::Dictionary(catalog_dict) = catalog_mut {
        catalog_dict.set("Metadata", metadata_id);
    }

    Ok(())
}

/// Update the PDF Info dictionary
fn update_info_dict(doc: &mut Document, metadata: &PdfMetadata) {
    doc.trailer.remove(b"Info");

    let mut dict = Dictionary::new();
    dict.set("Title", text_string(&metadata.title));
    dict.set("Subject", text_string(&metadata.subject));
    dict.set("Author", text_string(&metadata.author));
    dict.set("Producer", text_string(&metadata.application));
    dict.set("Creator", text_string(&metadata.application));
    let now = Local::now().format("%Y%m%d").to_string();
    dict.set("CreationDate", text_string(&now));
    dict.set("ModDate", text_string(&now));
    dict.set("Keywords", text_string(&metadata.keywords.join(", ")));
    metadata
        .custom_properties
        .iter()
        .for_each(|(k, v)| dict.set(k.to_string(), text_string(v)));
    let t = doc.add_object(Object::Dictionary(dict));

    doc.trailer.set("Info", t);
}
