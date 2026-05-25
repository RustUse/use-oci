#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

use core::{fmt, str::FromStr};
use std::error::Error;

/// Errors returned when hook metadata is invalid.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HookError {
    Empty,
    UnknownKind,
    InvalidEnv,
    InvalidTimeout,
}

impl fmt::Display for HookError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("OCI hook value cannot be empty"),
            Self::UnknownKind => formatter.write_str("unknown OCI hook kind"),
            Self::InvalidEnv => formatter.write_str("invalid OCI hook environment entry"),
            Self::InvalidTimeout => formatter.write_str("invalid OCI hook timeout"),
        }
    }
}

impl Error for HookError {}

/// OCI runtime hook lifecycle phase.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum HookKind {
    Prestart,
    CreateRuntime,
    CreateContainer,
    StartContainer,
    Poststart,
    Poststop,
}

impl HookKind {
    /// Returns the OCI hook kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Prestart => "prestart",
            Self::CreateRuntime => "createRuntime",
            Self::CreateContainer => "createContainer",
            Self::StartContainer => "startContainer",
            Self::Poststart => "poststart",
            Self::Poststop => "poststop",
        }
    }
}

impl fmt::Display for HookKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for HookKind {
    type Err = HookError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let key = value.trim().replace(['-', '_'], "").to_ascii_lowercase();
        match key.as_str() {
            "prestart" => Ok(Self::Prestart),
            "createruntime" => Ok(Self::CreateRuntime),
            "createcontainer" => Ok(Self::CreateContainer),
            "startcontainer" => Ok(Self::StartContainer),
            "poststart" => Ok(Self::Poststart),
            "poststop" => Ok(Self::Poststop),
            "" => Err(HookError::Empty),
            _ => Err(HookError::UnknownKind),
        }
    }
}

macro_rules! text_part {
    ($name:ident) => {
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(String);

        impl $name {
            /// Creates a non-empty hook text value.
            pub fn new(value: impl AsRef<str>) -> Result<Self, HookError> {
                let trimmed = value.as_ref().trim();
                if trimmed.is_empty() {
                    return Err(HookError::Empty);
                }
                if trimmed.contains('\0') {
                    return Err(HookError::InvalidEnv);
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

text_part!(HookPath);
text_part!(HookArg);
text_part!(HookEnv);

/// Hook timeout in seconds.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct HookTimeoutSeconds(u32);

impl HookTimeoutSeconds {
    /// Creates a positive timeout value.
    pub const fn new(value: u32) -> Result<Self, HookError> {
        if value == 0 {
            Err(HookError::InvalidTimeout)
        } else {
            Ok(Self(value))
        }
    }

    /// Returns the timeout seconds.
    #[must_use]
    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

/// OCI hook metadata. This type does not execute hooks.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OciHook {
    kind: HookKind,
    path: HookPath,
    args: Vec<HookArg>,
    env: Vec<HookEnv>,
    timeout: Option<HookTimeoutSeconds>,
}

impl OciHook {
    /// Creates hook metadata.
    #[must_use]
    pub fn new(kind: HookKind, path: HookPath) -> Self {
        Self {
            kind,
            path,
            args: Vec::new(),
            env: Vec::new(),
            timeout: None,
        }
    }

    /// Adds an argument.
    #[must_use]
    pub fn with_arg(mut self, arg: HookArg) -> Self {
        self.args.push(arg);
        self
    }

    /// Adds an environment entry.
    #[must_use]
    pub fn with_env(mut self, env: HookEnv) -> Self {
        self.env.push(env);
        self
    }

    /// Adds a timeout.
    #[must_use]
    pub const fn with_timeout(mut self, timeout: HookTimeoutSeconds) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Returns the hook kind.
    #[must_use]
    pub const fn kind(&self) -> HookKind {
        self.kind
    }

    /// Returns the hook path.
    #[must_use]
    pub const fn path(&self) -> &HookPath {
        &self.path
    }

    /// Returns hook args.
    #[must_use]
    pub fn args(&self) -> &[HookArg] {
        &self.args
    }

    /// Returns hook environment entries.
    #[must_use]
    pub fn env(&self) -> &[HookEnv] {
        &self.env
    }
}

#[cfg(test)]
mod tests {
    use super::{HookArg, HookKind, HookPath, HookTimeoutSeconds, OciHook};

    #[test]
    fn models_hook_metadata_without_execution() -> Result<(), Box<dyn std::error::Error>> {
        let hook = OciHook::new(HookKind::Prestart, HookPath::new("/bin/check")?)
            .with_arg(HookArg::new("--dry-run")?)
            .with_timeout(HookTimeoutSeconds::new(5)?);

        assert_eq!(hook.kind().to_string(), "prestart");
        assert_eq!(hook.path().as_str(), "/bin/check");
        assert_eq!(hook.args().len(), 1);
        assert_eq!(
            "create-runtime".parse::<HookKind>()?,
            HookKind::CreateRuntime
        );
        Ok(())
    }
}
