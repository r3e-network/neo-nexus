use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotApplication {
    pub snapshot_path: PathBuf,
    pub import_dir: PathBuf,
    pub manifest_path: PathBuf,
    pub import_mode: SnapshotImportMode,
    pub imported_files: usize,
    pub expanded_bytes: u64,
    pub sha256: String,
    pub bytes: u64,
    pub applied_at_unix: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotImportMode {
    RawFile,
    TarArchive,
    TarGzipArchive,
    ZipArchive,
}

impl SnapshotImportMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::RawFile => "raw file",
            Self::TarArchive => "tar archive",
            Self::TarGzipArchive => "tar.gz archive",
            Self::ZipArchive => "zip archive",
        }
    }

    pub(super) fn manifest_value(self) -> &'static str {
        match self {
            Self::RawFile => "raw-file",
            Self::TarArchive => "tar",
            Self::TarGzipArchive => "tar-gzip",
            Self::ZipArchive => "zip",
        }
    }
}
