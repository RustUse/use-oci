#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

use core::fmt;
use std::error::Error;

use use_oci_annotation::Annotation;
use use_oci_platform::{OciArchitecture, OciOs};

/// Errors returned when config metadata is invalid.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfigError {
    Empty,
    InvalidEnv,
    InvalidPort,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("OCI config value cannot be empty"),
            Self::InvalidEnv => formatter.write_str("invalid OCI config environment entry"),
            Self::InvalidPort => formatter.write_str("invalid OCI exposed port"),
        }
    }
}

impl Error for ConfigError {}

macro_rules! text_value {
    ($name:ident) => {
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(String);

        impl $name {
            /// Creates a non-empty config text value.
            pub fn new(value: impl AsRef<str>) -> Result<Self, ConfigError> {
                let trimmed = value.as_ref().trim();
                if trimmed.is_empty() {
                    return Err(ConfigError::Empty);
                }
                if trimmed.contains('\0') {
                    return Err(ConfigError::InvalidEnv);
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

text_value!(ConfigUser);
text_value!(Entrypoint);
text_value!(Command);
text_value!(WorkingDir);
text_value!(VolumePath);
text_value!(StopSignal);

/// An environment variable entry.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EnvVar {
    key: String,
    value: String,
}

impl EnvVar {
    /// Creates an environment variable entry.
    pub fn new(key: impl AsRef<str>, value: impl AsRef<str>) -> Result<Self, ConfigError> {
        let key = key.as_ref().trim();
        if key.is_empty()
            || key.contains('=')
            || key
                .bytes()
                .any(|byte| byte.is_ascii_control() || byte.is_ascii_whitespace())
        {
            return Err(ConfigError::InvalidEnv);
        }
        let value = value.as_ref();
        if value.contains('\0') {
            return Err(ConfigError::InvalidEnv);
        }
        Ok(Self {
            key: key.to_string(),
            value: value.to_string(),
        })
    }

    /// Returns the key.
    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Returns the value.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl fmt::Display for EnvVar {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}={}", self.key, self.value)
    }
}

/// Exposed port metadata.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ExposedPort(String);

impl ExposedPort {
    /// Creates exposed port metadata such as `8080/tcp`.
    pub fn new(value: impl AsRef<str>) -> Result<Self, ConfigError> {
        let trimmed = value.as_ref().trim().to_ascii_lowercase();
        let Some((number, protocol)) = trimmed.split_once('/') else {
            return Err(ConfigError::InvalidPort);
        };
        if number.parse::<u16>().is_err() || !matches!(protocol, "tcp" | "udp" | "sctp") {
            return Err(ConfigError::InvalidPort);
        }
        Ok(Self(trimmed))
    }

    /// Returns the port text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// OCI image config primitives.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OciImageConfig {
    architecture: OciArchitecture,
    os: OciOs,
    user: Option<ConfigUser>,
    env: Vec<EnvVar>,
    entrypoint: Vec<Entrypoint>,
    command: Vec<Command>,
    working_dir: Option<WorkingDir>,
    exposed_ports: Vec<ExposedPort>,
    labels: Vec<Annotation>,
    volumes: Vec<VolumePath>,
    stop_signal: Option<StopSignal>,
    annotations: Vec<Annotation>,
}

impl OciImageConfig {
    /// Creates image config metadata.
    #[must_use]
    pub fn new(architecture: OciArchitecture, os: OciOs) -> Self {
        Self {
            architecture,
            os,
            user: None,
            env: Vec::new(),
            entrypoint: Vec::new(),
            command: Vec::new(),
            working_dir: None,
            exposed_ports: Vec::new(),
            labels: Vec::new(),
            volumes: Vec::new(),
            stop_signal: None,
            annotations: Vec::new(),
        }
    }

    /// Adds a user label.
    #[must_use]
    pub fn with_user(mut self, user: ConfigUser) -> Self {
        self.user = Some(user);
        self
    }

    /// Adds an environment variable.
    #[must_use]
    pub fn with_env(mut self, env: EnvVar) -> Self {
        self.env.push(env);
        self
    }

    /// Adds an entrypoint part.
    #[must_use]
    pub fn with_entrypoint(mut self, entrypoint: Entrypoint) -> Self {
        self.entrypoint.push(entrypoint);
        self
    }

    /// Adds a command part.
    #[must_use]
    pub fn with_command(mut self, command: Command) -> Self {
        self.command.push(command);
        self
    }

    /// Adds an exposed port.
    #[must_use]
    pub fn with_exposed_port(mut self, port: ExposedPort) -> Self {
        self.exposed_ports.push(port);
        self
    }

    /// Adds an annotation.
    #[must_use]
    pub fn with_annotation(mut self, annotation: Annotation) -> Self {
        self.annotations.push(annotation);
        self
    }

    /// Returns the architecture label.
    #[must_use]
    pub const fn architecture(&self) -> &OciArchitecture {
        &self.architecture
    }

    /// Returns the OS label.
    #[must_use]
    pub const fn os(&self) -> &OciOs {
        &self.os
    }

    /// Returns environment entries.
    #[must_use]
    pub fn env(&self) -> &[EnvVar] {
        &self.env
    }

    /// Returns annotations.
    #[must_use]
    pub fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }
}

#[cfg(test)]
mod tests {
    use super::{ConfigError, EnvVar, ExposedPort, OciImageConfig};
    use use_oci_platform::{OciArchitecture, OciOs};

    #[test]
    fn models_image_config_metadata() -> Result<(), Box<dyn std::error::Error>> {
        let config = OciImageConfig::new(OciArchitecture::Amd64, OciOs::Linux)
            .with_env(EnvVar::new("RUST_LOG", "info")?)
            .with_exposed_port(ExposedPort::new("8080/tcp")?);

        assert_eq!(config.architecture(), &OciArchitecture::Amd64);
        assert_eq!(config.env()[0].to_string(), "RUST_LOG=info");
        assert_eq!(
            EnvVar::new("BAD KEY", "value"),
            Err(ConfigError::InvalidEnv)
        );
        assert_eq!(ExposedPort::new("tcp/8080"), Err(ConfigError::InvalidPort));
        Ok(())
    }
}
