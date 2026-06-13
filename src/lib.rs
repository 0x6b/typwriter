//! A naive Rust Library that provides a way to work with [Typst](https://typst.app/) document and PDF file programmatically.
//!
//! # Overview
//!
//! You can use this library to:
//!
//! - [compile](compile()) a Typst file to a PDF or PNG file
//! - [format](format()) a Typst file
//! - [update metadata](update_metadata()) of a PDF file
//! - [set permission](set_permission()) of a PDF file
//! - [watch](watch()) for changes in the input Typst file along with its dependencies and recompile
//!   it when a change is detected
//!
//! # Supported Typst Version
//!
//! Version [0.15.0-rc.1](https://github.com/typst/typst/releases/tag/v0.15.0-rc.1)
//!
//! This crate is for my personal use and Typst/Rust learning purposes; it is not affiliated with the [Typst](https://typst.app/) project.
//!
//! # Feature flags
//!
//! Below is a list of available feature flags. The crate does not define a `default` feature, so if
//! you're unsure what you need, specify `full`, which enables all capabilities and fonts.
//! However, be aware that this will result in longer compilation times and a larger binary size.
//!
//! ## Capabilities
//!
//! - `compile`: Enables the [`compile()`], [`list_fonts()`] functions, and [`PdfStandard`] enum.
//! - `format`: Enables the [`format()`] function.
//! - `pdf_metadata`: Enables the [`update_metadata()`] function.
//! - `pdf_permission`: Enables the [`set_permission()`] function.
//! - `watch`: Enables the [`watch()`] function. This feature also enables the `compile` feature.
//!
//! ## Fonts Embedding
//!
//! - `embed_additional_fonts`: embed all fonts listed below.
//! - `embed_cmu_roman`: [Computer Modern Roman](https://www.fontsquirrel.com/fonts/computer-modern)
//! - `embed_ia_writer_duo`: [iA Writer Duo](https://github.com/iaolo/iA-Fonts/)
//! - `embed_jet_brains_mono_nl`: [JetBrains Mono NL](https://github.com/JetBrains/JetBrainsMono)
//! - `embed_noto_emoji`: [Noto Emoji](https://fonts.google.com/noto/specimen/Noto+Emoji)
//! - `embed_noto_sans_jp`: [Noto Sans JP](https://fonts.google.com/noto/specimen/Noto+Sans+JP)
//! - `embed_noto_serif_jp`: [Noto Serif JP](https://fonts.google.com/noto/specimen/Noto+Serif+JP)
//! - `embed_recursive`: [Recursive Sans & Mono](https://github.com/arrowtype/recursive/)
//! - `embed_source_code_pro`: [Source Code Pro](https://fonts.google.com/specimen/Source+Code+Pro)
//! - `embed_warpnine_mono`: [Warpnine Mono](https://github.com/0x6b/warpnine-fonts)
//! - `embed_warpnine_sans`: [Warpnine Sans Condensed](https://github.com/0x6b/warpnine-fonts)
//!
//! Note that:
//!
//! - typst-cli [defaults](https://github.com/typst/typst-assets/blob/5ca2a6996da97dcba893247576a4a70bbbae8a7a/src/lib.rs#L67-L80)
//!   are always embedded.
//! - The crate won’t search system fonts to ensure the reproducibility. All fonts you need should
//!   be explicitly added via [`CompileParams::font_paths`].

#[cfg(feature = "compile")]
pub use compile::{CompileParams, PdfStandard, compile};
#[cfg(feature = "compile")]
pub use fonts::list_fonts;
#[cfg(feature = "format")]
pub use format::{FormatParams, format};
#[cfg(feature = "pdf_permission")]
pub use set_permission::{PermissionParams, PrintPermission, set_permission};
#[cfg(feature = "pdf_metadata")]
pub use update_metadata::{PdfMetadata, update_metadata};
pub use version::{typst_version, version};
#[cfg(feature = "watch")]
pub use watch::FittingType;
#[cfg(feature = "watch")]
pub use watch::watch;

#[cfg(feature = "compile")]
mod compile;
#[cfg(feature = "compile")]
mod fonts;
#[cfg(feature = "format")]
mod format;
#[cfg(feature = "pdf_permission")]
mod set_permission;
#[cfg(feature = "pdf_metadata")]
mod update_metadata;
mod version;
#[cfg(feature = "watch")]
mod watch;
#[cfg(feature = "compile")]
mod world;
