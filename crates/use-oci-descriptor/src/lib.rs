#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

use core::fmt;
use std::error::Error;

use use_oci_annotation::Annotation;
use use_oci_digest::OciDigest;
use use_oci_media_type::OciMediaType;
use use_oci_platform::OciPlatform;

/// Errors returned when descriptor metadata is invalid.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DescriptorError {
    Empty,
    InvalidUrl,
    InvalidData,
}

impl fmt::Display for DescriptorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("OCI descriptor value cannot be empty"),
            Self::InvalidUrl => formatter.write_str("invalid OCI descriptor URL"),
            Self::InvalidData => formatter.write_str("invalid OCI descriptor data marker"),
        }
    }
}

impl Error for DescriptorError {}

/// Descriptor size in bytes.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DescriptorSize(u64);

impl DescriptorSize {
    /// Creates a descriptor size.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the size in bytes.
    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

/// A descriptor URL string. This type does not fetch anything.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DescriptorUrl(String);

impl DescriptorUrl {
    /// Creates a descriptor URL marker.
    pub fn new(value: impl AsRef<str>) -> Result<Self, DescriptorError> {
        let trimmed = value.as_ref().trim();
        if trimmed.is_empty() {
            return Err(DescriptorError::Empty);
        }
        if trimmed.chars().any(char::is_whitespace) || !trimmed.contains("://") {
            return Err(DescriptorError::InvalidUrl);
        }
        Ok(Self(trimmed.to_string()))
    }

    /// Returns the URL text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DescriptorUrl {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// A descriptor artifact type marker.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ArtifactType(String);

impl ArtifactType {
    /// Creates an artifact type marker.
    pub fn new(value: impl AsRef<str>) -> Result<Self, DescriptorError> {
        let media_type = OciMediaType::custom(value).map_err(|_| DescriptorError::Empty)?;
        Ok(Self(media_type.to_string()))
    }

    /// Returns the artifact type text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ArtifactType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Embedded descriptor data marker.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DescriptorData(String);

impl DescriptorData {
    /// Creates an embedded data marker.
    pub fn new(value: impl AsRef<str>) -> Result<Self, DescriptorError> {
        let value = value.as_ref();
        if value.contains('\0') {
            return Err(DescriptorError::InvalidData);
        }
        Ok(Self(value.to_string()))
    }

    /// Returns the data marker text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// OCI descriptor metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OciDescriptor {
    media_type: OciMediaType,
    digest: OciDigest,
    size: DescriptorSize,
    urls: Vec<DescriptorUrl>,
    annotations: Vec<Annotation>,
    data: Option<DescriptorData>,
    artifact_type: Option<ArtifactType>,
    platform: Option<OciPlatform>,
}

impl OciDescriptor {
    /// Creates descriptor metadata from required fields.
    #[must_use]
    pub fn new(media_type: OciMediaType, digest: OciDigest, size: DescriptorSize) -> Self {
        Self {
            media_type,
            digest,
            size,
            urls: Vec::new(),
            annotations: Vec::new(),
            data: None,
            artifact_type: None,
            platform: None,
        }
    }

    /// Adds a URL marker.
    #[must_use]
    pub fn with_url(mut self, url: DescriptorUrl) -> Self {
        self.urls.push(url);
        self
    }

    /// Adds an annotation.
    #[must_use]
    pub fn with_annotation(mut self, annotation: Annotation) -> Self {
        self.annotations.push(annotation);
        self
    }

    /// Adds embedded data marker text.
    #[must_use]
    pub fn with_data(mut self, data: DescriptorData) -> Self {
        self.data = Some(data);
        self
    }

    /// Adds an artifact type.
    #[must_use]
    pub fn with_artifact_type(mut self, artifact_type: ArtifactType) -> Self {
        self.artifact_type = Some(artifact_type);
        self
    }

    /// Adds a platform.
    #[must_use]
    pub fn with_platform(mut self, platform: OciPlatform) -> Self {
        self.platform = Some(platform);
        self
    }

    /// Returns the media type.
    #[must_use]
    pub const fn media_type(&self) -> &OciMediaType {
        &self.media_type
    }

    /// Returns the digest.
    #[must_use]
    pub const fn digest(&self) -> &OciDigest {
        &self.digest
    }

    /// Returns the size.
    #[must_use]
    pub const fn size(&self) -> DescriptorSize {
        self.size
    }

    /// Returns URL markers.
    #[must_use]
    pub fn urls(&self) -> &[DescriptorUrl] {
        &self.urls
    }

    /// Returns annotations.
    #[must_use]
    pub fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }

    /// Returns the optional platform.
    #[must_use]
    pub const fn platform(&self) -> Option<&OciPlatform> {
        self.platform.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::{DescriptorSize, DescriptorUrl, OciDescriptor};
    use use_oci_annotation::Annotation;
    use use_oci_digest::OciDigest;
    use use_oci_media_type::OciMediaType;

    const SHA: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    #[test]
    fn builds_descriptor_metadata() -> Result<(), Box<dyn std::error::Error>> {
        let digest: OciDigest = format!("sha256:{SHA}").parse()?;
        let descriptor = OciDescriptor::new(
            OciMediaType::image_manifest(),
            digest,
            DescriptorSize::new(12),
        )
        .with_url(DescriptorUrl::new("https://example.com/blob")?)
        .with_annotation(Annotation::title("Example")?);

        assert_eq!(descriptor.size().as_u64(), 12);
        assert_eq!(descriptor.urls().len(), 1);
        assert_eq!(descriptor.annotations().len(), 1);
        Ok(())
    }
}
