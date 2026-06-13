use std::{
    collections::HashMap,
    fs::{read_to_string, remove_file},
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Result, anyhow};
use sha2_hasher::Sha2Hasher;
use test_context::{TestContext, test_context};
use typwriter::{
    CompileParams, FormatParams, PdfMetadata, PermissionParams, PrintPermission, compile, format,
    set_permission, typst_version, update_metadata,
};

struct TypwriterTestContext {
    export_pdf: (PathBuf, CompileParams),
    export_png: (PathBuf, CompileParams),
    update_metadata: (PathBuf, CompileParams),
    set_permission: (PathBuf, (PathBuf, CompileParams)),
    format: (String, FormatParams),
}

impl TestContext for TypwriterTestContext {
    fn setup() -> TypwriterTestContext {
        let path = |n| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join(n);
        let params = |n| {
            (
                path(n),
                CompileParams {
                    input: path("sample.typ"),
                    output: path(n),
                    ..Default::default()
                },
            )
        };

        TypwriterTestContext {
            export_pdf: params("export_pdf.pdf"),
            export_png: params("export_png.png"),
            update_metadata: params("update_metadata.pdf"),
            set_permission: (path("set_permission_protected.pdf"), params("set_permission.pdf")),
            format: (
                read_to_string(path("formatted.typ")).unwrap().trim().to_string(),
                FormatParams {
                    input: path("sample.typ"),
                    column: 80,
                    tab_spaces: 2,
                },
            ),
        }
    }

    fn teardown(self) {}
}

#[test_context(TypwriterTestContext)]
#[test]
fn test_export_pdf(ctx: &TypwriterTestContext) -> Result<()> {
    let TypwriterTestContext { export_pdf: (out, params), .. } = ctx;
    assert!(compile(params).is_ok());
    assert!(out.exists());
    assert_eq!(out.sha256()?, "59685271daa01489a58b3521854d547b11b892308a8fc052120e25c501312a48");

    remove_file(out)?;
    Ok(())
}

#[test_context(TypwriterTestContext)]
#[test]
fn test_export_png(ctx: &TypwriterTestContext) -> Result<()> {
    let TypwriterTestContext { export_png: (out, params), .. } = ctx;
    assert!(compile(params).is_ok());
    assert!(out.exists());
    assert_eq!(out.sha256()?, "4431b5b28132b5def1074ad20a06a81b80938ab1fe5b32e5ae559d04d16c5c42");

    remove_file(out)?;
    Ok(())
}

#[test_context(TypwriterTestContext)]
#[test]
fn test_update_metadata(ctx: &TypwriterTestContext) -> Result<()> {
    let TypwriterTestContext { update_metadata: (out, params), .. } = ctx;
    let mut custom_properties = HashMap::new();
    custom_properties.insert("robots".to_string(), "noindex".to_string());
    custom_properties.insert("custom".to_string(), "properties".to_string());

    let metadata = PdfMetadata {
        title: "Title タイトル (typwriter)".to_string(),
        author: "Author 著者 (typwriter)".to_string(),
        application: "Application アプリケーション (typwriter)".to_string(),
        subject: "Subject 題名 (typwriter)".to_string(),
        copyright_status: true,
        copyright_notice: "Copyright notice (typwriter)".to_string(),
        keywords: vec!["typwriter".to_string(), "rust".to_string(), "pdf".to_string()],
        language: "en".to_string(),
        custom_properties,
    };

    assert!(compile(params).is_ok());
    assert!(out.exists());
    assert!(update_metadata(out, &metadata).is_ok());
    assert!(out.metadata()?.len() > 0);

    let props = get_properties(out)?;
    assert_eq!(props.get("Title"), Some(&"Title タイトル (typwriter)".to_string()));
    assert_eq!(props.get("Author"), Some(&"Author 著者 (typwriter)".to_string()));
    assert_eq!(props.get("Creator"), Some(&"Author 著者 (typwriter)".to_string()));
    assert_eq!(
        props.get("Producer"),
        Some(&"Application アプリケーション (typwriter)".to_string())
    );
    assert_eq!(
        props.get("Creator Tool"),
        Some(&"Application アプリケーション (typwriter)".to_string())
    );
    // dc:description contains the document subject (displayed as "Description" by exiftool)
    assert_eq!(props.get("Description"), Some(&"Subject 題名 (typwriter)".to_string()));
    // dc:subject contains keywords as RDF Bag (displayed as "Subject" by exiftool)
    // This is required for Adobe Reader to display keywords without quotes
    assert_eq!(props.get("Subject"), Some(&"typwriter, rust, pdf".to_string()));
    assert_eq!(props.get("Marked"), Some(&"True".to_string()));
    assert_eq!(props.get("Rights"), Some(&"Copyright notice (typwriter)".to_string()));
    assert_eq!(props.get("Keywords"), Some(&"typwriter, rust, pdf".to_string()));
    assert_eq!(props.get("Language"), Some(&"en".to_string()));
    assert_eq!(props.get("Robots"), Some(&"noindex".to_string()));
    assert_eq!(props.get("Custom"), Some(&"properties".to_string()));

    remove_file(out)?;
    Ok(())
}

#[test_context(TypwriterTestContext)]
#[test]
fn test_set_permission(ctx: &TypwriterTestContext) -> Result<()> {
    let TypwriterTestContext {
        set_permission: (out_permission, (out, params)), ..
    } = ctx;
    assert!(compile(params).is_ok());
    assert!(
        set_permission(
            out.clone(),
            out_permission.clone(),
            &PermissionParams {
                owner_password: Some("owner".to_string()),
                allow_print: PrintPermission::None,
                ..Default::default()
            },
        )
        .is_ok()
    );
    assert!(out_permission.exists());
    assert!(out_permission.metadata()?.len() > 0);

    let props = get_properties(out_permission)?;
    assert!(props.contains_key("Encryption"));
    assert_eq!(props.get("User Access"), Some(&"Copy, Annotate, Extract".to_string()));

    remove_file(out)?;
    remove_file(out_permission)?;
    Ok(())
}

#[test_context(TypwriterTestContext)]
#[test]
fn test_format(ctx: &TypwriterTestContext) -> Result<()> {
    let TypwriterTestContext { format: (expected, params), .. } = ctx;
    assert_eq!(*expected, format(params).map_err(|e| anyhow!(e.to_string()))?.trim());

    Ok(())
}

#[test]
fn test_typst_version() -> Result<()> {
    assert_eq!(typst_version(), "0.15.0-rc.1");

    Ok(())
}

fn get_properties(path: &Path) -> Result<HashMap<String, String>> {
    if Command::new("exiftool").output().is_err() {
        return Err(anyhow!("ExifTool is not installed or not found in PATH"));
    }

    let out = String::from_utf8(Command::new("exiftool").arg(path).output()?.stdout)?;
    let props = out
        .split('\n')
        .map(|line| line.split(':'))
        .filter_map(|mut line| {
            let key = line.next().unwrap_or_default().trim().to_string();
            let value = line.next().unwrap_or_default().trim().to_string();
            if !key.is_empty() { Some((key, value)) } else { None }
        })
        .collect::<HashMap<_, _>>();

    Ok(props)
}
