#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

use core::{fmt, str::FromStr};
use std::error::Error;

use use_oci_digest::OciDigest;
use use_oci_tag::OciTag;

/// Errors returned when distribution identifiers are invalid.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DistributionError {
    Empty,
    InvalidHost,
    InvalidRepository,
}

impl fmt::Display for DistributionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("OCI distribution value cannot be empty"),
            Self::InvalidHost => formatter.write_str("invalid OCI registry host"),
            Self::InvalidRepository => formatter.write_str("invalid OCI repository name"),
        }
    }
}

impl Error for DistributionError {}

/// A registry host label.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RegistryHost(String);

impl RegistryHost {
    /// Creates a registry host label.
    pub fn new(value: impl AsRef<str>) -> Result<Self, DistributionError> {
        let trimmed = value.as_ref().trim();
        if trimmed.is_empty() {
            return Err(DistributionError::Empty);
        }
        if trimmed.contains('/') || trimmed.chars().any(char::is_whitespace) {
            return Err(DistributionError::InvalidHost);
        }
        Ok(Self(trimmed.to_string()))
    }

    /// Returns the host text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RegistryHost {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// A slash-separated repository namespace.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Namespace(String);

impl Namespace {
    /// Creates a namespace label.
    pub fn new(value: impl AsRef<str>) -> Result<Self, DistributionError> {
        let repository = RepositoryName::new(value)?;
        Ok(Self(repository.into_string()))
    }

    /// Returns the namespace text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A validated repository name.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RepositoryName(String);

impl RepositoryName {
    /// Creates a repository name.
    pub fn new(value: impl AsRef<str>) -> Result<Self, DistributionError> {
        let trimmed = value.as_ref().trim();
        if trimmed.is_empty() {
            return Err(DistributionError::Empty);
        }
        if trimmed
            .split('/')
            .any(|component| !is_valid_component(component))
        {
            return Err(DistributionError::InvalidRepository);
        }
        Ok(Self(trimmed.to_string()))
    }

    /// Returns the repository text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the repository and returns the owned string.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for RepositoryName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for RepositoryName {
    type Err = DistributionError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

/// A blob reference by digest.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BlobReference {
    repository: RepositoryName,
    digest: OciDigest,
}

impl BlobReference {
    /// Creates a blob reference.
    #[must_use]
    pub const fn new(repository: RepositoryName, digest: OciDigest) -> Self {
        Self { repository, digest }
    }

    /// Returns the repository.
    #[must_use]
    pub const fn repository(&self) -> &RepositoryName {
        &self.repository
    }

    /// Returns the digest.
    #[must_use]
    pub const fn digest(&self) -> &OciDigest {
        &self.digest
    }
}

/// A manifest reference by tag or digest.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ManifestReference {
    Tag(OciTag),
    Digest(OciDigest),
}

impl fmt::Display for ManifestReference {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tag(tag) => tag.fmt(formatter),
            Self::Digest(digest) => digest.fmt(formatter),
        }
    }
}

/// A tag reference.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TagReference {
    repository: RepositoryName,
    tag: OciTag,
}

impl TagReference {
    /// Creates a tag reference.
    #[must_use]
    pub const fn new(repository: RepositoryName, tag: OciTag) -> Self {
        Self { repository, tag }
    }

    /// Returns the tag.
    #[must_use]
    pub const fn tag(&self) -> &OciTag {
        &self.tag
    }
}

/// Pull or push route action metadata.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RouteAction {
    Pull,
    Push,
}

/// A distribution route path. This type performs no HTTP calls.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DistributionRoute {
    action: RouteAction,
    path: String,
}

impl DistributionRoute {
    /// Creates a blob route path.
    #[must_use]
    pub fn blob(repository: &RepositoryName, digest: &OciDigest) -> Self {
        Self {
            action: RouteAction::Pull,
            path: format!("/v2/{repository}/blobs/{digest}"),
        }
    }

    /// Creates a manifest route path.
    #[must_use]
    pub fn manifest(repository: &RepositoryName, reference: &ManifestReference) -> Self {
        Self {
            action: RouteAction::Pull,
            path: format!("/v2/{repository}/manifests/{reference}"),
        }
    }

    /// Marks the route as push metadata.
    #[must_use]
    pub const fn for_push(mut self) -> Self {
        self.action = RouteAction::Push;
        self
    }

    /// Returns the route action.
    #[must_use]
    pub const fn action(&self) -> RouteAction {
        self.action
    }

    /// Returns the route path.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }
}

fn is_valid_component(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_alphanumeric())
        && value
            .bytes()
            .last()
            .is_some_and(|byte| byte.is_ascii_alphanumeric())
}

#[cfg(test)]
mod tests {
    use super::{
        DistributionError, DistributionRoute, ManifestReference, RepositoryName, RouteAction,
    };
    use use_oci_digest::OciDigest;
    use use_oci_tag::OciTag;

    const SHA: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    #[test]
    fn builds_distribution_paths_without_http() -> Result<(), Box<dyn std::error::Error>> {
        let repository = RepositoryName::new("rustuse/app")?;
        let digest: OciDigest = format!("sha256:{SHA}").parse()?;
        let route = DistributionRoute::manifest(&repository, &ManifestReference::Digest(digest));
        let tag_route = DistributionRoute::manifest(
            &repository,
            &ManifestReference::Tag(OciTag::new("latest")?),
        )
        .for_push();

        assert_eq!(
            route.path(),
            format!("/v2/rustuse/app/manifests/sha256:{SHA}")
        );
        assert_eq!(tag_route.action(), RouteAction::Push);
        assert_eq!(
            RepositoryName::new("Bad/Name"),
            Err(DistributionError::InvalidRepository)
        );
        Ok(())
    }
}
