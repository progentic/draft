#[cfg(target_os = "macos")]
use std::process::Command;
use std::{
    cell::Cell,
    fs,
    io::{Cursor, Read, Write},
};

use serde_json::json;
use sha2::{Digest, Sha256};
use zip::{CompressionMethod, ZipArchive, ZipWriter, write::SimpleFileOptions};

use super::*;
use crate::{
    documents::test_support::TestDocumentPath,
    exports::docx::compile_docx,
    interoperability::{
        fidelity::{ExternalFeature, ExternalFidelity},
        import_docx_source,
        provenance::ExternalSourceProvenance,
    },
};

const DOCUMENT_ID: &str = "00000000-0000-4000-8000-000000000001";
const ORIGINAL_SOURCE: &[u8] = b"original external source";

#[test]
fn eligibility_inspection_never_writes_or_mutates_registry() {
    let source = docx_source("external-inspection");
    let original = envelope("Original", "Original text");
    let edited = envelope("Edited", "Replacement text");
    let registry = registered_external(&source, &original, ExternalFidelity::Exact);
    let writes = Cell::new(0);

    let outcome = save_external_document_with_writer(
        &registry,
        snapshot(&edited),
        ExternalSaveDecision::Inspect,
        |_, _| {
            writes.set(writes.get() + 1);
            Ok(())
        },
    )
    .unwrap();

    assert_eq!(
        outcome,
        SaveExternalDocumentOutcome::Eligibility {
            document_id: edited.document_id(),
            display_name: source_file_name(&source),
            disposition: SameFormatSaveDisposition::AllowedExact,
            normalizations: vec![],
        }
    );
    assert_eq!(writes.get(), 0);
    assert_source(&source, ORIGINAL_SOURCE);
    assert_eq!(registry.close(original.document_id()), Ok(original));
}

#[test]
fn eligibility_rejects_unrepresentable_content_without_writing() {
    let source = docx_source("external-inspection-compilation");
    let original = envelope("Original", "Original text");
    let invalid = envelope_with_citation();
    let registry = registered_external(&source, &original, ExternalFidelity::Exact);

    let error =
        save_external_document(&registry, snapshot(&invalid), ExternalSaveDecision::Inspect)
            .unwrap_err();

    assert!(matches!(
        error,
        SaveExternalDocumentError::Compilation {
            cause: DocxExportError::UnsupportedCitation { .. }
        }
    ));
    assert_source(&source, ORIGINAL_SOURCE);
    assert_eq!(registry.close(original.document_id()), Ok(original));
}

#[test]
fn unchanged_exact_source_performs_no_write() {
    let source = docx_source("external-unchanged");
    let original = envelope("Original", "Original text");
    let registry = registered_external(&source, &original, ExternalFidelity::Exact);
    let writes = Cell::new(0);

    let outcome = save_external_document_with_writer(
        &registry,
        snapshot(&original),
        ExternalSaveDecision::SaveExact,
        |_, _| {
            writes.set(writes.get() + 1);
            Ok(())
        },
    )
    .unwrap();

    assert_eq!(
        outcome,
        SaveExternalDocumentOutcome::Unchanged {
            document_id: original.document_id(),
            display_name: source_file_name(&source),
        }
    );
    assert_eq!(writes.get(), 0);
    assert_source(&source, ORIGINAL_SOURCE);
}

#[test]
fn exact_save_replaces_source_and_refreshes_provenance() {
    let source = docx_source("external-exact-save");
    let original = envelope("Original", "Original text");
    let edited = envelope("Edited", "Replacement text");
    let registry = registered_external(&source, &original, ExternalFidelity::Exact);

    let outcome = save_external_document(
        &registry,
        snapshot(&edited),
        ExternalSaveDecision::SaveExact,
    )
    .unwrap();
    let source_bytes = fs::read(source.path()).unwrap();
    let provenance = registry.external_source(edited.document_id()).unwrap();
    let reopened = import_docx_source(source.path()).unwrap();

    assert!(matches!(
        outcome,
        SaveExternalDocumentOutcome::Saved {
            disposition: SameFormatSaveDisposition::AllowedExact,
            ..
        }
    ));
    assert_eq!(
        provenance.save_disposition(&edited, CurrentSource::Bytes(&source_bytes)),
        SameFormatSaveDisposition::NoChanges
    );
    assert_eq!(reopened.envelope.document(), edited.document());
}

#[cfg(target_os = "macos")]
#[test]
fn exact_and_normalized_replacements_open_in_macos_text_reader() {
    for (source, decision) in [
        (
            exported_docx_source("external-reader-exact", "Original text"),
            ExternalSaveDecision::SaveExact,
        ),
        (
            normalized_docx_source("external-reader-normalized"),
            ExternalSaveDecision::AcceptNormalization,
        ),
    ] {
        let imported = import_docx_source(source.path()).unwrap();
        let edited = edited_import(&imported.envelope, "Replacement text");
        let registry = registry_with_provenance(&imported.envelope, imported.provenance);

        save_external_document(&registry, snapshot(&edited), decision).unwrap();
        let replacement = fs::read(source.path()).unwrap();
        let output = Command::new("/usr/bin/textutil")
            .args(["-convert", "txt", "-stdout"])
            .arg(source.path())
            .output()
            .expect("macOS text reader should launch");

        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("Replacement text"));
        eprintln!("replacement_sha256={:x}", Sha256::digest(&replacement));
    }
}

#[cfg(target_os = "macos")]
#[test]
#[ignore = "requires DRAFT_COMPATIBLE_READER to name a headless office executable"]
fn exact_and_normalized_replacements_render_in_configured_compatible_reader() {
    let reader = std::env::var("DRAFT_COMPATIBLE_READER")
        .expect("DRAFT_COMPATIBLE_READER must name the compatible reader");
    for (source, decision) in compatible_reader_sources() {
        let imported = import_docx_source(source.path()).unwrap();
        let edited = edited_import(&imported.envelope, "Compatible reader evidence");
        let registry = registry_with_provenance(&imported.envelope, imported.provenance);
        let source_hash = Sha256::digest(fs::read(source.path()).unwrap());

        save_external_document(&registry, snapshot(&edited), decision).unwrap();
        let replacement = fs::read(source.path()).unwrap();
        let profile = format!(
            "-env:UserInstallation=file://{}",
            source
                .path()
                .parent()
                .unwrap()
                .join("reader-profile")
                .display()
        );
        let output = Command::new(&reader)
            .arg(profile)
            .args(["--headless", "--convert-to", "pdf", "--outdir"])
            .arg(source.path().parent().unwrap())
            .arg(source.path())
            .output()
            .expect("configured compatible reader should launch");
        let rendered = source.path().with_extension("pdf");

        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_ne!(
            source_hash.as_slice(),
            Sha256::digest(&replacement).as_slice()
        );
        assert!(fs::metadata(&rendered).unwrap().len() > 0);
        eprintln!(
            "reader={reader} source_sha256={source_hash:x} replacement_sha256={:x}",
            Sha256::digest(&replacement)
        );
    }
}

#[cfg(target_os = "macos")]
fn compatible_reader_sources() -> [(TestDocumentPath, ExternalSaveDecision); 2] {
    [
        (
            exported_docx_source("compatible-reader-exact", "Original text"),
            ExternalSaveDecision::SaveExact,
        ),
        (
            normalized_docx_source("compatible-reader-normalized"),
            ExternalSaveDecision::AcceptNormalization,
        ),
    ]
}

#[test]
fn normalization_requires_acceptance_and_cancel_never_writes() {
    let source = docx_source("external-normalized-save");
    let original = envelope("Original", "Original text");
    let edited = envelope("Edited", "Replacement text");
    let fidelity = ExternalFidelity::CanonicallyNormalized {
        features: vec![ExternalFeature::AlternateHeadingStyleName],
    };
    let registry = registered_external(&source, &original, fidelity);

    let confirmation = save_external_document(
        &registry,
        snapshot(&edited),
        ExternalSaveDecision::SaveExact,
    )
    .unwrap();
    let cancelled =
        save_external_document(&registry, snapshot(&edited), ExternalSaveDecision::Cancel).unwrap();

    assert!(matches!(
        confirmation,
        SaveExternalDocumentOutcome::ConfirmationRequired {
            disposition: SameFormatSaveDisposition::AllowedAfterAcceptedNormalization,
            ..
        }
    ));
    assert_eq!(
        cancelled,
        SaveExternalDocumentOutcome::Cancelled {
            document_id: edited.document_id()
        }
    );
    assert_source(&source, ORIGINAL_SOURCE);

    assert!(matches!(
        save_external_document(
            &registry,
            snapshot(&edited),
            ExternalSaveDecision::AcceptNormalization,
        )
        .unwrap(),
        SaveExternalDocumentOutcome::Saved {
            disposition: SameFormatSaveDisposition::AllowedAfterAcceptedNormalization,
            ..
        }
    ));
}

#[test]
fn unsupported_source_behavior_denies_overwrite() {
    let source = docx_source("external-unsupported-save");
    let original = envelope("Original", "Original text");
    let edited = envelope("Edited", "Replacement text");
    let fidelity = ExternalFidelity::UnsupportedPreservable {
        features: vec![ExternalFeature::ParagraphBorder],
    };
    let registry = registered_external(&source, &original, fidelity);

    let outcome = save_external_document(
        &registry,
        snapshot(&edited),
        ExternalSaveDecision::SaveExact,
    )
    .unwrap();

    assert_eq!(
        outcome,
        denied(
            &edited,
            SameFormatSaveDisposition::DeniedUnsupportedSourceBehavior,
        )
    );
    assert_source(&source, ORIGINAL_SOURCE);
}

#[test]
fn missing_and_changed_sources_are_denied_before_write() {
    let original = envelope("Original", "Original text");
    let edited = envelope("Edited", "Replacement text");
    let missing = TestDocumentPath::with_extension("external-missing", "docx");
    let missing_registry = registry_with_provenance(
        &original,
        provenance(&missing, &original, ExternalFidelity::Exact),
    );
    let changed = docx_source("external-changed");
    let changed_registry = registered_external(&changed, &original, ExternalFidelity::Exact);
    changed.write(b"changed outside DRAFT");

    assert_eq!(
        save_external_document(
            &missing_registry,
            snapshot(&edited),
            ExternalSaveDecision::SaveExact,
        )
        .unwrap(),
        denied(&edited, SameFormatSaveDisposition::DeniedSourceMissing)
    );
    assert_eq!(
        save_external_document(
            &changed_registry,
            snapshot(&edited),
            ExternalSaveDecision::SaveExact,
        )
        .unwrap(),
        denied(&edited, SameFormatSaveDisposition::DeniedSourceChanged)
    );
    assert_source(&changed, b"changed outside DRAFT");
}

#[test]
fn source_change_after_compilation_is_denied_before_replacement() {
    let source = docx_source("external-late-change");
    let original = envelope("Original", "Original text");
    let edited = envelope("Edited", "Replacement text");
    let registry = registered_external(&source, &original, ExternalFidelity::Exact);
    let provenance = registry.external_source(edited.document_id()).unwrap();
    let plan = plan_save(edited.clone(), provenance, ExternalSaveDecision::SaveExact).unwrap();
    source.write(b"changed after preparation");

    let outcome = execute_plan(&registry, plan, write_document_atomically).unwrap();

    assert_eq!(
        outcome,
        denied(&edited, SameFormatSaveDisposition::DeniedSourceChanged)
    );
    assert_source(&source, b"changed after preparation");
}

#[test]
fn imported_source_changed_after_confirmation_is_denied_without_mutation() {
    let source = exported_docx_source("external-imported-stale", "Original text");
    let imported = import_docx_source(source.path()).unwrap();
    let original = imported.envelope.clone();
    let edited = edited_import(&original, "Edited text");
    let registry = registry_with_provenance(&original, imported.provenance);

    assert!(matches!(
        save_external_document(&registry, snapshot(&edited), ExternalSaveDecision::Inspect)
            .unwrap(),
        SaveExternalDocumentOutcome::Eligibility {
            disposition: SameFormatSaveDisposition::AllowedExact,
            ..
        }
    ));
    let external_change = compiled_docx("External change");
    source.write(&external_change);

    let outcome = save_external_document(
        &registry,
        snapshot(&edited),
        ExternalSaveDecision::SaveExact,
    )
    .unwrap();

    assert_eq!(
        outcome,
        denied(&edited, SameFormatSaveDisposition::DeniedSourceChanged)
    );
    assert_source(&source, &external_change);
    assert_eq!(registry.close(original.document_id()), Ok(original));
}

#[test]
fn imported_normalization_names_the_required_transformation() {
    let source = normalized_docx_source("external-imported-normalized");
    let imported = import_docx_source(source.path()).unwrap();
    let edited = edited_import(&imported.envelope, "Edited heading");
    let registry = registry_with_provenance(&imported.envelope, imported.provenance);

    let outcome =
        save_external_document(&registry, snapshot(&edited), ExternalSaveDecision::Inspect)
            .unwrap();

    assert!(matches!(
        outcome,
        SaveExternalDocumentOutcome::Eligibility {
            disposition: SameFormatSaveDisposition::AllowedAfterAcceptedNormalization,
            normalizations,
            ..
        } if normalizations == vec![ExternalFeature::AlternateHeadingStyleName]
    ));
}

#[test]
fn stale_normalized_source_returns_denial_without_obsolete_normalizations() {
    let source = normalized_docx_source("external-normalized-stale");
    let imported = import_docx_source(source.path()).unwrap();
    let edited = edited_import(&imported.envelope, "Edited heading");
    let registry = registry_with_provenance(&imported.envelope, imported.provenance);
    source.write(&compiled_docx("External change"));

    let outcome =
        save_external_document(&registry, snapshot(&edited), ExternalSaveDecision::Inspect)
            .unwrap();

    assert!(matches!(
        outcome,
        SaveExternalDocumentOutcome::Eligibility {
            disposition: SameFormatSaveDisposition::DeniedSourceChanged,
            normalizations,
            ..
        } if normalizations.is_empty()
    ));
}

#[test]
fn compilation_failure_preserves_source_and_registry() {
    let source = docx_source("external-compile-failure");
    let original = envelope("Original", "Original text");
    let invalid = envelope_with_citation();
    let registry = registered_external(&source, &original, ExternalFidelity::Exact);

    let error = save_external_document(
        &registry,
        snapshot(&invalid),
        ExternalSaveDecision::SaveExact,
    )
    .unwrap_err();

    assert!(matches!(
        error,
        SaveExternalDocumentError::Compilation {
            cause: DocxExportError::UnsupportedCitation { .. }
        }
    ));
    assert_source(&source, ORIGINAL_SOURCE);
    assert_eq!(registry.close(original.document_id()), Ok(original));
}

#[test]
fn pre_replacement_failure_preserves_source_and_registry() {
    let source = docx_source("external-write-failure");
    let original = envelope("Original", "Original text");
    let edited = envelope("Edited", "Replacement text");
    let registry = registered_external(&source, &original, ExternalFidelity::Exact);

    let error = save_external_document_with_writer(
        &registry,
        snapshot(&edited),
        ExternalSaveDecision::SaveExact,
        |_, _| Err(AtomicDocumentWriteError::WriteTemporaryFile),
    )
    .unwrap_err();

    assert_eq!(
        error,
        SaveExternalDocumentError::WriteFailed {
            cause: AtomicDocumentWriteError::WriteTemporaryFile
        }
    );
    assert_source(&source, ORIGINAL_SOURCE);
    assert_eq!(registry.close(original.document_id()), Ok(original));
}

#[test]
fn durability_failure_rolls_back_replacement() {
    let source = docx_source("external-durability-rollback");
    let original = envelope("Original", "Original text");
    let edited = envelope("Edited", "Replacement text");
    let registry = registered_external(&source, &original, ExternalFidelity::Exact);
    let calls = Cell::new(0);

    let error = save_external_document_with_writer(
        &registry,
        snapshot(&edited),
        ExternalSaveDecision::SaveExact,
        |_, bytes| {
            source.write(bytes);
            calls.set(calls.get() + 1);
            if calls.get() == 1 {
                Err(AtomicDocumentWriteError::SyncParentDirectory)
            } else {
                Ok(())
            }
        },
    )
    .unwrap_err();

    assert_eq!(
        error,
        SaveExternalDocumentError::ReplacementRolledBack {
            cause: ExternalSaveCommitFailure::DurabilityUncertain
        }
    );
    assert_eq!(calls.get(), 2);
    assert_source(&source, ORIGINAL_SOURCE);
    assert_eq!(registry.close(original.document_id()), Ok(original));
}

#[test]
fn rollback_failure_reports_uncertain_source_state() {
    let source = docx_source("external-rollback-failure");
    let original = envelope("Original", "Original text");
    let edited = envelope("Edited", "Replacement text");
    let registry = registered_external(&source, &original, ExternalFidelity::Exact);
    let calls = Cell::new(0);

    let error = save_external_document_with_writer(
        &registry,
        snapshot(&edited),
        ExternalSaveDecision::SaveExact,
        |_, bytes| {
            calls.set(calls.get() + 1);
            if calls.get() == 1 {
                source.write(bytes);
                Err(AtomicDocumentWriteError::SyncParentDirectory)
            } else {
                Err(AtomicDocumentWriteError::ReplaceTarget)
            }
        },
    )
    .unwrap_err();

    assert_eq!(
        error,
        SaveExternalDocumentError::RollbackFailed {
            cause: ExternalSaveCommitFailure::DurabilityUncertain,
            rollback: AtomicDocumentWriteError::ReplaceTarget,
        }
    );
    assert_ne!(fs::read(source.path()).unwrap(), ORIGINAL_SOURCE);
    assert_eq!(registry.close(original.document_id()), Ok(original));
}

#[test]
fn registry_commit_failure_rolls_back_source() {
    let source = docx_source("external-registry-rollback");
    let original = envelope("Original", "Original text");
    let edited = envelope("Edited", "Replacement text");
    let registry = registered_external(&source, &original, ExternalFidelity::Exact);
    let provenance = registry.external_source(edited.document_id()).unwrap();
    let plan = plan_save(edited, provenance, ExternalSaveDecision::SaveExact).unwrap();
    let ExternalSavePlan::Write(prepared) = plan else {
        panic!("edited exact source should prepare a write");
    };
    let calls = Cell::new(0);

    let error = apply_prepared_write(
        *prepared,
        |_, bytes| {
            source.write(bytes);
            calls.set(calls.get() + 1);
            Ok(())
        },
        |_, _| Err(DocumentRegistryError::RegistryUnavailable),
    )
    .unwrap_err();

    assert_eq!(
        error,
        SaveExternalDocumentError::ReplacementRolledBack {
            cause: ExternalSaveCommitFailure::Registry {
                cause: DocumentRegistryError::RegistryUnavailable
            }
        }
    );
    assert_eq!(calls.get(), 2);
    assert_source(&source, ORIGINAL_SOURCE);
    assert_eq!(registry.close(original.document_id()), Ok(original));
}

#[test]
fn native_document_has_no_external_overwrite_authority() {
    let registry = DocumentRegistry::new();
    let document = envelope("Native", "Native text");
    registry.open(document.clone()).unwrap();

    assert_eq!(
        save_external_document(
            &registry,
            snapshot(&document),
            ExternalSaveDecision::SaveExact,
        )
        .unwrap(),
        denied(
            &document,
            SameFormatSaveDisposition::DeniedMissingProvenance,
        )
    );
}

fn registered_external(
    source: &TestDocumentPath,
    envelope: &DocumentEnvelope,
    fidelity: ExternalFidelity,
) -> DocumentRegistry {
    registry_with_provenance(envelope, provenance(source, envelope, fidelity))
}

fn registry_with_provenance(
    envelope: &DocumentEnvelope,
    provenance: ExternalSourceProvenance,
) -> DocumentRegistry {
    let registry = DocumentRegistry::new();
    registry
        .open_imported_external(envelope.clone(), provenance)
        .unwrap();
    registry
}

fn provenance(
    source: &TestDocumentPath,
    envelope: &DocumentEnvelope,
    fidelity: ExternalFidelity,
) -> ExternalSourceProvenance {
    ExternalSourceProvenance::imported_docx(
        source.path().to_owned(),
        source_file_name(source),
        ORIGINAL_SOURCE,
        envelope,
        fidelity,
    )
}

fn docx_source(label: &str) -> TestDocumentPath {
    let source = TestDocumentPath::with_extension(label, "docx");
    source.write(ORIGINAL_SOURCE);
    source
}

fn exported_docx_source(label: &str, text: &str) -> TestDocumentPath {
    let source = TestDocumentPath::with_extension(label, "docx");
    source.write(&compiled_docx(text));
    source
}

fn normalized_docx_source(label: &str) -> TestDocumentPath {
    let source = TestDocumentPath::with_extension(label, "docx");
    source.write(&normalized_docx_bytes());
    source
}

fn compiled_docx(text: &str) -> Vec<u8> {
    compile_docx(&envelope("Reader evidence", text))
        .unwrap()
        .as_bytes()
        .to_vec()
}

fn normalized_docx_bytes() -> Vec<u8> {
    let artifact = compile_docx(&heading_envelope()).unwrap();
    rewrite_document_xml(artifact.as_bytes(), |xml| {
        xml.replace("w:val=\"Heading2\"", "w:val=\"Heading 2\"")
    })
}

fn rewrite_document_xml(bytes: &[u8], rewrite: impl FnOnce(String) -> String) -> Vec<u8> {
    let parts = read_package_parts(bytes);
    let mut rewrite = Some(rewrite);
    let updated = parts.into_iter().map(|(name, contents)| {
        if name == "word/document.xml" {
            let xml = String::from_utf8(contents).unwrap();
            (name, rewrite.take().unwrap()(xml).into_bytes())
        } else {
            (name, contents)
        }
    });
    write_package_parts(updated)
}

fn read_package_parts(bytes: &[u8]) -> Vec<(String, Vec<u8>)> {
    let mut archive = ZipArchive::new(Cursor::new(bytes)).unwrap();
    (0..archive.len())
        .map(|index| {
            let mut file = archive.by_index(index).unwrap();
            let name = file.name().to_owned();
            let mut contents = Vec::new();
            file.read_to_end(&mut contents).unwrap();
            (name, contents)
        })
        .collect()
}

fn write_package_parts(parts: impl Iterator<Item = (String, Vec<u8>)>) -> Vec<u8> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::DEFAULT
        .compression_method(CompressionMethod::Stored)
        .unix_permissions(0o644);
    for (name, contents) in parts {
        writer.start_file(name, options).unwrap();
        writer.write_all(&contents).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

fn edited_import(source: &DocumentEnvelope, text: &str) -> DocumentEnvelope {
    let mut value = serde_json::to_value(source).unwrap();
    value["title"] = json!("Edited");
    value["document"] = json!({
        "type": "doc",
        "content": [{
            "type": "paragraph",
            "content": [{ "type": "text", "text": text }]
        }]
    });
    DocumentEnvelope::from_json_value(value).unwrap()
}

fn source_file_name(source: &TestDocumentPath) -> String {
    source
        .path()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned()
}

fn envelope(title: &str, text: &str) -> DocumentEnvelope {
    DocumentEnvelope::from_json_value(json!({
        "schema_version": crate::documents::envelope::DOCUMENT_ENVELOPE_SCHEMA_VERSION,
        "document_id": DOCUMENT_ID,
        "title": title,
        "document": {
            "type": "doc",
            "content": [{
                "type": "paragraph",
                "content": [{ "type": "text", "text": text }]
            }]
        }
    }))
    .unwrap()
}

fn envelope_with_citation() -> DocumentEnvelope {
    DocumentEnvelope::from_json_value(json!({
        "schema_version": crate::documents::envelope::DOCUMENT_ENVELOPE_SCHEMA_VERSION,
        "document_id": DOCUMENT_ID,
        "title": "Citation",
        "document": {
            "type": "doc",
            "content": [{
                "type": "paragraph",
                "content": [{
                    "type": "citation",
                    "attrs": {
                        "schema_version": 1,
                        "citekey": "smith2026",
                        "render_style": "apa7"
                    }
                }]
            }]
        }
    }))
    .unwrap()
}

fn heading_envelope() -> DocumentEnvelope {
    DocumentEnvelope::from_json_value(json!({
        "schema_version": crate::documents::envelope::DOCUMENT_ENVELOPE_SCHEMA_VERSION,
        "document_id": DOCUMENT_ID,
        "title": "Normalized heading",
        "document": {
            "type": "doc",
            "content": [{
                "type": "heading",
                "attrs": { "level": 2 },
                "content": [{ "type": "text", "text": "Heading" }]
            }]
        }
    }))
    .unwrap()
}

fn snapshot(envelope: &DocumentEnvelope) -> Value {
    serde_json::to_value(envelope).unwrap()
}

fn denied(
    envelope: &DocumentEnvelope,
    disposition: SameFormatSaveDisposition,
) -> SaveExternalDocumentOutcome {
    SaveExternalDocumentOutcome::Denied {
        document_id: envelope.document_id(),
        disposition,
    }
}

fn assert_source(source: &TestDocumentPath, expected: &[u8]) {
    assert_eq!(fs::read(source.path()).unwrap(), expected);
}
