use std::{collections::HashMap, fs, path::PathBuf, sync::OnceLock};

use fontdb::{Database, Source};
use typst::{
    foundations::Bytes,
    text::{Font, FontBook, FontInfo},
};

/// Searches for fonts.
pub struct FontSearcher {
    /// Metadata about all discovered fonts.
    pub book: FontBook,
    /// Slots that the fonts are loaded into.
    pub fonts: Vec<FontSlot>,
}

/// Holds details about the location of a font and lazily the font itself.
pub struct FontSlot {
    /// The path at which the font can be found on the system.
    path: PathBuf,
    /// The index of the font in its collection. Zero if the path does not point
    /// to a collection.
    index: u32,
    /// The lazily loaded font.
    font: OnceLock<Option<Font>>,
}

impl FontSlot {
    /// Get the font for this slot.
    pub fn get(&self) -> Option<Font> {
        self.font
            .get_or_init(|| {
                let data = read(&self.path).ok()?;
                Font::new(Bytes::new(data), self.index)
            })
            .clone()
    }
}

impl FontSearcher {
    /// Create a new, empty system searcher.
    pub fn new() -> Self {
        Self { book: FontBook::new(), fonts: vec![] }
    }

    /// Search everything that is available.
    pub fn search(&mut self, font_paths: &[PathBuf]) {
        let mut db = Database::new();

        // Font paths have highest priority.
        for path in font_paths {
            db.load_fonts_dir(path);
        }

        for face in db.faces() {
            let path = match &face.source {
                Source::File(path) | Source::SharedFile(path, _) => path,
                // We never add binary sources to the database, so there
                // shouldn't be any.
                Source::Binary(_) => continue,
            };

            let info = db
                .with_face_data(face.id, FontInfo::new)
                .expect("database must contain this font");

            if let Some(info) = info {
                self.book.push(info);
                self.fonts.push(FontSlot {
                    path: path.clone(),
                    index: face.index,
                    font: OnceLock::new(),
                });
            }
        }

        self.add_embedded();
    }

    /// Add fonts that are embedded in the binary.
    fn add_embedded(&mut self) {
        let mut process = |bytes: &'static [u8]| {
            let buffer = Bytes::new(bytes);
            for (i, font) in Font::iter(buffer).enumerate() {
                self.book.push(font.info().clone());
                self.fonts.push(FontSlot {
                    path: PathBuf::new(),
                    index: i as u32,
                    font: OnceLock::from(Some(font)),
                });
            }
        };

        // Always embed the typst default fonts.
        for data in fonts() {
            process(data);
        }

        #[cfg(any(
            feature = "embed_cmu_roman",
            feature = "embed_ia_writer_duo",
            feature = "embed_noto_emoji",
            feature = "embed_source_code_pro",
        ))]
        macro_rules! add {
            ($filename:literal) => {
                process(include_bytes!(concat!("../assets/fonts/", $filename)));
            };
        }

        #[cfg(feature = "embed_cmu_roman")]
        {
            add!("ComputerModern/cmunrm.ttf");
        }
        #[cfg(feature = "embed_ia_writer_duo")]
        {
            add!("iAWriterDuo/iAWriterDuoS-Bold.ttf");
            add!("iAWriterDuo/iAWriterDuoS-BoldItalic.ttf");
            add!("iAWriterDuo/iAWriterDuoS-Italic.ttf");
            add!("iAWriterDuo/iAWriterDuoS-Regular.ttf");
        }
        #[cfg(feature = "embed_noto_emoji")]
        {
            add!("NotoEmoji/NotoEmoji-VariableFont_wght.ttf");
        }
        #[cfg(feature = "embed_jet_brains_mono_nl")]
        {
            include!(concat!(env!("OUT_DIR"), "/embed_jet_brains_mono_nl.rs"));
        }
        #[cfg(feature = "embed_noto_sans_jp")]
        {
            include!(concat!(env!("OUT_DIR"), "/embed_noto_sans_jp.rs"));
        }
        #[cfg(feature = "embed_noto_serif_jp")]
        {
            include!(concat!(env!("OUT_DIR"), "/embed_noto_serif_jp.rs"));
        }
        #[cfg(feature = "embed_recursive")]
        {
            include!(concat!(env!("OUT_DIR"), "/embed_recursive.rs"));
        }
        #[cfg(feature = "embed_source_code_pro")]
        {
            add!("SourceCodePro/SourceCodePro-Black.ttf");
            add!("SourceCodePro/SourceCodePro-BlackItalic.ttf");
            add!("SourceCodePro/SourceCodePro-Bold.ttf");
            add!("SourceCodePro/SourceCodePro-BoldItalic.ttf");
            add!("SourceCodePro/SourceCodePro-ExtraBold.ttf");
            add!("SourceCodePro/SourceCodePro-ExtraBoldItalic.ttf");
            add!("SourceCodePro/SourceCodePro-ExtraLight.ttf");
            add!("SourceCodePro/SourceCodePro-ExtraLightItalic.ttf");
            add!("SourceCodePro/SourceCodePro-Italic.ttf");
            add!("SourceCodePro/SourceCodePro-Light.ttf");
            add!("SourceCodePro/SourceCodePro-LightItalic.ttf");
            add!("SourceCodePro/SourceCodePro-Medium.ttf");
            add!("SourceCodePro/SourceCodePro-MediumItalic.ttf");
            add!("SourceCodePro/SourceCodePro-Regular.ttf");
            add!("SourceCodePro/SourceCodePro-SemiBold.ttf");
            add!("SourceCodePro/SourceCodePro-SemiBoldItalic.ttf");
        }
        #[cfg(feature = "embed_warpnine_mono")]
        {
            include!(concat!(env!("OUT_DIR"), "/embed_warpnine_mono.rs"));
        }
        #[cfg(feature = "embed_warpnine_sans")]
        {
            include!(concat!(env!("OUT_DIR"), "/embed_warpnine_sans.rs"));
        }
    }
}

use fs::read;
use typst_assets::fonts;

#[allow(unused_imports)]
use crate::CompileParams; // For documentation purposes.

/// Lists all fonts available for the library.
///
/// Note that:
///
/// - typst-cli [defaults](https://github.com/typst/typst-assets/blob/5ca2a6996da97dcba893247576a4a70bbbae8a7a/src/lib.rs#L67-L80)
///   are always embedded.
/// - The crate won't search system fonts to ensure the reproducibility. All fonts you need should
///   be explicitly added via [`CompileParams::font_paths`].
///
/// # Arguments
///
/// - `font_paths` - Paths to additional font directories.
///
/// # Returns
///
/// A [`Vec`] of [`FontInfo`] structs.
///
/// # Example
///
/// ```rust
/// // List fonts with no additional font paths (only embedded fonts)
/// typwriter::list_fonts(&[])
///     .iter()
///     .for_each(|(family, _)| println!("{family}"));
///
/// // List fonts with additional font directories
/// typwriter::list_fonts(&["assets/fonts".into()])
///     .iter()
///     .for_each(|(family, _)| println!("{family}"));
/// ```
pub fn list_fonts(font_paths: &[PathBuf]) -> HashMap<String, Vec<FontInfo>> {
    let mut searcher = FontSearcher::new();
    searcher.search(font_paths);
    searcher
        .book
        .families()
        .map(|(family, infos)| (family.to_string(), infos.cloned().collect::<Vec<FontInfo>>()))
        .collect::<HashMap<String, Vec<FontInfo>>>()
}
