#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

//! Thin facade for primitive OCI vocabulary crates.

#[cfg(feature = "image")]
pub use use_oci_image as image;

#[cfg(feature = "manifest")]
pub use use_oci_manifest as manifest;

#[cfg(feature = "index")]
pub use use_oci_index as index;

#[cfg(feature = "descriptor")]
pub use use_oci_descriptor as descriptor;

#[cfg(feature = "layer")]
pub use use_oci_layer as layer;

#[cfg(feature = "config")]
pub use use_oci_config as config;

#[cfg(feature = "layout")]
pub use use_oci_layout as layout;

#[cfg(feature = "annotation")]
pub use use_oci_annotation as annotation;

#[cfg(feature = "media-type")]
pub use use_oci_media_type as media_type;

#[cfg(feature = "platform")]
pub use use_oci_platform as platform;

#[cfg(feature = "digest")]
pub use use_oci_digest as digest;

#[cfg(feature = "distribution")]
pub use use_oci_distribution as distribution;

#[cfg(feature = "reference")]
pub use use_oci_reference as reference;

#[cfg(feature = "runtime")]
pub use use_oci_runtime as runtime;

#[cfg(feature = "hook")]
pub use use_oci_hook as hook;

#[cfg(feature = "namespace")]
pub use use_oci_namespace as namespace;

#[cfg(feature = "tag")]
pub use use_oci_tag as tag;

/// Common OCI primitive re-exports.
pub mod prelude {
    #[cfg(feature = "annotation")]
    pub use use_oci_annotation::{Annotation, AnnotationKey, AnnotationValue};
    #[cfg(feature = "descriptor")]
    pub use use_oci_descriptor::{DescriptorSize, DescriptorUrl, OciDescriptor};
    #[cfg(feature = "digest")]
    pub use use_oci_digest::{DigestAlgorithm, DigestValue, OciDigest};
    #[cfg(feature = "distribution")]
    pub use use_oci_distribution::{
        DistributionRoute, ManifestReference, RegistryHost, RepositoryName,
    };
    #[cfg(feature = "image")]
    pub use use_oci_image::{ImageId, ImageKind, ImageMetadata, ImageName};
    #[cfg(feature = "index")]
    pub use use_oci_index::{IndexManifest, OciIndex};
    #[cfg(feature = "layer")]
    pub use use_oci_layer::{LayerMediaType, LayerSize, OciLayer};
    #[cfg(feature = "layout")]
    pub use use_oci_layout::{LayoutVersion, OciLayoutPaths};
    #[cfg(feature = "manifest")]
    pub use use_oci_manifest::{OciManifest, SchemaVersion};
    #[cfg(feature = "media-type")]
    pub use use_oci_media_type::{KnownMediaType, OciMediaType};
    #[cfg(feature = "namespace")]
    pub use use_oci_namespace::{Namespace, NamespaceKind};
    #[cfg(feature = "platform")]
    pub use use_oci_platform::{OciArchitecture, OciOs, OciPlatform};
    #[cfg(feature = "reference")]
    pub use use_oci_reference::{
        CanonicalReference, DigestedReference, ImageReference, TaggedReference,
    };
    #[cfg(feature = "runtime")]
    pub use use_oci_runtime::{Mount, MountKind, RootFilesystem, RuntimeSpec};
    #[cfg(feature = "tag")]
    pub use use_oci_tag::{ArchitectureTag, OciTag, VersionTag};
}
