#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

use core::{fmt, str::FromStr};
use std::error::Error;

/// Common OCI annotation key for a title.
pub const OCI_TITLE: &str = "org.opencontainers.image.title";
/// Common OCI annotation key for a description.
pub const OCI_DESCRIPTION: &str = "org.opencontainers.image.description";
/// Common OCI annotation key for a source URL.
pub const OCI_SOURCE: &str = "org.opencontainers.image.source";
/// Common OCI annotation key for a revision.
pub const OCI_REVISION: &str = "org.opencontainers.image.revision";
/// Common OCI annotation key for a version.
pub const OCI_VERSION: &str = "org.opencontainers.image.version";
/// Common OCI annotation key for creation metadata.
pub const OCI_CREATED: &str = "org.opencontainers.image.created";
/// Common OCI annotation key for authors.
pub const OCI_AUTHORS: &str = "org.opencontainers.image.authors";
/// Common OCI annotation key for licenses.
pub const OCI_LICENSES: &str = "org.opencontainers.image.licenses";
/// Common OCI annotation key for documentation.
pub const OCI_DOCUMENTATION: &str = "org.opencontainers.image.documentation";
/// Common OCI annotation key for a URL.
pub const OCI_URL: &str = "org.opencontainers.image.url";

/// Errors returned when annotation text is invalid.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnnotationError {
    EmptyKey,
    InvalidKey,
    InvalidValue,
}

impl fmt::Display for AnnotationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyKey => formatter.write_str("OCI annotation key cannot be empty"),
            Self::InvalidKey => formatter.write_str("invalid OCI annotation key"),
            Self::InvalidValue => formatter.write_str("invalid OCI annotation value"),
        }
    }
}

impl Error for AnnotationError {}

/// A validated OCI annotation key.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AnnotationKey(String);

impl AnnotationKey {
    /// Creates an annotation key.
    pub fn new(value: impl AsRef<str>) -> Result<Self, AnnotationError> {
        let trimmed = value.as_ref().trim();
        validate_key(trimmed)?;
        Ok(Self(trimmed.to_string()))
    }

    /// Returns the key text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for AnnotationKey {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for AnnotationKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for AnnotationKey {
    type Err = AnnotationError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

impl TryFrom<&str> for AnnotationKey {
    type Error = AnnotationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

/// A validated OCI annotation value.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AnnotationValue(String);

impl AnnotationValue {
    /// Creates an annotation value.
    pub fn new(value: impl AsRef<str>) -> Result<Self, AnnotationError> {
        let value = value.as_ref();
        if value.contains('\0') {
            return Err(AnnotationError::InvalidValue);
        }
        Ok(Self(value.to_string()))
    }

    /// Returns the value text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for AnnotationValue {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for AnnotationValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// An OCI annotation key/value pair.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Annotation {
    key: AnnotationKey,
    value: AnnotationValue,
}

impl Annotation {
    /// Creates an annotation.
    pub fn new(key: impl AsRef<str>, value: impl AsRef<str>) -> Result<Self, AnnotationError> {
        Ok(Self {
            key: AnnotationKey::new(key)?,
            value: AnnotationValue::new(value)?,
        })
    }

    /// Creates an OCI title annotation.
    pub fn title(value: impl AsRef<str>) -> Result<Self, AnnotationError> {
        Self::new(OCI_TITLE, value)
    }

    /// Creates an OCI description annotation.
    pub fn description(value: impl AsRef<str>) -> Result<Self, AnnotationError> {
        Self::new(OCI_DESCRIPTION, value)
    }

    /// Creates an OCI source annotation.
    pub fn source(value: impl AsRef<str>) -> Result<Self, AnnotationError> {
        Self::new(OCI_SOURCE, value)
    }

    /// Returns the key.
    #[must_use]
    pub const fn key(&self) -> &AnnotationKey {
        &self.key
    }

    /// Returns the value.
    #[must_use]
    pub const fn value(&self) -> &AnnotationValue {
        &self.value
    }
}

impl fmt::Display for Annotation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}={}", self.key, self.value)
    }
}

fn validate_key(value: &str) -> Result<(), AnnotationError> {
    if value.is_empty() {
        return Err(AnnotationError::EmptyKey);
    }
    if value.starts_with(['.', '/', '-'])
        || value.ends_with(['.', '/', '-'])
        || value.chars().any(char::is_whitespace)
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_' | b'/'))
    {
        Err(AnnotationError::InvalidKey)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Annotation, AnnotationError, AnnotationKey, OCI_TITLE};

    #[test]
    fn validates_and_renders_annotations() -> Result<(), Box<dyn std::error::Error>> {
        let annotation = Annotation::title("RustUse OCI")?;

        assert_eq!(annotation.key().as_str(), OCI_TITLE);
        assert_eq!(
            annotation.to_string(),
            "org.opencontainers.image.title=RustUse OCI"
        );
        assert_eq!(
            AnnotationKey::new("bad key"),
            Err(AnnotationError::InvalidKey)
        );
        assert_eq!(
            Annotation::new("example.key", "bad\0value"),
            Err(AnnotationError::InvalidValue)
        );
        Ok(())
    }
}
