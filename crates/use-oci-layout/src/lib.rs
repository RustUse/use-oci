#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

use core::{fmt, str::FromStr};
use std::{error::Error, path::PathBuf};

/// Errors returned when layout metadata is invalid.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayoutError {
    Empty,
    UnsupportedVersion,
    InvalidPath,
}

impl fmt::Display for LayoutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("OCI layout value cannot be empty"),
            Self::UnsupportedVersion => formatter.write_str("unsupported OCI layout version"),
            Self::InvalidPath => formatter.write_str("invalid OCI layout path"),
        }
    }
}

impl Error for LayoutError {}

/// OCI image layout version marker.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LayoutVersion(String);

impl LayoutVersion {
    /// Creates a layout version marker. OCI image layout v1 uses `1.0.0`.
    pub fn new(value: impl AsRef<str>) -> Result<Self, LayoutError> {
        let trimmed = value.as_ref().trim();
        if trimmed == "1.0.0" {
            Ok(Self(trimmed.to_string()))
        } else if trimmed.is_empty() {
            Err(LayoutError::Empty)
        } else {
            Err(LayoutError::UnsupportedVersion)
        }
    }

    /// Returns the layout version text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for LayoutVersion {
    fn default() -> Self {
        Self("1.0.0".to_string())
    }
}

impl fmt::Display for LayoutVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for LayoutVersion {
    type Err = LayoutError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

/// Marker for the `blobs` directory.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BlobsDirectory;

/// Marker for `index.json`.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct IndexFile;

/// Marker for the optional `refs` directory.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RefsDirectory;

/// Lexical OCI image layout paths. This type does not create directories or files.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OciLayoutPaths {
    root: PathBuf,
}

impl OciLayoutPaths {
    /// Creates layout path helpers from a lexical root path.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Returns the root path.
    #[must_use]
    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    /// Returns the `blobs` directory path.
    #[must_use]
    pub fn blobs_dir(&self) -> PathBuf {
        self.root.join("blobs")
    }

    /// Returns the `index.json` path.
    #[must_use]
    pub fn index_file(&self) -> PathBuf {
        self.root.join("index.json")
    }

    /// Returns the `refs` directory path.
    #[must_use]
    pub fn refs_dir(&self) -> PathBuf {
        self.root.join("refs")
    }

    /// Returns a lexical blob path for an algorithm and encoded value.
    pub fn blob_path(
        &self,
        algorithm: impl AsRef<str>,
        encoded: impl AsRef<str>,
    ) -> Result<PathBuf, LayoutError> {
        let algorithm = validate_part(algorithm.as_ref())?;
        let encoded = validate_part(encoded.as_ref())?;
        Ok(self.blobs_dir().join(algorithm).join(encoded))
    }
}

fn validate_part(value: &str) -> Result<&str, LayoutError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(LayoutError::Empty);
    }
    if trimmed.contains(['/', '\\']) || trimmed.chars().any(char::is_whitespace) {
        return Err(LayoutError::InvalidPath);
    }
    Ok(trimmed)
}

#[cfg(test)]
mod tests {
    use super::{LayoutError, LayoutVersion, OciLayoutPaths};

    #[test]
    fn renders_layout_paths_without_mutation() -> Result<(), LayoutError> {
        let paths = OciLayoutPaths::new("layout");
        let blob = paths.blob_path("sha256", "abc")?;

        assert_eq!(LayoutVersion::default().as_str(), "1.0.0");
        assert!(paths.index_file().ends_with("index.json"));
        assert!(blob.ends_with("abc"));
        assert_eq!(
            LayoutVersion::new("2.0.0"),
            Err(LayoutError::UnsupportedVersion)
        );
        Ok(())
    }
}
