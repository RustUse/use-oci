#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

use core::{fmt, str::FromStr};
use std::error::Error;

/// Errors returned when OCI tag text is invalid.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OciTagError {
    Empty,
    TooLong,
    InvalidStart,
    InvalidCharacter,
    NotVersionLike,
    NotArchitectureLike,
}

impl fmt::Display for OciTagError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("OCI tag cannot be empty"),
            Self::TooLong => formatter.write_str("OCI tag cannot exceed 128 characters"),
            Self::InvalidStart => {
                formatter.write_str("OCI tag must start with an ASCII word character")
            },
            Self::InvalidCharacter => formatter.write_str("OCI tag contains invalid characters"),
            Self::NotVersionLike => formatter.write_str("OCI tag is not version-like"),
            Self::NotArchitectureLike => formatter.write_str("OCI tag is not architecture-like"),
        }
    }
}

impl Error for OciTagError {}

/// A validated OCI tag.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OciTag(String);

impl OciTag {
    /// Creates a validated tag.
    pub fn new(value: impl AsRef<str>) -> Result<Self, OciTagError> {
        let trimmed = value.as_ref().trim();
        validate_tag(trimmed)?;
        Ok(Self(trimmed.to_string()))
    }

    /// Returns the conventional `latest` marker.
    #[must_use]
    pub fn latest() -> Self {
        Self("latest".to_string())
    }

    /// Returns the tag text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns true when the tag is exactly `latest`.
    #[must_use]
    pub fn is_latest(&self) -> bool {
        self.as_str() == "latest"
    }

    /// Returns true when the tag looks like a simple semantic version.
    #[must_use]
    pub fn is_version_like(&self) -> bool {
        is_version_like(self.as_str())
    }

    /// Returns true when the tag contains a common architecture label.
    #[must_use]
    pub fn is_architecture_like(&self) -> bool {
        architecture_token(self.as_str()).is_some()
    }
}

impl AsRef<str> for OciTag {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for OciTag {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for OciTag {
    type Err = OciTagError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

impl TryFrom<&str> for OciTag {
    type Error = OciTagError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

/// A version-looking OCI tag.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct VersionTag(OciTag);

impl VersionTag {
    /// Creates a version-looking tag.
    pub fn new(value: impl AsRef<str>) -> Result<Self, OciTagError> {
        let tag = OciTag::new(value)?;
        if tag.is_version_like() {
            Ok(Self(tag))
        } else {
            Err(OciTagError::NotVersionLike)
        }
    }

    /// Returns the tag text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for VersionTag {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// An architecture-looking OCI tag.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ArchitectureTag(OciTag);

impl ArchitectureTag {
    /// Creates an architecture-looking tag.
    pub fn new(value: impl AsRef<str>) -> Result<Self, OciTagError> {
        let tag = OciTag::new(value)?;
        if tag.is_architecture_like() {
            Ok(Self(tag))
        } else {
            Err(OciTagError::NotArchitectureLike)
        }
    }

    /// Returns the tag text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for ArchitectureTag {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Returns true when text is a valid OCI tag.
#[must_use]
pub fn is_valid_oci_tag(value: impl AsRef<str>) -> bool {
    validate_tag(value.as_ref().trim()).is_ok()
}

fn validate_tag(value: &str) -> Result<(), OciTagError> {
    if value.is_empty() {
        return Err(OciTagError::Empty);
    }
    if value.len() > 128 {
        return Err(OciTagError::TooLong);
    }
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return Err(OciTagError::Empty);
    };
    if !(first.is_ascii_alphanumeric() || first == '_') {
        return Err(OciTagError::InvalidStart);
    }
    if chars.any(|character| {
        !(character.is_ascii_alphanumeric() || matches!(character, '_' | '.' | '-'))
    }) {
        return Err(OciTagError::InvalidCharacter);
    }
    Ok(())
}

fn is_version_like(value: &str) -> bool {
    let value = value.strip_prefix('v').unwrap_or(value);
    let core = value.split_once('-').map_or(value, |(core, _)| core);
    let mut parts = core.split('.');
    matches!(
        (parts.next(), parts.next(), parts.next(), parts.next()),
        (Some(major), Some(minor), Some(patch), None)
            if is_digits(major) && is_digits(minor) && is_digits(patch)
    )
}

fn architecture_token(value: &str) -> Option<&str> {
    value.split(['-', '_', '.']).find(|part| {
        matches!(
            *part,
            "amd64" | "arm64" | "arm" | "386" | "ppc64le" | "riscv64" | "s390x" | "wasm"
        )
    })
}

fn is_digits(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::{ArchitectureTag, OciTag, OciTagError, VersionTag, is_valid_oci_tag};

    #[test]
    fn validates_and_classifies_tags() -> Result<(), Box<dyn std::error::Error>> {
        let tag: OciTag = "v1.2.3-arm64".parse()?;

        assert!(tag.is_version_like());
        assert!(tag.is_architecture_like());
        assert!(OciTag::latest().is_latest());
        assert!(is_valid_oci_tag("_dev"));
        assert_eq!(OciTag::new("-bad"), Err(OciTagError::InvalidStart));
        assert_eq!(VersionTag::new("release"), Err(OciTagError::NotVersionLike));
        assert_eq!(
            ArchitectureTag::new("v1.2.3"),
            Err(OciTagError::NotArchitectureLike)
        );
        Ok(())
    }
}
