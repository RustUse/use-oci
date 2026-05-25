#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

use core::{fmt, str::FromStr};
use std::error::Error;

use use_oci_annotation::Annotation;
use use_oci_descriptor::OciDescriptor;
use use_oci_digest::OciDigest;
use use_oci_media_type::OciMediaType;
use use_oci_platform::OciPlatform;
use use_oci_reference::ImageReference;

/// Errors returned when image metadata is invalid.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImageError {
    Empty,
    InvalidName,
}

impl fmt::Display for ImageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("OCI image value cannot be empty"),
            Self::InvalidName => formatter.write_str("invalid OCI image name"),
        }
    }
}

impl Error for ImageError {}

/// A lightweight image name label.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ImageName(String);

impl ImageName {
    /// Creates an image name label.
    pub fn new(value: impl AsRef<str>) -> Result<Self, ImageError> {
        let trimmed = value.as_ref().trim().to_ascii_lowercase();
        if trimmed.is_empty() {
            return Err(ImageError::Empty);
        }
        if trimmed.chars().any(char::is_whitespace)
            || trimmed.contains('@')
            || trimmed.contains(':')
        {
            return Err(ImageError::InvalidName);
        }
        Ok(Self(trimmed))
    }

    /// Returns the image name text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for ImageName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for ImageName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for ImageName {
    type Err = ImageError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

/// OCI image ID metadata, represented by a digest.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ImageId(OciDigest);

impl ImageId {
    /// Creates an image ID from a digest.
    #[must_use]
    pub const fn new(digest: OciDigest) -> Self {
        Self(digest)
    }

    /// Returns the digest.
    #[must_use]
    pub const fn digest(&self) -> &OciDigest {
        &self.0
    }
}

impl fmt::Display for ImageId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// OCI image kind labels.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ImageKind {
    Image,
    Artifact,
    Unknown,
}

impl Default for ImageKind {
    fn default() -> Self {
        Self::Image
    }
}

impl ImageKind {
    /// Returns the stable image kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Image => "image",
            Self::Artifact => "artifact",
            Self::Unknown => "unknown",
        }
    }
}

impl fmt::Display for ImageKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// OCI image metadata composed from focused primitive crates.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImageMetadata {
    name: ImageName,
    reference: Option<ImageReference>,
    id: Option<ImageId>,
    kind: ImageKind,
    descriptors: Vec<OciDescriptor>,
    platforms: Vec<OciPlatform>,
    media_types: Vec<OciMediaType>,
    annotations: Vec<Annotation>,
}

impl ImageMetadata {
    /// Creates image metadata from a name.
    #[must_use]
    pub fn new(name: ImageName) -> Self {
        Self {
            name,
            reference: None,
            id: None,
            kind: ImageKind::Image,
            descriptors: Vec::new(),
            platforms: Vec::new(),
            media_types: Vec::new(),
            annotations: Vec::new(),
        }
    }

    /// Adds an image reference.
    #[must_use]
    pub fn with_reference(mut self, reference: ImageReference) -> Self {
        self.reference = Some(reference);
        self
    }

    /// Adds an image ID.
    #[must_use]
    pub fn with_id(mut self, id: ImageId) -> Self {
        self.id = Some(id);
        self
    }

    /// Adds an image kind.
    #[must_use]
    pub const fn with_kind(mut self, kind: ImageKind) -> Self {
        self.kind = kind;
        self
    }

    /// Adds a descriptor.
    #[must_use]
    pub fn with_descriptor(mut self, descriptor: OciDescriptor) -> Self {
        self.descriptors.push(descriptor);
        self
    }

    /// Adds platform metadata.
    #[must_use]
    pub fn with_platform(mut self, platform: OciPlatform) -> Self {
        self.platforms.push(platform);
        self
    }

    /// Adds a media type marker.
    #[must_use]
    pub fn with_media_type(mut self, media_type: OciMediaType) -> Self {
        self.media_types.push(media_type);
        self
    }

    /// Adds an annotation.
    #[must_use]
    pub fn with_annotation(mut self, annotation: Annotation) -> Self {
        self.annotations.push(annotation);
        self
    }

    /// Returns the image name.
    #[must_use]
    pub const fn name(&self) -> &ImageName {
        &self.name
    }

    /// Returns the optional reference.
    #[must_use]
    pub const fn reference(&self) -> Option<&ImageReference> {
        self.reference.as_ref()
    }

    /// Returns the image kind.
    #[must_use]
    pub const fn kind(&self) -> ImageKind {
        self.kind
    }

    /// Returns descriptors.
    #[must_use]
    pub fn descriptors(&self) -> &[OciDescriptor] {
        &self.descriptors
    }

    /// Returns annotations.
    #[must_use]
    pub fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }
}

#[cfg(test)]
mod tests {
    use super::{ImageError, ImageId, ImageKind, ImageMetadata, ImageName};
    use use_oci_descriptor::{DescriptorSize, OciDescriptor};
    use use_oci_digest::OciDigest;
    use use_oci_media_type::OciMediaType;

    const SHA: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    #[test]
    fn composes_image_metadata() -> Result<(), Box<dyn std::error::Error>> {
        let digest: OciDigest = format!("sha256:{SHA}").parse()?;
        let descriptor = OciDescriptor::new(
            OciMediaType::image_manifest(),
            digest.clone(),
            DescriptorSize::new(10),
        );
        let image = ImageMetadata::new(ImageName::new("rustuse/app")?)
            .with_id(ImageId::new(digest))
            .with_kind(ImageKind::Artifact)
            .with_descriptor(descriptor);

        assert_eq!(image.name().as_str(), "rustuse/app");
        assert_eq!(image.kind(), ImageKind::Artifact);
        assert_eq!(image.descriptors().len(), 1);
        assert_eq!(ImageName::new("bad name"), Err(ImageError::InvalidName));
        Ok(())
    }
}
