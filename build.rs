#[allow(unused_imports)]
use std::{
    env::var,
    error::Error,
    fs::{self, File, read_to_string},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};
use std::{
    fs::{create_dir_all, read_dir},
    io::copy,
};

use serde::Deserialize;
use toml::from_str;
use ureq::get;

#[derive(Deserialize)]
struct ProjectMetadata {
    package: Package,
    dependencies: Dependencies,
}

#[derive(Deserialize)]
struct Package {
    #[serde(rename = "version")]
    typwriter_version: String,
}

#[derive(Deserialize)]
struct Dependencies {
    typst: Typst,
}

#[derive(Deserialize)]
pub struct Typst {
    #[serde(rename = "version")]
    pub typst_version: String,
}

/// Returns the cache directory for fonts, respecting XDG_CACHE_HOME.
#[allow(dead_code)]
fn cache_dir() -> PathBuf {
    var("XDG_CACHE_HOME")
        .map_or_else(
            |_| dirs::cache_dir().unwrap_or_else(|| PathBuf::from(".cache")),
            PathBuf::from,
        )
        .join("typwriter")
        .join("fonts")
}

/// Downloads a file from a URL and returns the bytes.
#[allow(dead_code)]
fn download(url: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    println!("cargo::warning=Downloading {url}");
    let bytes = get(url)
        .call()?
        .body_mut()
        .with_config()
        .limit(500 * 1024 * 1024) // 500 MB limit
        .read_to_vec()?;
    Ok(bytes)
}

/// Extracts a tar.gz archive to the destination directory.
#[allow(dead_code)]
fn extract_tar_gz(data: &[u8], dest: &Path) -> Result<(), Box<dyn Error>> {
    use flate2::read::GzDecoder;
    use tar::Archive;

    create_dir_all(dest)?;
    let decoder = GzDecoder::new(data);
    let mut archive = Archive::new(decoder);
    archive.unpack(dest)?;
    Ok(())
}

/// Extracts a zip archive to the destination directory.
#[allow(dead_code)]
fn extract_zip(data: &[u8], dest: &Path) -> Result<(), Box<dyn Error>> {
    use std::io::Cursor;

    use zip::ZipArchive;

    create_dir_all(dest)?;
    let reader = Cursor::new(data);
    let mut archive = ZipArchive::new(reader)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();

        // Skip directories and non-ttf/otf files
        if file.is_dir() {
            continue;
        }

        // Extract only font files
        if let Some(filename) = Path::new(&name).file_name() {
            let filename_str = filename.to_string_lossy();
            if filename_str.ends_with(".ttf")
                || filename_str.ends_with(".otf")
                || filename_str.ends_with(".otc")
            {
                let outpath = dest.join(filename);
                let mut outfile = File::create(&outpath)?;
                copy(&mut file, &mut outfile)?;
            }
        }
    }
    Ok(())
}

/// Downloads and caches a font archive if not already cached.
/// Returns the path to the cached font directory.
#[allow(dead_code)]
fn download_font(
    name: &str,
    url: &str,
    archive_type: &ArchiveType,
) -> Result<PathBuf, Box<dyn Error>> {
    let cache = cache_dir().join(name);

    // Check if already cached (directory exists and has files)
    if cache.exists() && read_dir(&cache)?.next().is_some() {
        println!("cargo::warning=Using cached fonts from {}", cache.display());
        return Ok(cache);
    }

    // Download and extract
    let data = download(url)?;
    match archive_type {
        ArchiveType::TarGz => extract_tar_gz(&data, &cache)?,
        ArchiveType::Zip => extract_zip(&data, &cache)?,
    }

    println!("cargo::warning=Cached fonts to {}", cache.display());
    Ok(cache)
}

#[allow(dead_code)]
enum ArchiveType {
    TarGz,
    Zip,
}

/// Generates an include file for embedding fonts from a directory.
#[allow(dead_code)]
fn generate_font_includes(
    out_dir: &Path,
    feature_name: &str,
    font_dir: &Path,
    files: &[&str],
) -> Result<(), Box<dyn Error>> {
    let include_file = out_dir.join(format!("embed_{feature_name}.rs"));
    let mut f = File::create(&include_file)?;

    writeln!(f, "{{")?;
    for file in files {
        let font_path = font_dir.join(file);
        let path_str = font_path.display();
        writeln!(f, "    process(include_bytes!(\"{path_str}\"));")?;
    }
    writeln!(f, "}}")?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = PathBuf::from(var("OUT_DIR")?);

    // Generate version.rs
    let mut f = File::create(out_dir.join("version.rs"))?;
    let ProjectMetadata {
        package: Package { typwriter_version },
        dependencies: Dependencies { typst: Typst { typst_version } },
    } = from_str(&read_to_string("Cargo.toml")?)?;

    write!(
        f,
        r#"/// Returns the version of the library.
///
/// # Example
///
/// ```rust
/// println!("Typwriter version: {{}}", typwriter::version());
/// ```
pub fn version() -> &'static str {{ "{typwriter_version}" }}

/// Returns the Typst version the library was compiled with.
///
/// # Example
///
/// ```rust
/// println!("Typst version: {{}}", typwriter::typst_version());
/// ```
pub fn typst_version() -> &'static str {{ "{typst_version}" }}
"#,
    )?;

    // Download and generate includes for large fonts based on features
    #[cfg(feature = "embed_warpnine_mono")]
    {
        let font_dir = download_font(
            "WarpnineFonts",
            "https://github.com/0x6b/warpnine-fonts/releases/download/v2026-01-11.1/warpnine-fonts-2026-01-11.1.zip",
            &ArchiveType::Zip,
        )?;
        generate_font_includes(
            &out_dir,
            "warpnine_mono",
            &font_dir,
            &[
                "WarpnineMono-Black.ttf",
                "WarpnineMono-BlackItalic.ttf",
                "WarpnineMono-Bold.ttf",
                "WarpnineMono-BoldItalic.ttf",
                "WarpnineMono-ExtraBlack.ttf",
                "WarpnineMono-ExtraBlackItalic.ttf",
                "WarpnineMono-ExtraBold.ttf",
                "WarpnineMono-ExtraBoldItalic.ttf",
                "WarpnineMono-Italic.ttf",
                "WarpnineMono-Light.ttf",
                "WarpnineMono-LightItalic.ttf",
                "WarpnineMono-Medium.ttf",
                "WarpnineMono-MediumItalic.ttf",
                "WarpnineMono-Regular.ttf",
                "WarpnineMono-SemiBold.ttf",
                "WarpnineMono-SemiBoldItalic.ttf",
            ],
        )?;
    }

    #[cfg(feature = "embed_warpnine_sans")]
    {
        let font_dir = download_font(
            "WarpnineFonts",
            "https://github.com/0x6b/warpnine-fonts/releases/download/v2026-05-03.1/warpnine-fonts-2026-05-03.1.zip",
            &ArchiveType::Zip,
        )?;
        generate_font_includes(
            &out_dir,
            "warpnine_sans",
            &font_dir,
            &[
                "WarpnineSans-Black.ttf",
                "WarpnineSans-BlackItalic.ttf",
                "WarpnineSans-Bold.ttf",
                "WarpnineSans-BoldItalic.ttf",
                "WarpnineSans-ExtraBold.ttf",
                "WarpnineSans-ExtraBoldItalic.ttf",
                "WarpnineSans-Italic.ttf",
                "WarpnineSans-Light.ttf",
                "WarpnineSans-LightItalic.ttf",
                "WarpnineSans-Medium.ttf",
                "WarpnineSans-MediumItalic.ttf",
                "WarpnineSans-Regular.ttf",
                "WarpnineSans-SemiBold.ttf",
                "WarpnineSans-SemiBoldItalic.ttf",
                "WarpnineSansCondensed-Black.ttf",
                "WarpnineSansCondensed-BlackItalic.ttf",
                "WarpnineSansCondensed-Bold.ttf",
                "WarpnineSansCondensed-BoldItalic.ttf",
                "WarpnineSansCondensed-ExtraBold.ttf",
                "WarpnineSansCondensed-ExtraBoldItalic.ttf",
                "WarpnineSansCondensed-Italic.ttf",
                "WarpnineSansCondensed-Light.ttf",
                "WarpnineSansCondensed-LightItalic.ttf",
                "WarpnineSansCondensed-Medium.ttf",
                "WarpnineSansCondensed-MediumItalic.ttf",
                "WarpnineSansCondensed-Regular.ttf",
                "WarpnineSansCondensed-SemiBold.ttf",
                "WarpnineSansCondensed-SemiBoldItalic.ttf",
            ],
        )?;
    }

    #[cfg(feature = "embed_noto_sans_jp")]
    {
        let font_dir = download_font(
            "NotoSansJP",
            "https://github.com/notofonts/noto-cjk/releases/download/Sans2.004/16_NotoSansJP.zip",
            &ArchiveType::Zip,
        )?;
        generate_font_includes(
            &out_dir,
            "noto_sans_jp",
            &font_dir,
            &[
                "NotoSansJP-Black.otf",
                "NotoSansJP-Bold.otf",
                "NotoSansJP-DemiLight.otf",
                "NotoSansJP-Light.otf",
                "NotoSansJP-Medium.otf",
                "NotoSansJP-Regular.otf",
                "NotoSansJP-Thin.otf",
            ],
        )?;
    }

    #[cfg(feature = "embed_noto_serif_jp")]
    {
        let font_dir = download_font(
            "NotoSerifJP",
            "https://github.com/notofonts/noto-cjk/releases/download/Serif2.003/12_NotoSerifJP.zip",
            &ArchiveType::Zip,
        )?;
        generate_font_includes(
            &out_dir,
            "noto_serif_jp",
            &font_dir,
            &[
                "NotoSerifJP-Black.otf",
                "NotoSerifJP-Bold.otf",
                "NotoSerifJP-ExtraLight.otf",
                "NotoSerifJP-Light.otf",
                "NotoSerifJP-Medium.otf",
                "NotoSerifJP-Regular.otf",
                "NotoSerifJP-SemiBold.otf",
            ],
        )?;
    }

    #[cfg(feature = "embed_jet_brains_mono_nl")]
    {
        let cache = cache_dir().join("JetBrainsMonoNL");
        let files = [
            "JetBrainsMonoNL-Bold.ttf",
            "JetBrainsMonoNL-BoldItalic.ttf",
            "JetBrainsMonoNL-ExtraBold.ttf",
            "JetBrainsMonoNL-ExtraBoldItalic.ttf",
            "JetBrainsMonoNL-ExtraLight.ttf",
            "JetBrainsMonoNL-ExtraLightItalic.ttf",
            "JetBrainsMonoNL-Italic.ttf",
            "JetBrainsMonoNL-Light.ttf",
            "JetBrainsMonoNL-LightItalic.ttf",
            "JetBrainsMonoNL-Medium.ttf",
            "JetBrainsMonoNL-MediumItalic.ttf",
            "JetBrainsMonoNL-Regular.ttf",
            "JetBrainsMonoNL-SemiBold.ttf",
            "JetBrainsMonoNL-SemiBoldItalic.ttf",
            "JetBrainsMonoNL-Thin.ttf",
            "JetBrainsMonoNL-ThinItalic.ttf",
        ];
        let cached = cache.exists() && read_dir(&cache)?.next().is_some();
        if cached {
            println!("cargo::warning=Using cached fonts from {}", cache.display());
        } else {
            create_dir_all(&cache)?;
            let base = "https://raw.githubusercontent.com/JetBrains/JetBrainsMono/master/fonts/archives/ttf";
            for file in &files {
                let data = download(&format!("{base}/{file}"))?;
                File::create(cache.join(file))?.write_all(&data)?;
            }
            println!("cargo::warning=Cached fonts to {}", cache.display());
        }
        generate_font_includes(&out_dir, "jet_brains_mono_nl", &cache, &files)?;
    }

    #[cfg(feature = "embed_recursive")]
    {
        let font_dir = download_font(
            "Recursive",
            "https://github.com/arrowtype/recursive/releases/download/v1.085/ArrowType-Recursive-1.085.zip",
            &ArchiveType::Zip,
        )?;
        generate_font_includes(&out_dir, "recursive", &font_dir, &["recursive-static-OTFs.otc"])?;
    }

    Ok(())
}
