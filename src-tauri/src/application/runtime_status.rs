/// Rust-owned application status before it crosses the Tauri command boundary.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct RuntimeStatus {
    version: String,
}

/// Failure to construct a valid Rust-owned runtime status.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum RuntimeStatusError {
    MissingVersion,
}

/// Builds status from the package metadata compiled into the trusted runtime.
pub(crate) fn current_runtime_status() -> Result<RuntimeStatus, RuntimeStatusError> {
    runtime_status_from_version(env!("CARGO_PKG_VERSION"))
}

impl RuntimeStatus {
    /// Moves the validated package version into an IPC response.
    pub(crate) fn into_version(self) -> String {
        self.version
    }
}

fn runtime_status_from_version(version: &str) -> Result<RuntimeStatus, RuntimeStatusError> {
    Ok(RuntimeStatus {
        version: validated_version(version)?,
    })
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

        assert_eq!(status.into_version(), env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn blank_version_is_rejected() {
        let result = runtime_status_from_version("  ");

        assert_eq!(result, Err(RuntimeStatusError::MissingVersion));
    }

    #[test]
    fn surrounding_version_whitespace_is_removed() {
        let status = runtime_status_from_version(" 0.1.0 ").expect("version should be valid");

        assert_eq!(status.into_version(), "0.1.0");
    }
}
