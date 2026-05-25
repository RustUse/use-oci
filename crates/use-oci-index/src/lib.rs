#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

use use_oci_annotation::Annotation;
use use_oci_descriptor::OciDescriptor;
use use_oci_platform::OciPlatform;

/// A platform-specific manifest reference in an OCI image index.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexManifest {
    descriptor: OciDescriptor,
    platform: Option<OciPlatform>,
    annotations: Vec<Annotation>,
}

impl IndexManifest {
    /// Creates an index manifest reference.
    #[must_use]
    pub fn new(descriptor: OciDescriptor) -> Self {
        Self {
            descriptor,
            platform: None,
            annotations: Vec::new(),
        }
    }

    /// Adds platform metadata.
    #[must_use]
    pub fn with_platform(mut self, platform: OciPlatform) -> Self {
        self.platform = Some(platform);
        self
    }

    /// Adds an annotation.
    #[must_use]
    pub fn with_annotation(mut self, annotation: Annotation) -> Self {
        self.annotations.push(annotation);
        self
    }

    /// Returns the descriptor.
    #[must_use]
    pub const fn descriptor(&self) -> &OciDescriptor {
        &self.descriptor
    }

    /// Returns the optional platform.
    #[must_use]
    pub const fn platform(&self) -> Option<&OciPlatform> {
        self.platform.as_ref()
    }

    /// Returns annotations.
    #[must_use]
    pub fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }
}

/// OCI image index metadata.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OciIndex {
    manifests: Vec<IndexManifest>,
    annotations: Vec<Annotation>,
}

impl OciIndex {
    /// Creates an empty image index.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            manifests: Vec::new(),
            annotations: Vec::new(),
        }
    }

    /// Adds a manifest reference.
    #[must_use]
    pub fn with_manifest(mut self, manifest: IndexManifest) -> Self {
        self.manifests.push(manifest);
        self
    }

    /// Adds an annotation.
    #[must_use]
    pub fn with_annotation(mut self, annotation: Annotation) -> Self {
        self.annotations.push(annotation);
        self
    }

    /// Returns manifest references.
    #[must_use]
    pub fn manifests(&self) -> &[IndexManifest] {
        &self.manifests
    }

    /// Returns index annotations.
    #[must_use]
    pub fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }
}

/// Lightweight index metadata counters.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct IndexMetadata {
    manifest_count: usize,
}

impl IndexMetadata {
    /// Creates index metadata.
    #[must_use]
    pub const fn new(manifest_count: usize) -> Self {
        Self { manifest_count }
    }

    /// Returns the manifest count.
    #[must_use]
    pub const fn manifest_count(self) -> usize {
        self.manifest_count
    }
}

#[cfg(test)]
mod tests {
    use super::{IndexManifest, IndexMetadata, OciIndex};
    use use_oci_descriptor::{DescriptorSize, OciDescriptor};
    use use_oci_digest::OciDigest;
    use use_oci_media_type::OciMediaType;
    use use_oci_platform::{OciArchitecture, OciOs, OciPlatform};

    const SHA: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    #[test]
    fn models_index_entries() -> Result<(), Box<dyn std::error::Error>> {
        let digest: OciDigest = format!("sha256:{SHA}").parse()?;
        let descriptor = OciDescriptor::new(
            OciMediaType::image_manifest(),
            digest,
            DescriptorSize::new(10),
        );
        let platform = OciPlatform::new(OciOs::Linux, OciArchitecture::Arm64);
        let entry = IndexManifest::new(descriptor).with_platform(platform.clone());
        let index = OciIndex::new().with_manifest(entry);

        assert_eq!(index.manifests().len(), 1);
        assert_eq!(index.manifests()[0].platform(), Some(&platform));
        assert_eq!(
            IndexMetadata::new(index.manifests().len()).manifest_count(),
            1
        );
        Ok(())
    }
}
