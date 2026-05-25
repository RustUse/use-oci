#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

use core::fmt;
use std::error::Error;

use use_oci_annotation::Annotation;
use use_oci_descriptor::OciDescriptor;

/// Errors returned when manifest metadata is invalid.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ManifestError {
    UnsupportedSchemaVersion,
}

impl fmt::Display for ManifestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion => {
                formatter.write_str("unsupported OCI manifest schema version")
            },
        }
    }
}

impl Error for ManifestError {}

/// OCI manifest schema version.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SchemaVersion(u8);

impl SchemaVersion {
    /// Creates a schema version. OCI image manifests use schema version 2.
    pub const fn new(value: u8) -> Result<Self, ManifestError> {
        if value == 2 {
            Ok(Self(value))
        } else {
            Err(ManifestError::UnsupportedSchemaVersion)
        }
    }

    /// Returns the version number.
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self.0
    }
}

impl Default for SchemaVersion {
    fn default() -> Self {
        Self(2)
    }
}

/// OCI image manifest primitives.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OciManifest {
    schema_version: SchemaVersion,
    config: OciDescriptor,
    layers: Vec<OciDescriptor>,
    subject: Option<OciDescriptor>,
    annotations: Vec<Annotation>,
}

impl OciManifest {
    /// Creates a manifest with a config descriptor.
    #[must_use]
    pub fn new(config: OciDescriptor) -> Self {
        Self {
            schema_version: SchemaVersion::default(),
            config,
            layers: Vec::new(),
            subject: None,
            annotations: Vec::new(),
        }
    }

    /// Adds a layer descriptor.
    #[must_use]
    pub fn with_layer(mut self, layer: OciDescriptor) -> Self {
        self.layers.push(layer);
        self
    }

    /// Adds a subject descriptor.
    #[must_use]
    pub fn with_subject(mut self, subject: OciDescriptor) -> Self {
        self.subject = Some(subject);
        self
    }

    /// Adds an annotation.
    #[must_use]
    pub fn with_annotation(mut self, annotation: Annotation) -> Self {
        self.annotations.push(annotation);
        self
    }

    /// Returns the schema version.
    #[must_use]
    pub const fn schema_version(&self) -> SchemaVersion {
        self.schema_version
    }

    /// Returns the config descriptor.
    #[must_use]
    pub const fn config(&self) -> &OciDescriptor {
        &self.config
    }

    /// Returns layer descriptors.
    #[must_use]
    pub fn layers(&self) -> &[OciDescriptor] {
        &self.layers
    }

    /// Returns the optional subject descriptor.
    #[must_use]
    pub const fn subject(&self) -> Option<&OciDescriptor> {
        self.subject.as_ref()
    }

    /// Returns manifest annotations.
    #[must_use]
    pub fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }
}

/// A named wrapper for manifest annotations.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ManifestAnnotations(Vec<Annotation>);

impl ManifestAnnotations {
    /// Creates an empty annotation list.
    #[must_use]
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    /// Adds an annotation.
    #[must_use]
    pub fn with_annotation(mut self, annotation: Annotation) -> Self {
        self.0.push(annotation);
        self
    }

    /// Returns annotations.
    #[must_use]
    pub fn as_slice(&self) -> &[Annotation] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::{ManifestAnnotations, ManifestError, OciManifest, SchemaVersion};
    use use_oci_annotation::Annotation;
    use use_oci_descriptor::{DescriptorSize, OciDescriptor};
    use use_oci_digest::OciDigest;
    use use_oci_media_type::OciMediaType;

    const SHA: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn descriptor() -> Result<OciDescriptor, Box<dyn std::error::Error>> {
        let digest: OciDigest = format!("sha256:{SHA}").parse()?;
        Ok(OciDescriptor::new(
            OciMediaType::image_config(),
            digest,
            DescriptorSize::new(1),
        ))
    }

    #[test]
    fn models_manifest_layer_lists() -> Result<(), Box<dyn std::error::Error>> {
        let manifest = OciManifest::new(descriptor()?)
            .with_layer(descriptor()?)
            .with_annotation(Annotation::title("Example")?);
        let annotations = ManifestAnnotations::new().with_annotation(Annotation::title("Example")?);

        assert_eq!(manifest.schema_version().as_u8(), 2);
        assert_eq!(manifest.layers().len(), 1);
        assert_eq!(manifest.annotations().len(), 1);
        assert_eq!(annotations.as_slice().len(), 1);
        assert_eq!(
            SchemaVersion::new(1),
            Err(ManifestError::UnsupportedSchemaVersion)
        );
        Ok(())
    }
}
