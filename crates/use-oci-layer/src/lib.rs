#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

use core::fmt;
use std::error::Error;

use use_oci_digest::OciDigest;
use use_oci_media_type::{KnownMediaType, OciMediaType};

/// Errors returned when OCI layer metadata is invalid.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayerError {
    InvalidMediaType,
}

impl fmt::Display for LayerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMediaType => {
                formatter.write_str("media type is not an OCI layer media type")
            },
        }
    }
}

impl Error for LayerError {}

/// OCI layer compression labels.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LayerCompression {
    Uncompressed,
    Gzip,
    Zstd,
}

impl LayerCompression {
    /// Returns the stable compression label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Uncompressed => "uncompressed",
            Self::Gzip => "gzip",
            Self::Zstd => "zstd",
        }
    }
}

impl fmt::Display for LayerCompression {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// OCI layer kind labels.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LayerKind {
    Filesystem,
    Foreign,
    Nondistributable,
    Unknown,
}

impl Default for LayerKind {
    fn default() -> Self {
        Self::Filesystem
    }
}

/// Layer size in bytes.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LayerSize(u64);

impl LayerSize {
    /// Creates a layer size.
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

/// An OCI layer media type.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LayerMediaType(OciMediaType);

impl LayerMediaType {
    /// Creates a layer media type after checking that it is layer-shaped.
    pub fn new(media_type: OciMediaType) -> Result<Self, LayerError> {
        if media_type.is_layer() {
            Ok(Self(media_type))
        } else {
            Err(LayerError::InvalidMediaType)
        }
    }

    /// Returns an uncompressed tar layer media type.
    #[must_use]
    pub const fn tar() -> Self {
        Self(OciMediaType::Known(KnownMediaType::LayerTar))
    }

    /// Returns a gzip-compressed tar layer media type.
    #[must_use]
    pub const fn gzip_tar() -> Self {
        Self(OciMediaType::Known(KnownMediaType::LayerTarGzip))
    }

    /// Returns a zstd-compressed tar layer media type.
    #[must_use]
    pub const fn zstd_tar() -> Self {
        Self(OciMediaType::Known(KnownMediaType::LayerTarZstd))
    }

    /// Returns the underlying media type.
    #[must_use]
    pub const fn media_type(&self) -> &OciMediaType {
        &self.0
    }

    /// Returns the compression implied by the media type.
    #[must_use]
    pub const fn compression(&self) -> LayerCompression {
        match self.0 {
            OciMediaType::Known(KnownMediaType::LayerTarGzip) => LayerCompression::Gzip,
            OciMediaType::Known(KnownMediaType::LayerTarZstd) => LayerCompression::Zstd,
            _ => LayerCompression::Uncompressed,
        }
    }
}

impl fmt::Display for LayerMediaType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// A layer diff ID digest marker.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DiffId(OciDigest);

impl DiffId {
    /// Creates a diff ID from a digest.
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

/// OCI layer metadata. This type does not extract or decompress layers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OciLayer {
    media_type: LayerMediaType,
    digest: OciDigest,
    size: LayerSize,
    diff_id: Option<DiffId>,
    kind: LayerKind,
}

impl OciLayer {
    /// Creates layer metadata.
    #[must_use]
    pub fn new(media_type: LayerMediaType, digest: OciDigest, size: LayerSize) -> Self {
        Self {
            media_type,
            digest,
            size,
            diff_id: None,
            kind: LayerKind::Filesystem,
        }
    }

    /// Adds a diff ID.
    #[must_use]
    pub fn with_diff_id(mut self, diff_id: DiffId) -> Self {
        self.diff_id = Some(diff_id);
        self
    }

    /// Adds a layer kind.
    #[must_use]
    pub const fn with_kind(mut self, kind: LayerKind) -> Self {
        self.kind = kind;
        self
    }

    /// Returns the layer media type.
    #[must_use]
    pub const fn media_type(&self) -> &LayerMediaType {
        &self.media_type
    }

    /// Returns the digest.
    #[must_use]
    pub const fn digest(&self) -> &OciDigest {
        &self.digest
    }

    /// Returns the size.
    #[must_use]
    pub const fn size(&self) -> LayerSize {
        self.size
    }

    /// Returns the compression marker.
    #[must_use]
    pub const fn compression(&self) -> LayerCompression {
        self.media_type.compression()
    }
}

#[cfg(test)]
mod tests {
    use super::{LayerCompression, LayerMediaType, LayerSize, OciLayer};
    use use_oci_digest::OciDigest;
    use use_oci_media_type::OciMediaType;

    const SHA: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    #[test]
    fn models_layer_metadata_without_extraction() -> Result<(), Box<dyn std::error::Error>> {
        let digest: OciDigest = format!("sha256:{SHA}").parse()?;
        let layer = OciLayer::new(LayerMediaType::gzip_tar(), digest, LayerSize::new(42));

        assert_eq!(layer.compression(), LayerCompression::Gzip);
        assert_eq!(layer.size().as_u64(), 42);
        assert!(LayerMediaType::new(OciMediaType::image_config()).is_err());
        Ok(())
    }
}
