//! # config-get
//!
//! Cross-platform configuration file locator and reader for Rust.
//!
//! `config-get` automatically discovers and reads configuration files from
//! standard OS-specific locations, supporting `.env`, `.ini`, `.toml`, `.json`,
//! `.yml`, and `.yaml` formats — no manual path wrangling required.
//!
//! ## Quick start
//!
//! ```rust
//! # use config_get::ConfigGet;
//! # fn main() -> config_get::Result<()> {
//! # /*
//! let cfg = ConfigGet::builder("myapp")
//!     .config_dir("myapp")
//!     .build()?;
//!
//! let host = cfg.get("DB_HOST").unwrap_or("localhost");
//! let port: u16 = cfg.parse("DB_PORT")?;
//! let debug = cfg.get_in_or("server", "debug", "false");
//! # */
//! # Ok(())
//! # }
//! ```
//!
//! ## Features
//!
//! | Feature | Enables | Default |
//! |---------|---------|---------|
//! | `dotenv` | `.env` via `dotenvy` | ✓ |
//! | `ini`   | `.ini` via `rust-ini` | ✓ |
//! | `toml`  | `.toml` via `toml`   | ✓ |
//! | `yaml`  | `.yaml`/`.yml` via `serde_yaml` | ✓ |
//! | `cli`   | `config-get` binary via `clap` | ✗ |
//! | `all`   | All of the above | ✗ |

#![cfg_attr(docsrs, feature(doc_cfg))]
#![forbid(unsafe_code)]
#![warn(
    missing_docs,
    clippy::pedantic,
    clippy::unwrap_used,
    clippy::expect_used
)]
#![allow(clippy::module_name_repetitions)]

/// Core [`ConfigGet`] type and its builder.
pub mod config_get;

/// Platform-aware directory search logic.
pub mod discovery;

/// Error types for this crate.
pub mod error;

/// Supported file format detection.
pub mod format;

/// Format-specific parsers.
pub mod parsers;

/// Internal key/value storage.
pub mod value;

// Top-level re-exports for ergonomic use.
pub use config_get::ConfigGet;
pub use config_get::ConfigGetBuilder;
pub use error::{ConfigError, Result};
pub use format::Format;
pub use value::ConfigMap;

/// Convenience function — returns the path of the first matching config file,
/// or `None`.
///
/// ```rust
/// use config_get::get_config_file;
/// let path = get_config_file("myapp", "myapp");
/// // path.is_some() if a config file was found
/// ```
#[must_use]
pub fn get_config_file(stem: &str, config_dir: &str) -> Option<std::path::PathBuf> {
    ConfigGet::search_paths(stem, config_dir)
        .into_iter()
        .find(|p| p.is_file())
}
