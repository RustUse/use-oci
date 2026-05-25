#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

use core::{fmt, str::FromStr};
use std::error::Error;

/// Errors returned when namespace metadata is invalid.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NamespaceError {
    Empty,
    UnknownKind,
    InvalidPath,
}

impl fmt::Display for NamespaceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("OCI namespace value cannot be empty"),
            Self::UnknownKind => formatter.write_str("unknown OCI namespace kind"),
            Self::InvalidPath => formatter.write_str("invalid OCI namespace path"),
        }
    }
}

impl Error for NamespaceError {}

/// OCI/Linux namespace kind labels.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NamespaceKind {
    Pid,
    Network,
    Mount,
    Ipc,
    Uts,
    User,
    Cgroup,
}

impl NamespaceKind {
    /// Returns the stable namespace label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pid => "pid",
            Self::Network => "network",
            Self::Mount => "mount",
            Self::Ipc => "ipc",
            Self::Uts => "uts",
            Self::User => "user",
            Self::Cgroup => "cgroup",
        }
    }
}

impl fmt::Display for NamespaceKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for NamespaceKind {
    type Err = NamespaceError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "pid" => Ok(Self::Pid),
            "network" | "net" => Ok(Self::Network),
            "mount" | "mnt" => Ok(Self::Mount),
            "ipc" => Ok(Self::Ipc),
            "uts" => Ok(Self::Uts),
            "user" => Ok(Self::User),
            "cgroup" => Ok(Self::Cgroup),
            "" => Err(NamespaceError::Empty),
            _ => Err(NamespaceError::UnknownKind),
        }
    }
}

/// A lexical namespace path marker.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NamespacePath(String);

impl NamespacePath {
    /// Creates a namespace path marker without touching the filesystem.
    pub fn new(value: impl AsRef<str>) -> Result<Self, NamespaceError> {
        let trimmed = value.as_ref().trim();
        if trimmed.is_empty() {
            return Err(NamespaceError::Empty);
        }
        if trimmed
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte == b'\0')
        {
            return Err(NamespaceError::InvalidPath);
        }
        Ok(Self(trimmed.to_string()))
    }

    /// Returns the path text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for NamespacePath {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for NamespacePath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Namespace metadata without platform syscalls.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Namespace {
    kind: NamespaceKind,
    path: Option<NamespacePath>,
}

impl Namespace {
    /// Creates namespace metadata from a kind.
    #[must_use]
    pub const fn new(kind: NamespaceKind) -> Self {
        Self { kind, path: None }
    }

    /// Adds a lexical namespace path.
    #[must_use]
    pub fn with_path(mut self, path: NamespacePath) -> Self {
        self.path = Some(path);
        self
    }

    /// Returns the namespace kind.
    #[must_use]
    pub const fn kind(&self) -> NamespaceKind {
        self.kind
    }

    /// Returns the optional namespace path.
    #[must_use]
    pub const fn path(&self) -> Option<&NamespacePath> {
        self.path.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::{Namespace, NamespaceError, NamespaceKind, NamespacePath};

    #[test]
    fn parses_namespace_kinds() -> Result<(), NamespaceError> {
        assert_eq!("pid".parse::<NamespaceKind>()?, NamespaceKind::Pid);
        assert_eq!("net".parse::<NamespaceKind>()?, NamespaceKind::Network);
        assert_eq!("mnt".parse::<NamespaceKind>()?, NamespaceKind::Mount);
        assert_eq!(
            "bad".parse::<NamespaceKind>(),
            Err(NamespaceError::UnknownKind)
        );
        Ok(())
    }

    #[test]
    fn models_namespace_paths_lexically() -> Result<(), NamespaceError> {
        let namespace =
            Namespace::new(NamespaceKind::Pid).with_path(NamespacePath::new("/proc/self/ns/pid")?);

        assert_eq!(namespace.kind(), NamespaceKind::Pid);
        assert_eq!(
            namespace.path().map(NamespacePath::as_str),
            Some("/proc/self/ns/pid")
        );
        Ok(())
    }
}
