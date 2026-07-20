/// Rust-owned application status before it crosses the Tauri command boundary.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct RuntimeStatus {
    build_commit: String,
    build_profile: String,
    version: String,
}

/// Failure to construct a valid Rust-owned runtime status.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum RuntimeStatusError {
    InvalidBuildCommit,
    InvalidBuildProfile,
    MissingVersion,
}

/// Builds status from the package metadata compiled into the trusted runtime.
pub(crate) fn current_runtime_status() -> Result<RuntimeStatus, RuntimeStatusError> {
    runtime_status_from_metadata(
        env!("CARGO_PKG_VERSION"),
        env!("DRAFT_BUILD_COMMIT"),
        env!("DRAFT_BUILD_PROFILE"),
    )
}

impl RuntimeStatus {
    /// Returns the validated product version without exposing build internals.
    pub(crate) fn version(&self) -> &str {
        &self.version
    }

    /// Moves validated runtime metadata into the typed command response.
    pub(crate) fn into_parts(self) -> (String, String, String) {
        (self.version, self.build_commit, self.build_profile)
    }
}

fn runtime_status_from_metadata(
    version: &str,
    build_commit: &str,
    build_profile: &str,
) -> Result<RuntimeStatus, RuntimeStatusError> {
    Ok(RuntimeStatus {
        build_commit: validated_build_commit(build_commit)?,
        build_profile: validated_build_profile(build_profile)?,
        version: validated_version(version)?,
    })
}

fn validated_build_commit(commit: &str) -> Result<String, RuntimeStatusError> {
    if commit == "development" || is_full_git_commit(commit) {
        return Ok(commit.to_owned());
    }
    Err(RuntimeStatusError::InvalidBuildCommit)
}

fn is_full_git_commit(commit: &str) -> bool {
    commit.len() == 40
        && commit
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn validated_build_profile(profile: &str) -> Result<String, RuntimeStatusError> {
    match profile {
        "debug" | "release" => Ok(profile.to_owned()),
        _ => Err(RuntimeStatusError::InvalidBuildProfile),
    }
}

fn validated_version(version: &str) -> Result<String, RuntimeStatusError> {
    let normalized_version = version.trim();
    if normalized_version.is_empty() {
        return Err(RuntimeStatusError::MissingVersion);
    }

    Ok(normalized_version.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_status_contains_manifest_version() {
        let status = current_runtime_status().expect("manifest version should be valid");

        assert_eq!(status.version(), env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn blank_version_is_rejected() {
        let result = runtime_status_from_metadata("  ", "development", "debug");

        assert_eq!(result, Err(RuntimeStatusError::MissingVersion));
    }

    #[test]
    fn surrounding_version_whitespace_is_removed() {
        let status = runtime_status_from_metadata(" 0.1.0 ", "development", "debug")
            .expect("metadata should be valid");

        assert_eq!(status.version(), "0.1.0");
    }

    #[test]
    fn packaged_build_identity_requires_a_full_lowercase_commit() {
        let commit = "0123456789abcdef0123456789abcdef01234567";
        let status = runtime_status_from_metadata("0.1.0", commit, "release")
            .expect("packaged metadata should validate");

        assert_eq!(
            status.into_parts(),
            ("0.1.0".to_owned(), commit.to_owned(), "release".to_owned())
        );
        assert_eq!(
            runtime_status_from_metadata("0.1.0", "0123456", "release"),
            Err(RuntimeStatusError::InvalidBuildCommit)
        );
        assert_eq!(
            runtime_status_from_metadata(
                "0.1.0",
                "0123456789ABCDEF0123456789ABCDEF01234567",
                "release"
            ),
            Err(RuntimeStatusError::InvalidBuildCommit)
        );
    }

    #[test]
    fn build_profile_is_closed() {
        assert_eq!(
            runtime_status_from_metadata("0.1.0", "development", "profiling"),
            Err(RuntimeStatusError::InvalidBuildProfile)
        );
    }
}
