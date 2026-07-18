use std::path::{Path, PathBuf};

use serde::Serialize;

const OUTPUT_EXTENSIONS: [&str; 3] = ["draft", "docx", "txt"];

/// Closed output formats available through the Save As workflow.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SaveAsFormat {
    Draft,
    Docx,
    Txt,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub(crate) enum SaveAsTargetError {
    InvalidFileName,
    ConflictingExtension,
    ExtensionMismatch,
    TargetIsDirectory,
}

impl SaveAsFormat {
    pub(crate) fn from_code(code: &str) -> Option<Self> {
        match code {
            "draft" => Some(Self::Draft),
            "docx" => Some(Self::Docx),
            "txt" => Some(Self::Txt),
            _ => None,
        }
    }

    pub(crate) fn extension(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Docx => "docx",
            Self::Txt => "txt",
        }
    }

    pub(crate) fn dialog_title(self) -> &'static str {
        match self {
            Self::Draft => "Save DRAFT document",
            Self::Docx => "Save Word document copy",
            Self::Txt => "Save plain-text copy",
        }
    }

    pub(crate) fn filter_name(self) -> &'static str {
        match self {
            Self::Draft => "DRAFT document",
            Self::Docx => "Word document",
            Self::Txt => "Plain text",
        }
    }
}

pub(crate) fn normalize_save_as_target(
    mut path: PathBuf,
    format: SaveAsFormat,
) -> Result<PathBuf, SaveAsTargetError> {
    validate_target_file_name(&path)?;
    if path.is_dir() {
        return Err(SaveAsTargetError::TargetIsDirectory);
    }
    match target_extension(&path) {
        None => path.set_extension(format.extension()),
        Some(extension) if extension.eq_ignore_ascii_case(format.extension()) => {
            reject_conflicting_extension(&path)?;
            path.set_extension(format.extension())
        }
        Some(_) => return Err(SaveAsTargetError::ExtensionMismatch),
    };
    Ok(path)
}

fn validate_target_file_name(path: &Path) -> Result<(), SaveAsTargetError> {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .ok_or(SaveAsTargetError::InvalidFileName)?;
    let stem = path
        .file_stem()
        .and_then(|name| name.to_str())
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .ok_or(SaveAsTargetError::InvalidFileName)?;
    if invalid_file_name(file_name, stem) {
        return Err(SaveAsTargetError::InvalidFileName);
    }
    Ok(())
}

fn invalid_file_name(file_name: &str, stem: &str) -> bool {
    file_name.starts_with('.')
        || file_name.ends_with(['.', ' '])
        || file_name.chars().any(invalid_file_name_character)
        || is_reserved_file_stem(stem)
}

fn invalid_file_name_character(character: char) -> bool {
    character.is_control() || matches!(character, '<' | '>' | ':' | '"' | '\\' | '|' | '?' | '*')
}

fn is_reserved_file_stem(stem: &str) -> bool {
    let stem = stem.to_ascii_uppercase();
    matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || reserved_numbered_stem(&stem, "COM")
        || reserved_numbered_stem(&stem, "LPT")
}

fn reserved_numbered_stem(stem: &str, prefix: &str) -> bool {
    stem.strip_prefix(prefix)
        .is_some_and(|suffix| matches!(suffix, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"))
}

fn target_extension(path: &Path) -> Option<&str> {
    path.extension().and_then(|extension| extension.to_str())
}

fn reject_conflicting_extension(path: &Path) -> Result<(), SaveAsTargetError> {
    let conflicting = path
        .file_stem()
        .and_then(|stem| Path::new(stem).extension())
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            OUTPUT_EXTENSIONS
                .iter()
                .any(|candidate| extension.eq_ignore_ascii_case(candidate))
        });
    if conflicting {
        Err(SaveAsTargetError::ConflictingExtension)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::documents::test_support::TestDocumentPath;

    #[test]
    fn format_codes_and_extensions_are_closed() {
        assert_eq!(SaveAsFormat::from_code("draft"), Some(SaveAsFormat::Draft));
        assert_eq!(SaveAsFormat::from_code("docx"), Some(SaveAsFormat::Docx));
        assert_eq!(SaveAsFormat::from_code("txt"), Some(SaveAsFormat::Txt));
        for rejected in ["pdf", "md", "DOCX", "", "unknown"] {
            assert_eq!(SaveAsFormat::from_code(rejected), None);
        }
    }

    #[test]
    fn missing_and_uppercase_extensions_normalize() {
        assert_eq!(
            normalize_save_as_target(PathBuf::from("Paper"), SaveAsFormat::Docx),
            Ok(PathBuf::from("Paper.docx"))
        );
        assert_eq!(
            normalize_save_as_target(PathBuf::from("Paper.DOCX"), SaveAsFormat::Docx),
            Ok(PathBuf::from("Paper.docx"))
        );
    }

    #[test]
    fn mismatched_and_conflicting_extensions_fail() {
        assert_eq!(
            normalize_save_as_target(PathBuf::from("Paper.draft"), SaveAsFormat::Docx),
            Err(SaveAsTargetError::ExtensionMismatch)
        );
        assert_eq!(
            normalize_save_as_target(PathBuf::from("Paper.draft.docx"), SaveAsFormat::Docx),
            Err(SaveAsTargetError::ConflictingExtension)
        );
    }

    #[test]
    fn invalid_and_reserved_names_fail() {
        for name in ["", " ", "report?.txt", "CON.txt", "LPT9.draft", ".docx"] {
            assert_eq!(
                normalize_save_as_target(PathBuf::from(name), SaveAsFormat::Txt),
                Err(SaveAsTargetError::InvalidFileName)
            );
        }
    }

    #[test]
    fn directory_targets_fail_before_output_policy() {
        let target = TestDocumentPath::with_extension("save-as-directory", "docx");
        fs::create_dir(target.path()).unwrap();

        assert_eq!(
            normalize_save_as_target(target.path().to_owned(), SaveAsFormat::Docx),
            Err(SaveAsTargetError::TargetIsDirectory)
        );
    }
}
