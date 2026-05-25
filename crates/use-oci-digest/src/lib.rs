#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

use core::{fmt, str::FromStr};
use std::error::Error;

/// Errors returned while parsing OCI digest text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OciDigestError {
    Empty,
    MissingSeparator,
    InvalidAlgorithm,
    InvalidValue,
    InvalidSha256Length,
}

impl fmt::Display for OciDigestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("OCI digest cannot be empty"),
            Self::MissingSeparator => formatter.write_str("OCI digest must contain ':'"),
            Self::InvalidAlgorithm => formatter.write_str("invalid OCI digest algorithm"),
            Self::InvalidValue => formatter.write_str("invalid OCI digest value"),
            Self::InvalidSha256Length => {
                formatter.write_str("sha256 digests must be 64 hex characters")
            },
        }
    }
}

impl Error for OciDigestError {}

/// A validated OCI digest algorithm label.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DigestAlgorithm(String);

impl DigestAlgorithm {
    /// Creates a digest algorithm label.
    pub fn new(value: impl AsRef<str>) -> Result<Self, OciDigestError> {
        let normalized = value.as_ref().trim().to_ascii_lowercase();
        if normalized.is_empty() {
            return Err(OciDigestError::Empty);
        }
        if !is_valid_algorithm(&normalized) {
            return Err(OciDigestError::InvalidAlgorithm);
        }
        Ok(Self(normalized))
    }

    /// Returns the conventional `sha256` algorithm label.
    #[must_use]
    pub fn sha256() -> Self {
        Self("sha256".to_string())
    }

    /// Returns the algorithm text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns true when the algorithm is `sha256`.
    #[must_use]
    pub fn is_sha256(&self) -> bool {
        self.as_str() == "sha256"
    }
}

impl AsRef<str> for DigestAlgorithm {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for DigestAlgorithm {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for DigestAlgorithm {
    type Err = OciDigestError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

impl TryFrom<&str> for DigestAlgorithm {
    type Error = OciDigestError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

/// A validated encoded digest value.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DigestValue(String);

impl DigestValue {
    /// Creates an encoded digest value.
    pub fn new(value: impl AsRef<str>) -> Result<Self, OciDigestError> {
        let trimmed = value.as_ref().trim();
        if trimmed.is_empty() {
            return Err(OciDigestError::InvalidValue);
        }
        if !trimmed
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'='))
        {
            return Err(OciDigestError::InvalidValue);
        }
        Ok(Self(trimmed.to_string()))
    }

    /// Creates a sha256 hex digest value.
    pub fn sha256_hex(value: impl AsRef<str>) -> Result<Self, OciDigestError> {
        let trimmed = value.as_ref().trim();
        if trimmed.len() != 64 {
            return Err(OciDigestError::InvalidSha256Length);
        }
        if !trimmed.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(OciDigestError::InvalidValue);
        }
        Ok(Self(trimmed.to_ascii_lowercase()))
    }

    /// Returns the encoded digest text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for DigestValue {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for DigestValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// A parsed OCI digest such as `sha256:<hex>`.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OciDigest {
    value: String,
    algorithm: DigestAlgorithm,
    encoded: DigestValue,
}

impl OciDigest {
    /// Creates an OCI digest from typed parts.
    pub fn new(algorithm: DigestAlgorithm, encoded: DigestValue) -> Result<Self, OciDigestError> {
        if algorithm.is_sha256() && encoded.as_str().len() != 64 {
            return Err(OciDigestError::InvalidSha256Length);
        }
        let value = format!("{algorithm}:{encoded}");
        Ok(Self {
            value,
            algorithm,
            encoded,
        })
    }

    /// Parses digest text.
    pub fn parse(value: impl AsRef<str>) -> Result<Self, OciDigestError> {
        let trimmed = value.as_ref().trim();
        if trimmed.is_empty() {
            return Err(OciDigestError::Empty);
        }
        let Some((algorithm, encoded)) = trimmed.split_once(':') else {
            return Err(OciDigestError::MissingSeparator);
        };
        let algorithm = DigestAlgorithm::new(algorithm)?;
        let encoded = if algorithm.is_sha256() {
            DigestValue::sha256_hex(encoded)?
        } else {
            DigestValue::new(encoded)?
        };
        Self::new(algorithm, encoded)
    }

    /// Returns the full digest text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }

    /// Returns the digest algorithm.
    #[must_use]
    pub const fn algorithm(&self) -> &DigestAlgorithm {
        &self.algorithm
    }

    /// Returns the encoded digest value.
    #[must_use]
    pub const fn encoded(&self) -> &DigestValue {
        &self.encoded
    }
}

impl AsRef<str> for OciDigest {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for OciDigest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for OciDigest {
    type Err = OciDigestError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl TryFrom<&str> for OciDigest {
    type Error = OciDigestError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

fn is_valid_algorithm(value: &str) -> bool {
    value
        .bytes()
        .next()
        .is_some_and(|byte| byte.is_ascii_alphanumeric())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'-' | b'+'))
}

#[cfg(test)]
mod tests {
    use super::{DigestAlgorithm, DigestValue, OciDigest, OciDigestError};

    const SHA: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    #[test]
    fn parses_sha256_digest() -> Result<(), Box<dyn std::error::Error>> {
        let digest: OciDigest = format!("sha256:{SHA}").parse()?;

        assert_eq!(digest.algorithm().as_str(), "sha256");
        assert_eq!(digest.encoded().as_str(), SHA);
        assert_eq!(digest.to_string(), format!("sha256:{SHA}"));
        Ok(())
    }

    #[test]
    fn validates_digest_parts() -> Result<(), Box<dyn std::error::Error>> {
        let digest = OciDigest::new(DigestAlgorithm::sha256(), DigestValue::sha256_hex(SHA)?)?;

        assert_eq!(digest.as_str(), format!("sha256:{SHA}"));
        assert_eq!(
            OciDigest::parse("sha256:abc"),
            Err(OciDigestError::InvalidSha256Length)
        );
        assert_eq!(
            DigestAlgorithm::new("bad algorithm"),
            Err(OciDigestError::InvalidAlgorithm)
        );
        Ok(())
    }
}
