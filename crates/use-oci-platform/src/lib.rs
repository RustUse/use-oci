#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

use core::{fmt, str::FromStr};
use std::error::Error;

/// Errors returned while parsing OCI platform values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlatformError {
    Empty,
    InvalidPart,
    InvalidPlatform,
}

impl fmt::Display for PlatformError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("OCI platform value cannot be empty"),
            Self::InvalidPart => formatter.write_str("invalid OCI platform part"),
            Self::InvalidPlatform => formatter.write_str("invalid OCI platform string"),
        }
    }
}

impl Error for PlatformError {}

/// OCI operating system labels.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum OciOs {
    Linux,
    Windows,
    Darwin,
    FreeBsd,
    Wasm,
    Unknown,
    Custom(String),
}

impl OciOs {
    /// Returns the stable OS label.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Linux => "linux",
            Self::Windows => "windows",
            Self::Darwin => "darwin",
            Self::FreeBsd => "freebsd",
            Self::Wasm => "wasm",
            Self::Unknown => "unknown",
            Self::Custom(value) => value,
        }
    }
}

impl fmt::Display for OciOs {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for OciOs {
    type Err = PlatformError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let normalized = normalize_part(value)?;
        match normalized.as_str() {
            "linux" => Ok(Self::Linux),
            "windows" => Ok(Self::Windows),
            "darwin" | "macos" => Ok(Self::Darwin),
            "freebsd" => Ok(Self::FreeBsd),
            "wasm" | "wasi" => Ok(Self::Wasm),
            "unknown" => Ok(Self::Unknown),
            _ => Ok(Self::Custom(normalized)),
        }
    }
}

/// OCI architecture labels.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum OciArchitecture {
    Amd64,
    Arm64,
    Arm,
    I386,
    Ppc64le,
    Riscv64,
    S390x,
    Wasm,
    Unknown,
    Custom(String),
}

impl OciArchitecture {
    /// Returns the stable architecture label.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Amd64 => "amd64",
            Self::Arm64 => "arm64",
            Self::Arm => "arm",
            Self::I386 => "386",
            Self::Ppc64le => "ppc64le",
            Self::Riscv64 => "riscv64",
            Self::S390x => "s390x",
            Self::Wasm => "wasm",
            Self::Unknown => "unknown",
            Self::Custom(value) => value,
        }
    }
}

impl fmt::Display for OciArchitecture {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for OciArchitecture {
    type Err = PlatformError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let normalized = normalize_part(value)?;
        match normalized.replace(['_', '-', ' '], "").as_str() {
            "amd64" | "x8664" | "x64" => Ok(Self::Amd64),
            "arm64" | "aarch64" => Ok(Self::Arm64),
            "arm" => Ok(Self::Arm),
            "386" | "i386" | "i686" => Ok(Self::I386),
            "ppc64le" => Ok(Self::Ppc64le),
            "riscv64" => Ok(Self::Riscv64),
            "s390x" => Ok(Self::S390x),
            "wasm" | "wasm32" | "wasm64" => Ok(Self::Wasm),
            "unknown" => Ok(Self::Unknown),
            _ => Ok(Self::Custom(normalized)),
        }
    }
}

macro_rules! text_part {
    ($name:ident) => {
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(String);

        impl $name {
            /// Creates a validated platform text part.
            pub fn new(value: impl AsRef<str>) -> Result<Self, PlatformError> {
                normalize_part(value.as_ref()).map(Self)
            }

            /// Returns the text part.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(self.as_str())
            }
        }
    };
}

text_part!(PlatformVariant);
text_part!(OsVersion);
text_part!(OsFeature);

/// OCI platform metadata.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OciPlatform {
    os: OciOs,
    architecture: OciArchitecture,
    variant: Option<PlatformVariant>,
    os_version: Option<OsVersion>,
    os_features: Vec<OsFeature>,
}

impl OciPlatform {
    /// Creates platform metadata from OS and architecture labels.
    #[must_use]
    pub fn new(os: OciOs, architecture: OciArchitecture) -> Self {
        Self {
            os,
            architecture,
            variant: None,
            os_version: None,
            os_features: Vec::new(),
        }
    }

    /// Adds an architecture variant.
    pub fn with_variant(mut self, variant: impl AsRef<str>) -> Result<Self, PlatformError> {
        self.variant = Some(PlatformVariant::new(variant)?);
        Ok(self)
    }

    /// Adds an OS version label.
    pub fn with_os_version(mut self, version: impl AsRef<str>) -> Result<Self, PlatformError> {
        self.os_version = Some(OsVersion::new(version)?);
        Ok(self)
    }

    /// Adds an OS feature label.
    pub fn with_os_feature(mut self, feature: impl AsRef<str>) -> Result<Self, PlatformError> {
        self.os_features.push(OsFeature::new(feature)?);
        Ok(self)
    }

    /// Returns the OS label.
    #[must_use]
    pub const fn os(&self) -> &OciOs {
        &self.os
    }

    /// Returns the architecture label.
    #[must_use]
    pub const fn architecture(&self) -> &OciArchitecture {
        &self.architecture
    }

    /// Returns the optional variant.
    #[must_use]
    pub const fn variant(&self) -> Option<&PlatformVariant> {
        self.variant.as_ref()
    }

    /// Returns OS features.
    #[must_use]
    pub fn os_features(&self) -> &[OsFeature] {
        &self.os_features
    }
}

impl fmt::Display for OciPlatform {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}/{}", self.os, self.architecture)?;
        if let Some(variant) = &self.variant {
            write!(formatter, "/{variant}")?;
        }
        Ok(())
    }
}

impl FromStr for OciPlatform {
    type Err = PlatformError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let parts = value.trim().split('/').collect::<Vec<_>>();
        match parts.as_slice() {
            [os, architecture] => Ok(Self::new(os.parse()?, architecture.parse()?)),
            [os, architecture, variant] => {
                Self::new(os.parse()?, architecture.parse()?).with_variant(variant)
            },
            _ => Err(PlatformError::InvalidPlatform),
        }
    }
}

fn normalize_part(value: &str) -> Result<String, PlatformError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(PlatformError::Empty);
    }
    if trimmed
        .bytes()
        .any(|byte| byte.is_ascii_control() || byte.is_ascii_whitespace() || byte == b'/')
    {
        return Err(PlatformError::InvalidPart);
    }
    Ok(trimmed.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::{OciArchitecture, OciOs, OciPlatform, PlatformError};

    #[test]
    fn parses_and_renders_platforms() -> Result<(), Box<dyn std::error::Error>> {
        let platform: OciPlatform = "linux/arm64/v8".parse()?;

        assert_eq!(platform.os(), &OciOs::Linux);
        assert_eq!(platform.architecture(), &OciArchitecture::Arm64);
        assert_eq!(platform.to_string(), "linux/arm64/v8");
        assert_eq!(
            "linux".parse::<OciPlatform>(),
            Err(PlatformError::InvalidPlatform)
        );
        Ok(())
    }

    #[test]
    fn accepts_common_architecture_aliases() -> Result<(), PlatformError> {
        assert_eq!("x86_64".parse::<OciArchitecture>()?, OciArchitecture::Amd64);
        assert_eq!(
            "aarch64".parse::<OciArchitecture>()?,
            OciArchitecture::Arm64
        );
        assert_eq!("i686".parse::<OciArchitecture>()?, OciArchitecture::I386);
        Ok(())
    }
}
