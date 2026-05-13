use thiserror::Error;

/// All errors that can occur in `config-get`.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// No configuration file was found in any of the searched paths.
    #[error("no configuration file found for '{0}'")]
    NotFound(String),

    /// The file could not be read from disk.
    #[error("I/O error reading '{path}': {source}")]
    Io {
        /// Path of the file that could not be read.
        path: String,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// The file content could not be parsed.
    #[error("parse error in '{path}': {message}")]
    Parse {
        /// Path of the file that failed to parse.
        path: String,
        /// Human-readable description of the parse failure.
        message: String,
    },

    /// A required key was not found in the config.
    #[error("key not found: '{0}'")]
    KeyNotFound(String),

    /// The requested section was not found.
    #[error("section not found: '{0}'")]
    SectionNotFound(String),

    /// A feature required to handle this format is not enabled.
    #[error("feature '{feature}' is not enabled; recompile with `--features {feature}`")]
    FeatureNotEnabled {
        /// The Cargo feature that must be enabled.
        feature: &'static str,
    },

    /// Generic / unexpected error.
    #[error("{0}")]
    Other(String),
}

/// Convenience alias used throughout the crate.
pub type Result<T, E = ConfigError> = std::result::Result<T, E>;
