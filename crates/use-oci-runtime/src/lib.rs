#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

use core::fmt;
use std::error::Error;

use use_oci_hook::OciHook;
use use_oci_namespace::NamespaceKind;

/// Errors returned when runtime metadata is invalid.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeError {
    Empty,
    InvalidMount,
    InvalidResource,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("OCI runtime value cannot be empty"),
            Self::InvalidMount => formatter.write_str("invalid OCI mount metadata"),
            Self::InvalidResource => formatter.write_str("invalid OCI resource metadata"),
        }
    }
}

impl Error for RuntimeError {}

macro_rules! text_value {
    ($name:ident) => {
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(String);

        impl $name {
            /// Creates a non-empty runtime text value.
            pub fn new(value: impl AsRef<str>) -> Result<Self, RuntimeError> {
                let trimmed = value.as_ref().trim();
                if trimmed.is_empty() {
                    return Err(RuntimeError::Empty);
                }
                if trimmed.contains('\0') {
                    return Err(RuntimeError::InvalidMount);
                }
                Ok(Self(trimmed.to_string()))
            }

            /// Returns the text value.
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

text_value!(ProcessArg);
text_value!(RuntimeEnv);
text_value!(Cwd);
text_value!(Capability);
text_value!(RootFilesystem);

/// OCI mount kind metadata.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MountKind {
    Bind,
    Tmpfs,
    Proc,
    Sysfs,
    Cgroup,
    Custom,
}

/// OCI mount metadata. This type does not mount filesystems.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Mount {
    kind: MountKind,
    source: String,
    destination: String,
    options: Vec<String>,
}

impl Mount {
    /// Creates mount metadata.
    pub fn new(
        kind: MountKind,
        source: impl AsRef<str>,
        destination: impl AsRef<str>,
    ) -> Result<Self, RuntimeError> {
        let source = non_empty(source.as_ref(), RuntimeError::InvalidMount)?;
        let destination = non_empty(destination.as_ref(), RuntimeError::InvalidMount)?;
        Ok(Self {
            kind,
            source: source.to_string(),
            destination: destination.to_string(),
            options: Vec::new(),
        })
    }

    /// Adds a mount option.
    #[must_use]
    pub fn with_option(mut self, option: impl Into<String>) -> Self {
        self.options.push(option.into());
        self
    }

    /// Returns the mount kind.
    #[must_use]
    pub const fn kind(&self) -> MountKind {
        self.kind
    }

    /// Returns the mount destination.
    #[must_use]
    pub fn destination(&self) -> &str {
        &self.destination
    }
}

/// Resource limit metadata.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResourceLimit {
    name: String,
    value: u64,
}

impl ResourceLimit {
    /// Creates resource limit metadata.
    pub fn new(name: impl AsRef<str>, value: u64) -> Result<Self, RuntimeError> {
        let name = non_empty(name.as_ref(), RuntimeError::InvalidResource)?;
        Ok(Self {
            name: name.to_string(),
            value,
        })
    }

    /// Returns the resource name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the resource value.
    #[must_use]
    pub const fn value(&self) -> u64 {
        self.value
    }
}

/// OCI runtime metadata. This type does not execute a runtime.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeSpec {
    root: RootFilesystem,
    args: Vec<ProcessArg>,
    env: Vec<RuntimeEnv>,
    cwd: Option<Cwd>,
    mounts: Vec<Mount>,
    hooks: Vec<OciHook>,
    namespaces: Vec<NamespaceKind>,
    capabilities: Vec<Capability>,
    resources: Vec<ResourceLimit>,
}

impl RuntimeSpec {
    /// Creates runtime metadata from a root filesystem marker.
    #[must_use]
    pub fn new(root: RootFilesystem) -> Self {
        Self {
            root,
            args: Vec::new(),
            env: Vec::new(),
            cwd: None,
            mounts: Vec::new(),
            hooks: Vec::new(),
            namespaces: Vec::new(),
            capabilities: Vec::new(),
            resources: Vec::new(),
        }
    }

    /// Adds a process argument.
    #[must_use]
    pub fn with_arg(mut self, arg: ProcessArg) -> Self {
        self.args.push(arg);
        self
    }

    /// Adds a mount.
    #[must_use]
    pub fn with_mount(mut self, mount: Mount) -> Self {
        self.mounts.push(mount);
        self
    }

    /// Adds a hook.
    #[must_use]
    pub fn with_hook(mut self, hook: OciHook) -> Self {
        self.hooks.push(hook);
        self
    }

    /// Adds a namespace kind.
    #[must_use]
    pub fn with_namespace(mut self, namespace: NamespaceKind) -> Self {
        self.namespaces.push(namespace);
        self
    }

    /// Returns the root filesystem marker.
    #[must_use]
    pub const fn root(&self) -> &RootFilesystem {
        &self.root
    }

    /// Returns namespaces.
    #[must_use]
    pub fn namespaces(&self) -> &[NamespaceKind] {
        &self.namespaces
    }

    /// Returns hooks.
    #[must_use]
    pub fn hooks(&self) -> &[OciHook] {
        &self.hooks
    }
}

fn non_empty(value: &str, error: RuntimeError) -> Result<&str, RuntimeError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.contains('\0') {
        Err(error)
    } else {
        Ok(trimmed)
    }
}

#[cfg(test)]
mod tests {
    use super::{Mount, MountKind, ProcessArg, RootFilesystem, RuntimeSpec};
    use use_oci_hook::{HookKind, HookPath, OciHook};
    use use_oci_namespace::NamespaceKind;

    #[test]
    fn models_runtime_metadata_without_execution() -> Result<(), Box<dyn std::error::Error>> {
        let hook = OciHook::new(HookKind::Prestart, HookPath::new("/bin/check")?);
        let spec = RuntimeSpec::new(RootFilesystem::new("rootfs")?)
            .with_arg(ProcessArg::new("/bin/sh")?)
            .with_mount(Mount::new(MountKind::Bind, "/host", "/container")?)
            .with_namespace(NamespaceKind::Pid)
            .with_hook(hook);

        assert_eq!(spec.root().as_str(), "rootfs");
        assert_eq!(spec.namespaces(), &[NamespaceKind::Pid]);
        assert_eq!(spec.hooks().len(), 1);
        Ok(())
    }
}
