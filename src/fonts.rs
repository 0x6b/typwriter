use std::{collections::HashMap, path::PathBuf};

use typst::{
    foundations::Bytes,
    text::{Font, FontInfo},
};
use typst_kit::fonts::{FontStore, embedded, scan};

#[allow(unused_imports)]
use crate::CompileParams; // For documentation purposes.

/// Discovers fonts: font directories first, then the embedded typst defaults
/// and any fonts compiled in via the `embed_*` features.
///
/// System fonts are intentionally never searched, to keep output reproducible.
/// All extra fonts must be supplied explicitly via `font_paths`.
pub fn discover_fonts(font_paths: &[PathBuf]) -> FontStore {
    let mut store = FontStore::new();

    // Font paths have highest priority.
    for path in font_paths {
        store.extend(scan(path));
    }

    add_embedded(&mut store);
    store
}

/// Add fonts that are embedded in the binary.
fn add_embedded(store: &mut FontStore) {
    // Always embed the typst default fonts.
    store.extend(embedded());

    let mut process = |bytes: &'static [u8]| {
        store.extend(Font::iter(Bytes::new(bytes)).map(|font| {
            let info = font.info().clone();
            (font, info)
        }));
    };

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
        // Variable fonts (wght axis) replace the former static instances; one
        // file each for upright and italic. Family name stays "Source Code Pro".
        add!("SourceCodePro/SourceCodePro-VF.ttf");
        add!("SourceCodePro/SourceCodePro-Italic-VF.ttf");
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
/// A [`HashMap`] from family name to its [`FontInfo`] variants.
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
    let store = discover_fonts(font_paths);
    let book = store.book();
    book.families()
        .map(|(family, indices)| {
            let infos = indices
                .filter_map(|index| book.info(index).cloned())
                .collect::<Vec<FontInfo>>();
            (family.to_string(), infos)
        })
        .collect::<HashMap<String, Vec<FontInfo>>>()
}
