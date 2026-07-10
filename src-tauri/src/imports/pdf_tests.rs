use std::{
    fs::{self, OpenOptions},
    io::Write,
};

use super::*;

const COMPLETE_PDF: &[u8] = b"%PDF-1.7\nbody\n%%EOF";

#[test]
fn explicit_pdf_enters_pending_after_validation() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("article.PDF");
    fs::write(&path, COMPLETE_PDF).unwrap();

    let pending = prepare_explicit_pdf(&path).unwrap();

    assert_ne!(pending.import_id().as_uuid(), Uuid::nil());
    assert_eq!(pending.source(), PdfImportSource::Explicit);
    assert_eq!(pending.path(), fs::canonicalize(path).unwrap());
    assert_eq!(pending.byte_length(), COMPLETE_PDF.len() as u64);
}

#[test]
fn explicit_import_rejects_non_pdf_and_symlink() {
    let directory = tempfile::tempdir().unwrap();
    let text_path = directory.path().join("article.txt");
    let invalid_pdf_path = directory.path().join("invalid.pdf");
    let directory_path = directory.path().join("folder.pdf");
    fs::write(&text_path, COMPLETE_PDF).unwrap();
    fs::write(&invalid_pdf_path, b"not a PDF").unwrap();
    fs::create_dir(&directory_path).unwrap();

    assert_eq!(
        prepare_explicit_pdf(&text_path),
        Err(PdfImportError::UnsupportedFileType)
    );
    assert_eq!(
        prepare_explicit_pdf(&invalid_pdf_path),
        Err(PdfImportError::InvalidPdf)
    );
    assert_eq!(
        prepare_explicit_pdf(&directory_path),
        Err(PdfImportError::NotRegularFile)
    );
    assert_eq!(
        prepare_explicit_pdf(directory.path().join("missing.pdf")),
        Err(PdfImportError::FileUnavailable)
    );

    #[cfg(unix)]
    {
        let link_path = directory.path().join("linked.pdf");
        std::os::unix::fs::symlink(&invalid_pdf_path, &link_path).unwrap();
        assert_eq!(
            prepare_explicit_pdf(link_path),
            Err(PdfImportError::SymbolicLinkNotAllowed)
        );
    }
}

#[test]
fn watched_pdf_waits_during_chunked_write() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("article.pdf");
    let start = Instant::now();
    let second_change = start + Duration::from_millis(500);
    let mut intake = WatchedPdfIntake::new(directory.path()).unwrap();

    fs::write(&path, b"%P").unwrap();
    intake.record_change(&path, start).unwrap();
    assert_eq!(
        intake.confirm_stable(&path, start + Duration::from_millis(500)),
        Ok(WatchedPdfIntakeStatus::Waiting)
    );

    append(&path, b"DF-1.7\nbody\n%%EOF");
    intake.record_change(&path, second_change).unwrap();
    assert_eq!(
        intake.confirm_stable(&path, second_change + Duration::from_millis(999)),
        Ok(WatchedPdfIntakeStatus::Waiting)
    );

    let pending = intake
        .confirm_stable(&path, second_change + STABLE_WRITE_DEBOUNCE)
        .unwrap();
    let WatchedPdfIntakeStatus::Pending(pending) = pending else {
        panic!("stable watched PDF should become pending");
    };
    assert_eq!(pending.source(), PdfImportSource::WatchedFolder);
    assert_eq!(pending.byte_length(), COMPLETE_PDF.len() as u64);
}

#[test]
fn watched_pdf_requires_debounce_and_stable_snapshot() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("article.pdf");
    let start = Instant::now();
    let mut intake = WatchedPdfIntake::new(directory.path()).unwrap();
    fs::write(&path, b"%PDF-1.7\n").unwrap();
    intake.record_change(&path, start).unwrap();

    append(&path, b"body\n%%EOF");
    assert_eq!(
        intake.confirm_stable(&path, start + STABLE_WRITE_DEBOUNCE),
        Ok(WatchedPdfIntakeStatus::Waiting)
    );
    assert!(matches!(
        intake.confirm_stable(&path, start + STABLE_WRITE_DEBOUNCE * 2),
        Ok(WatchedPdfIntakeStatus::Pending(_))
    ));
}

#[test]
fn watched_pdf_rejects_paths_outside_root() {
    let watched_directory = tempfile::tempdir().unwrap();
    let outside_directory = tempfile::tempdir().unwrap();
    let outside_path = outside_directory.path().join("outside.pdf");
    fs::write(&outside_path, COMPLETE_PDF).unwrap();
    let mut intake = WatchedPdfIntake::new(watched_directory.path()).unwrap();

    assert_eq!(
        intake.record_change(&outside_path, Instant::now()),
        Err(PdfImportError::OutsideWatchedFolder)
    );

    #[cfg(unix)]
    {
        let link_path = watched_directory.path().join("linked.pdf");
        std::os::unix::fs::symlink(&outside_path, &link_path).unwrap();
        assert_eq!(
            intake.record_change(link_path, Instant::now()),
            Err(PdfImportError::SymbolicLinkNotAllowed)
        );

        let linked_directory = watched_directory.path().join("linked-directory");
        std::os::unix::fs::symlink(outside_directory.path(), &linked_directory).unwrap();
        assert_eq!(
            intake.record_change(linked_directory.join("outside.pdf"), Instant::now()),
            Err(PdfImportError::OutsideWatchedFolder)
        );
    }
}

#[test]
fn watched_pdf_returns_typed_file_failures() {
    let missing_root = tempfile::tempdir().unwrap().path().join("missing");
    assert_eq!(
        WatchedPdfIntake::new(missing_root).err(),
        Some(PdfImportError::InvalidWatchedFolder)
    );

    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("article.pdf");
    fs::write(&path, COMPLETE_PDF).unwrap();
    let start = Instant::now();
    let mut intake = WatchedPdfIntake::new(directory.path()).unwrap();
    assert_eq!(
        intake.confirm_stable(&path, start),
        Err(PdfImportError::FileNotObserved)
    );

    intake.record_change(&path, start).unwrap();
    let earlier = start.checked_sub(Duration::from_millis(1)).unwrap();
    assert_eq!(
        intake.confirm_stable(&path, earlier),
        Err(PdfImportError::ObservationOutOfOrder)
    );
    assert_eq!(
        PdfImportError::ObservationOutOfOrder.to_string(),
        "PDF observation time is out of order"
    );
}

fn append(path: &Path, bytes: &[u8]) {
    let mut file = OpenOptions::new().append(true).open(path).unwrap();
    file.write_all(bytes).unwrap();
    file.sync_all().unwrap();
}
