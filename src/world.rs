use std::{
    error,
    fmt::{self, Display, Formatter},
    io,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use ecow::{EcoString, eco_format};
use typst::{
    Library, LibraryExt, World,
    diag::FileResult,
    foundations::{Bytes, Datetime, Dict, Duration, IntoValue},
    syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot, VirtualizeError},
    text::{Font, FontBook},
    utils::LazyHash,
};
use typst_kit::{
    datetime::Time,
    downloader::SystemDownloader,
    files::{FileStore, FsRoot, SystemFiles},
    fonts::FontStore,
    packages::{FsPackages, SystemPackages, UniversePackages},
};

use crate::fonts::discover_fonts;

/// A world that provides access to the operating system.
pub struct SystemWorld {
    /// The id of the main source file.
    main: FileId,
    /// Typst's standard library.
    library: LazyHash<Library>,
    /// Discovered and lazily loaded fonts.
    fonts: FontStore,
    /// Source files and buffers, keyed by file id.
    files: FileStore<SystemFiles>,
    /// The current datetime, fixed within a single compilation.
    now: Time,
}

impl SystemWorld {
    /// Create a new system world.
    pub fn new(
        input: &Path,
        font_paths: &[PathBuf],
        inputs: &[(String, String)],
        package_path: &Option<PathBuf>,
        package_cache_path: &Option<PathBuf>,
    ) -> Result<Self, WorldCreationError> {
        // Resolve the input path.
        let input = input.canonicalize().map_err(|err| match err.kind() {
            ErrorKind::NotFound => WorldCreationError::InputNotFound(input.to_path_buf()),
            _ => WorldCreationError::Io(err),
        })?;

        // Resolve the root directory (the input file's parent).
        let root =
            input
                .parent()
                .unwrap_or(Path::new("."))
                .canonicalize()
                .map_err(|err| match err.kind() {
                    ErrorKind::NotFound => WorldCreationError::RootNotFound(input.clone()),
                    _ => WorldCreationError::Io(err),
                })?;

        // Resolve the virtual path of the main file within the project root.
        let main =
            RootedPath::new(VirtualRoot::Project, VirtualPath::virtualize(&root, &input)?).intern();

        // Build package storage honoring optional custom paths.
        let downloader = SystemDownloader::new(concat!("typwriter/", env!("CARGO_PKG_VERSION")));
        let data = package_path
            .clone()
            .map(FsPackages::new)
            .or_else(FsPackages::system_data);
        let cache = package_cache_path
            .clone()
            .map(FsPackages::new)
            .or_else(FsPackages::system_cache);
        let packages = SystemPackages::from_parts(data, cache, UniversePackages::new(downloader));

        let files = FileStore::new(SystemFiles::new(FsRoot::new(root), packages));

        // Convert the input pairs to a dictionary.
        let inputs: Dict = inputs
            .iter()
            .map(|(k, v)| (k.as_str().into(), v.as_str().into_value()))
            .collect();
        let library = Library::builder().with_inputs(inputs).build();

        Ok(Self {
            main,
            library: LazyHash::new(library),
            fonts: discover_fonts(font_paths),
            files,
            now: Time::system(),
        })
    }
}

impl World for SystemWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        self.fonts.book()
    }

    fn main(&self) -> FileId {
        self.main
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        self.files.source(id)
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.files.file(id)
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.font(index)
    }

    fn today(&self, offset: Option<Duration>) -> Option<Datetime> {
        self.now.today(offset)
    }
}

/// An error that occurs during world construction.
#[derive(Debug)]
pub enum WorldCreationError {
    /// The input file does not appear to exist.
    InputNotFound(PathBuf),
    /// The input file path was malformed or escapes the project root.
    InputMalformed(VirtualizeError),
    /// The root directory does not appear to exist.
    RootNotFound(PathBuf),
    /// Another type of I/O error.
    Io(io::Error),
}

impl Display for WorldCreationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            WorldCreationError::InputNotFound(path) => {
                write!(f, "input file not found (searched at {})", path.display())
            }
            WorldCreationError::InputMalformed(_) => {
                write!(f, "source file must be contained in project root")
            }
            WorldCreationError::RootNotFound(path) => {
                write!(f, "root directory not found (searched at {})", path.display())
            }
            WorldCreationError::Io(err) => write!(f, "{err}"),
        }
    }
}

impl error::Error for WorldCreationError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<VirtualizeError> for WorldCreationError {
    fn from(err: VirtualizeError) -> Self {
        Self::InputMalformed(err)
    }
}

impl From<WorldCreationError> for EcoString {
    fn from(err: WorldCreationError) -> Self {
        eco_format!("{err}")
    }
}
