use std::{
    collections::HashMap,
    error::Error,
    fmt,
    fs::{self, File},
    io::{self, Read},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use uuid::Uuid;

/// Minimum quiet period required before a watched PDF can enter intake.
pub const STABLE_WRITE_DEBOUNCE: Duration = Duration::from_secs(1);

const PDF_HEADER: &[u8; 5] = b"%PDF-";

/// Rust-generated identity for one pending PDF intake candidate.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PdfImportId(Uuid);

/// Provenance of one PDF candidate before metadata processing begins.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PdfImportSource {
    Explicit,
    WatchedFolder,
}

/// Validated Rust-only candidate ready for a later persistent import job.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingPdfImport {
    import_id: PdfImportId,
    source: PdfImportSource,
    path: PathBuf,
    byte_length: u64,
}

/// Result of checking one previously observed watched PDF.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WatchedPdfIntakeStatus {
    Waiting,
    Pending(PendingPdfImport),
}

/// Bounded failures from explicit or watched PDF intake.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PdfImportError {
    UnsupportedFileType,
    FileUnavailable,
    NotRegularFile,
    SymbolicLinkNotAllowed,
    FileMetadataUnavailable,
    FileReadFailed,
    InvalidPdf,
    FileChangedDuringValidation,
    InvalidWatchedFolder,
    OutsideWatchedFolder,
    FileNotObserved,
    ObservationOutOfOrder,
}

/// Root-confined stable-write intake for filesystem observations supplied by Rust.
pub struct WatchedPdfIntake {
    watched_root: PathBuf,
    observations: HashMap<PathBuf, FileObservation>,
}

/// Validates one explicitly approved PDF path into a pending candidate.
pub fn prepare_explicit_pdf(path: impl AsRef<Path>) -> Result<PendingPdfImport, PdfImportError> {
    let path = canonical_pdf_file_path(path.as_ref())?;
    let snapshot = file_snapshot(&path)?;
    match inspect_pdf(&path, snapshot)? {
        PdfInspection::Stable(byte_length) => {
            Ok(pending_import(PdfImportSource::Explicit, path, byte_length))
        }
        PdfInspection::Changed(_) => Err(PdfImportError::FileChangedDuringValidation),
    }
}

impl WatchedPdfIntake {
    /// Creates an intake gate rooted to one existing non-symlink directory.
    pub fn new(watched_root: impl AsRef<Path>) -> Result<Self, PdfImportError> {
        Ok(Self {
            watched_root: canonical_watched_root(watched_root.as_ref())?,
            observations: HashMap::new(),
        })
    }

    /// Records a file-change event and resets its stable-write deadline.
    pub fn record_change(
        &mut self,
        path: impl AsRef<Path>,
        observed_at: Instant,
    ) -> Result<(), PdfImportError> {
        let path = self.watched_pdf_path(path.as_ref())?;
        let snapshot = file_snapshot(&path)?;
        self.record_snapshot(path, snapshot, observed_at);
        Ok(())
    }

    /// Confirms a previously observed file only after quiet and stable-size checks.
    pub fn confirm_stable(
        &mut self,
        path: impl AsRef<Path>,
        observed_at: Instant,
    ) -> Result<WatchedPdfIntakeStatus, PdfImportError> {
        let path = self.watched_pdf_path(path.as_ref())?;
        let snapshot = file_snapshot(&path)?;
        if !self.observation_is_ready(&path, snapshot, observed_at)? {
            return Ok(WatchedPdfIntakeStatus::Waiting);
        }
        self.finish_watched_inspection(path, snapshot, observed_at)
    }

    fn watched_pdf_path(&self, path: &Path) -> Result<PathBuf, PdfImportError> {
        let path = canonical_pdf_file_path(path)?;
        if path.starts_with(&self.watched_root) {
            Ok(path)
        } else {
            Err(PdfImportError::OutsideWatchedFolder)
        }
    }

    fn observation_is_ready(
        &mut self,
        path: &Path,
        snapshot: FileSnapshot,
        observed_at: Instant,
    ) -> Result<bool, PdfImportError> {
        let observation = self
            .observations
            .get(path)
            .copied()
            .ok_or(PdfImportError::FileNotObserved)?;
        let elapsed = observed_at
            .checked_duration_since(observation.changed_at)
            .ok_or(PdfImportError::ObservationOutOfOrder)?;
        if snapshot != observation.snapshot {
            self.record_snapshot(path.to_owned(), snapshot, observed_at);
            return Ok(false);
        }
        Ok(elapsed >= STABLE_WRITE_DEBOUNCE)
    }

    fn finish_watched_inspection(
        &mut self,
        path: PathBuf,
        snapshot: FileSnapshot,
        observed_at: Instant,
    ) -> Result<WatchedPdfIntakeStatus, PdfImportError> {
        match inspect_pdf(&path, snapshot)? {
            PdfInspection::Stable(byte_length) => {
                self.observations.remove(&path);
                Ok(WatchedPdfIntakeStatus::Pending(pending_import(
                    PdfImportSource::WatchedFolder,
                    path,
                    byte_length,
                )))
            }
            PdfInspection::Changed(snapshot) => {
                self.record_snapshot(path, snapshot, observed_at);
                Ok(WatchedPdfIntakeStatus::Waiting)
            }
        }
    }

    fn record_snapshot(&mut self, path: PathBuf, snapshot: FileSnapshot, changed_at: Instant) {
        self.observations.insert(
            path,
            FileObservation {
                snapshot,
                changed_at,
            },
        );
    }
}

impl PdfImportId {
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl PendingPdfImport {
    pub fn import_id(&self) -> PdfImportId {
        self.import_id
    }

    pub fn source(&self) -> PdfImportSource {
        self.source
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn byte_length(&self) -> u64 {
        self.byte_length
    }
}

impl fmt::Display for PdfImportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::UnsupportedFileType => "file is not a PDF",
            Self::FileUnavailable => "PDF file is unavailable",
            Self::NotRegularFile => "PDF path is not a regular file",
            Self::SymbolicLinkNotAllowed => "PDF symbolic links are not allowed",
            Self::FileMetadataUnavailable => "PDF metadata could not be read",
            Self::FileReadFailed => "PDF file could not be read",
            Self::InvalidPdf => "PDF signature is invalid",
            Self::FileChangedDuringValidation => "PDF changed during validation",
            Self::InvalidWatchedFolder => "watched folder is invalid",
            Self::OutsideWatchedFolder => "PDF is outside the watched folder",
            Self::FileNotObserved => "PDF has no recorded change observation",
            Self::ObservationOutOfOrder => "PDF observation time is out of order",
        })
    }
}

impl Error for PdfImportError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FileSnapshot {
    byte_length: u64,
}

#[derive(Clone, Copy, Debug)]
struct FileObservation {
    snapshot: FileSnapshot,
    changed_at: Instant,
}

enum PdfInspection {
    Stable(u64),
    Changed(FileSnapshot),
}

fn pending_import(source: PdfImportSource, path: PathBuf, byte_length: u64) -> PendingPdfImport {
    PendingPdfImport {
        import_id: PdfImportId(Uuid::new_v4()),
        source,
        path,
        byte_length,
    }
}

fn canonical_watched_root(path: &Path) -> Result<PathBuf, PdfImportError> {
    let metadata = fs::symlink_metadata(path).map_err(|_| PdfImportError::InvalidWatchedFolder)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(PdfImportError::InvalidWatchedFolder);
    }
    fs::canonicalize(path).map_err(|_| PdfImportError::InvalidWatchedFolder)
}

fn canonical_pdf_file_path(path: &Path) -> Result<PathBuf, PdfImportError> {
    require_pdf_extension(path)?;
    let metadata = pdf_path_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Err(PdfImportError::SymbolicLinkNotAllowed);
    }
    if !metadata.is_file() {
        return Err(PdfImportError::NotRegularFile);
    }
    fs::canonicalize(path).map_err(|_| PdfImportError::FileMetadataUnavailable)
}

fn require_pdf_extension(path: &Path) -> Result<(), PdfImportError> {
    let is_pdf = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("pdf"));
    if is_pdf {
        Ok(())
    } else {
        Err(PdfImportError::UnsupportedFileType)
    }
}

fn pdf_path_metadata(path: &Path) -> Result<fs::Metadata, PdfImportError> {
    fs::symlink_metadata(path).map_err(|error| match error.kind() {
        io::ErrorKind::NotFound => PdfImportError::FileUnavailable,
        _ => PdfImportError::FileMetadataUnavailable,
    })
}

fn file_snapshot(path: &Path) -> Result<FileSnapshot, PdfImportError> {
    fs::metadata(path)
        .map(|metadata| FileSnapshot {
            byte_length: metadata.len(),
        })
        .map_err(|_| PdfImportError::FileMetadataUnavailable)
}

fn inspect_pdf(path: &Path, expected: FileSnapshot) -> Result<PdfInspection, PdfImportError> {
    let mut file = open_pdf_file(path)?;
    let before = snapshot_from_file(&file)?;
    if before != expected {
        return Ok(PdfInspection::Changed(before));
    }
    let header = read_pdf_header(&mut file)?;
    let after = snapshot_from_file(&file)?;
    if after != before {
        return Ok(PdfInspection::Changed(after));
    }
    if &header != PDF_HEADER {
        return Err(PdfImportError::InvalidPdf);
    }
    Ok(PdfInspection::Stable(after.byte_length))
}

fn open_pdf_file(path: &Path) -> Result<File, PdfImportError> {
    File::open(path).map_err(|error| match error.kind() {
        io::ErrorKind::NotFound => PdfImportError::FileUnavailable,
        _ => PdfImportError::FileReadFailed,
    })
}

fn snapshot_from_file(file: &File) -> Result<FileSnapshot, PdfImportError> {
    file.metadata()
        .map(|metadata| FileSnapshot {
            byte_length: metadata.len(),
        })
        .map_err(|_| PdfImportError::FileMetadataUnavailable)
}

fn read_pdf_header(file: &mut File) -> Result<[u8; 5], PdfImportError> {
    let mut header = [0; 5];
    file.read_exact(&mut header).map_err(|error| {
        if error.kind() == io::ErrorKind::UnexpectedEof {
            PdfImportError::InvalidPdf
        } else {
            PdfImportError::FileReadFailed
        }
    })?;
    Ok(header)
}

#[cfg(test)]
#[path = "pdf_tests.rs"]
mod tests;
