#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

use core::{fmt, str::FromStr};
use std::error::Error;

/// OCI image manifest media type.
pub const IMAGE_MANIFEST: &str = "application/vnd.oci.image.manifest.v1+json";
/// OCI image index media type.
pub const IMAGE_INDEX: &str = "application/vnd.oci.image.index.v1+json";
/// OCI image config media type.
pub const IMAGE_CONFIG: &str = "application/vnd.oci.image.config.v1+json";
/// OCI artifact manifest media type.
pub const ARTIFACT_MANIFEST: &str = "application/vnd.oci.artifact.manifest.v1+json";
/// OCI uncompressed tar layer media type.
pub const LAYER_TAR: &str = "application/vnd.oci.image.layer.v1.tar";
/// OCI gzip-compressed tar layer media type.
pub const LAYER_TAR_GZIP: &str = "application/vnd.oci.image.layer.v1.tar+gzip";
/// OCI zstd-compressed tar layer media type.
pub const LAYER_TAR_ZSTD: &str = "application/vnd.oci.image.layer.v1.tar+zstd";

/// Errors returned when media type text is invalid.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MediaTypeError {
    Empty,
    MissingSlash,
    InvalidCharacter,
}

impl fmt::Display for MediaTypeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("OCI media type cannot be empty"),
            Self::MissingSlash => formatter.write_str("OCI media type must contain '/'"),
            Self::InvalidCharacter => {
                formatter.write_str("OCI media type contains invalid characters")
            },
        }
    }
}

impl Error for MediaTypeError {}

/// Known OCI media type labels.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum KnownMediaType {
    ImageManifest,
    ImageIndex,
    ImageConfig,
    ArtifactManifest,
    LayerTar,
    LayerTarGzip,
    LayerTarZstd,
}

impl KnownMediaType {
    /// Returns the stable media type string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ImageManifest => IMAGE_MANIFEST,
            Self::ImageIndex => IMAGE_INDEX,
            Self::ImageConfig => IMAGE_CONFIG,
            Self::ArtifactManifest => ARTIFACT_MANIFEST,
            Self::LayerTar => LAYER_TAR,
            Self::LayerTarGzip => LAYER_TAR_GZIP,
            Self::LayerTarZstd => LAYER_TAR_ZSTD,
        }
    }
}

impl fmt::Display for KnownMediaType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for KnownMediaType {
    type Err = MediaTypeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim() {
            IMAGE_MANIFEST => Ok(Self::ImageManifest),
            IMAGE_INDEX => Ok(Self::ImageIndex),
            IMAGE_CONFIG => Ok(Self::ImageConfig),
            ARTIFACT_MANIFEST => Ok(Self::ArtifactManifest),
            LAYER_TAR => Ok(Self::LayerTar),
            LAYER_TAR_GZIP => Ok(Self::LayerTarGzip),
            LAYER_TAR_ZSTD => Ok(Self::LayerTarZstd),
            "" => Err(MediaTypeError::Empty),
            _ => Err(MediaTypeError::InvalidCharacter),
        }
    }
}

/// A known or custom OCI media type.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum OciMediaType {
    Known(KnownMediaType),
    Custom(String),
}

impl OciMediaType {
    /// Creates a custom media type after conservative validation.
    pub fn custom(value: impl AsRef<str>) -> Result<Self, MediaTypeError> {
        validate_media_type(value.as_ref()).map(|value| Self::Custom(value.to_string()))
    }

    /// Returns the OCI image manifest media type.
    #[must_use]
    pub const fn image_manifest() -> Self {
        Self::Known(KnownMediaType::ImageManifest)
    }

    /// Returns the OCI image index media type.
    #[must_use]
    pub const fn image_index() -> Self {
        Self::Known(KnownMediaType::ImageIndex)
    }

    /// Returns the OCI image config media type.
    #[must_use]
    pub const fn image_config() -> Self {
        Self::Known(KnownMediaType::ImageConfig)
    }

    /// Returns the OCI artifact manifest media type.
    #[must_use]
    pub const fn artifact_manifest() -> Self {
        Self::Known(KnownMediaType::ArtifactManifest)
    }

    /// Returns media type text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Known(known) => known.as_str(),
            Self::Custom(value) => value,
        }
    }

    /// Returns true for layer media types.
    #[must_use]
    pub fn is_layer(&self) -> bool {
        matches!(
            self,
            Self::Known(
                KnownMediaType::LayerTar
                    | KnownMediaType::LayerTarGzip
                    | KnownMediaType::LayerTarZstd
            )
        )
    }
}

impl AsRef<str> for OciMediaType {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for OciMediaType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl From<KnownMediaType> for OciMediaType {
    fn from(value: KnownMediaType) -> Self {
        Self::Known(value)
    }
}

impl FromStr for OciMediaType {
    type Err = MediaTypeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let trimmed = validate_media_type(value)?;
        KnownMediaType::from_str(trimmed).map_or_else(
            |_| Ok(Self::Custom(trimmed.to_string())),
            |known| Ok(Self::Known(known)),
        )
    }
}

impl TryFrom<&str> for OciMediaType {
    type Error = MediaTypeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

fn validate_media_type(value: &str) -> Result<&str, MediaTypeError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(MediaTypeError::Empty);
    }
    if !trimmed.contains('/') {
        return Err(MediaTypeError::MissingSlash);
    }
    if trimmed
        .bytes()
        .any(|byte| byte.is_ascii_control() || byte.is_ascii_whitespace())
    {
        return Err(MediaTypeError::InvalidCharacter);
    }
    Ok(trimmed)
}

#[cfg(test)]
mod tests {
    use super::{KnownMediaType, MediaTypeError, OciMediaType};

    #[test]
    fn parses_known_and_custom_media_types() -> Result<(), Box<dyn std::error::Error>> {
        let manifest: OciMediaType = "application/vnd.oci.image.manifest.v1+json".parse()?;
        let custom: OciMediaType = "application/vnd.example.artifact.v1+json".parse()?;

        assert_eq!(manifest, OciMediaType::Known(KnownMediaType::ImageManifest));
        assert_eq!(custom.as_str(), "application/vnd.example.artifact.v1+json");
        assert_eq!(OciMediaType::image_index().to_string(), super::IMAGE_INDEX);
        assert_eq!(
            "plain".parse::<OciMediaType>(),
            Err(MediaTypeError::MissingSlash)
        );
        Ok(())
    }
}
