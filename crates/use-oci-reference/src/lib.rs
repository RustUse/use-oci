#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

use core::{fmt, str::FromStr};
use std::error::Error;

pub use use_oci_digest::OciDigest as Digest;
use use_oci_digest::OciDigest;
pub use use_oci_distribution::{RegistryHost as Registry, RepositoryName as Repository};
use use_oci_distribution::{RegistryHost, RepositoryName};
pub use use_oci_tag::OciTag as TagName;
use use_oci_tag::{OciTag, OciTagError};

/// Errors returned when image references are invalid.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReferenceError {
    Empty,
    InvalidName,
    InvalidTag,
    InvalidDigest,
}

impl fmt::Display for ReferenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("OCI reference cannot be empty"),
            Self::InvalidName => formatter.write_str("invalid OCI image name"),
            Self::InvalidTag => formatter.write_str("invalid OCI tag"),
            Self::InvalidDigest => formatter.write_str("invalid OCI digest"),
        }
    }
}

impl Error for ReferenceError {}

/// A repository name with an optional registry.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ImageName {
    registry: Option<RegistryHost>,
    repository: RepositoryName,
    value: String,
}

impl ImageName {
    /// Creates an image name from reference text without tag or digest components.
    pub fn new(value: impl AsRef<str>) -> Result<Self, ReferenceError> {
        let trimmed = value.as_ref().trim();
        if trimmed.is_empty() {
            return Err(ReferenceError::Empty);
        }
        if trimmed.contains('@') || has_tag_separator(trimmed) {
            return Err(ReferenceError::InvalidName);
        }
        parse_name(trimmed)
    }

    /// Returns the optional registry.
    #[must_use]
    pub const fn registry(&self) -> Option<&RegistryHost> {
        self.registry.as_ref()
    }

    /// Returns the repository.
    #[must_use]
    pub const fn repository(&self) -> &RepositoryName {
        &self.repository
    }

    /// Returns the name text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
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

/// A name plus tag reference.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TaggedReference {
    name: ImageName,
    tag: OciTag,
}

impl TaggedReference {
    /// Creates a tagged reference.
    #[must_use]
    pub fn new(name: ImageName, tag: OciTag) -> Self {
        Self { name, tag }
    }

    /// Returns the tag.
    #[must_use]
    pub const fn tag(&self) -> &OciTag {
        &self.tag
    }
}

impl fmt::Display for TaggedReference {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}", self.name, self.tag)
    }
}

/// A name plus digest reference.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DigestedReference {
    name: ImageName,
    digest: OciDigest,
}

impl DigestedReference {
    /// Creates a digested reference.
    #[must_use]
    pub fn new(name: ImageName, digest: OciDigest) -> Self {
        Self { name, digest }
    }

    /// Returns the digest.
    #[must_use]
    pub const fn digest(&self) -> &OciDigest {
        &self.digest
    }
}

impl fmt::Display for DigestedReference {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}@{}", self.name, self.digest)
    }
}

/// A parsed OCI image reference with optional tag and digest.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ImageReference {
    name: ImageName,
    tag: Option<OciTag>,
    digest: Option<OciDigest>,
    value: String,
}

impl ImageReference {
    /// Parses image reference text.
    pub fn parse(value: impl AsRef<str>) -> Result<Self, ReferenceError> {
        parse_reference(value.as_ref())
    }

    /// Returns the image name.
    #[must_use]
    pub const fn name(&self) -> &ImageName {
        &self.name
    }

    /// Returns the optional registry.
    #[must_use]
    pub const fn registry(&self) -> Option<&RegistryHost> {
        self.name.registry()
    }

    /// Returns the repository.
    #[must_use]
    pub const fn repository(&self) -> &RepositoryName {
        self.name.repository()
    }

    /// Returns the optional tag.
    #[must_use]
    pub const fn tag(&self) -> Option<&OciTag> {
        self.tag.as_ref()
    }

    /// Returns the optional digest.
    #[must_use]
    pub const fn digest(&self) -> Option<&OciDigest> {
        self.digest.as_ref()
    }

    /// Returns the normalized reference text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl fmt::Display for ImageReference {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for ImageReference {
    type Err = ReferenceError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl TryFrom<&str> for ImageReference {
    type Error = ReferenceError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

/// A reference that includes a digest.
pub type CanonicalReference = DigestedReference;

fn parse_reference(value: &str) -> Result<ImageReference, ReferenceError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ReferenceError::Empty);
    }
    if trimmed.chars().any(char::is_whitespace) {
        return Err(ReferenceError::InvalidName);
    }
    let (without_digest, digest) = match trimmed.split_once('@') {
        Some((name, digest)) => (
            name,
            Some(
                digest
                    .parse::<OciDigest>()
                    .map_err(|_| ReferenceError::InvalidDigest)?,
            ),
        ),
        None => (trimmed, None),
    };
    let slash_index = without_digest.rfind('/');
    let colon_index = without_digest.rfind(':');
    let (name_part, tag) = match colon_index {
        Some(index) if slash_index.is_none_or(|slash| index > slash) => {
            let tag = OciTag::new(&without_digest[index + 1..]).map_err(map_tag_error)?;
            (&without_digest[..index], Some(tag))
        },
        _ => (without_digest, None),
    };
    let name = parse_name(name_part)?;
    let value = render_reference(name.as_str(), tag.as_ref(), digest.as_ref());
    Ok(ImageReference {
        name,
        tag,
        digest,
        value,
    })
}

fn parse_name(value: &str) -> Result<ImageName, ReferenceError> {
    let (registry, repository_text) = split_registry(value);
    let registry = registry
        .map(RegistryHost::new)
        .transpose()
        .map_err(|_| ReferenceError::InvalidName)?;
    let repository =
        RepositoryName::new(repository_text).map_err(|_| ReferenceError::InvalidName)?;
    let value = registry.as_ref().map_or_else(
        || repository.to_string(),
        |registry| format!("{registry}/{repository}"),
    );
    Ok(ImageName {
        registry,
        repository,
        value,
    })
}

fn split_registry(value: &str) -> (Option<&str>, &str) {
    let Some((first, rest)) = value.split_once('/') else {
        return (None, value);
    };
    if first.contains('.') || first.contains(':') || first == "localhost" {
        (Some(first), rest)
    } else {
        (None, value)
    }
}

fn has_tag_separator(value: &str) -> bool {
    let slash_index = value.rfind('/');
    value
        .rfind(':')
        .is_some_and(|colon| slash_index.is_none_or(|slash| colon > slash))
}

fn render_reference(name: &str, tag: Option<&OciTag>, digest: Option<&OciDigest>) -> String {
    let mut value = name.to_string();
    if let Some(tag) = tag {
        value.push(':');
        value.push_str(tag.as_str());
    }
    if let Some(digest) = digest {
        value.push('@');
        value.push_str(digest.as_str());
    }
    value
}

fn map_tag_error(_error: OciTagError) -> ReferenceError {
    ReferenceError::InvalidTag
}

#[cfg(test)]
mod tests {
    use super::{ImageName, ImageReference, ReferenceError, TaggedReference};
    use use_oci_tag::OciTag;

    const SHA: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    #[test]
    fn parses_tagged_and_digested_references() -> Result<(), Box<dyn std::error::Error>> {
        let reference: ImageReference =
            format!("ghcr.io/rustuse/app:0.1.0@sha256:{SHA}").parse()?;
        let tagged = TaggedReference::new(ImageName::new("rustuse/app")?, OciTag::new("latest")?);

        assert_eq!(
            reference.registry().map(ToString::to_string),
            Some("ghcr.io".to_string())
        );
        assert_eq!(reference.repository().as_str(), "rustuse/app");
        assert_eq!(reference.tag().map(OciTag::as_str), Some("0.1.0"));
        assert!(reference.digest().is_some());
        assert_eq!(tagged.to_string(), "rustuse/app:latest");
        assert_eq!(ImageName::new("bad:name"), Err(ReferenceError::InvalidName));
        Ok(())
    }
}
