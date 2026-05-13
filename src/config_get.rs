use std::path::{Path, PathBuf};

use crate::{
    discovery::search_dirs,
    error::{ConfigError, Result},
    format::Format,
    parsers,
    value::ConfigMap,
};

/// Cross-platform configuration file reader.
///
/// `ConfigGet` auto-discovers configuration files from OS-standard locations
/// and exposes a unified API for reading values regardless of the underlying
/// format.
///
/// # Quick start
///
/// ```rust
/// # use config_get::ConfigGet;
/// # fn main() -> config_get::Result<()> {
/// # /*
/// let cfg = ConfigGet::builder("myapp")
///     .config_dir("myapp")
///     .build()?;
///
/// let host = cfg.get("DB_HOST").unwrap_or("localhost");
/// # */
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct ConfigGet {
    stem: String,
    config_dir: String,
    data: ConfigMap,
    loaded_from: Option<PathBuf>,
}

// ── Builder ───────────────────────────────────────────────────────────────────

/// Fluent builder for [`ConfigGet`].
#[must_use]
pub struct ConfigGetBuilder {
    stem: String,
    config_dir: String,
    explicit_path: Option<PathBuf>,
    auto_load: bool,
    create: bool,
}

impl ConfigGetBuilder {
    fn new(stem: impl Into<String>) -> Self {
        let stem = stem.into();
        let config_dir = stem.clone();
        Self {
            stem,
            config_dir,
            explicit_path: None,
            auto_load: true,
            create: false,
        }
    }

    /// Set the sub-directory appended to each search root (default: same as stem).
        pub fn config_dir(mut self, dir: impl Into<String>) -> Self {
        self.config_dir = dir.into();
        self
    }

    /// Load from an explicit path rather than auto-discovering.
        pub fn path(mut self, p: impl Into<PathBuf>) -> Self {
        self.explicit_path = Some(p.into());
        self
    }

    /// Whether to load on [`build`](Self::build) (default: `true`).
        pub fn auto_load(mut self, v: bool) -> Self {
        self.auto_load = v;
        self
    }

    /// Create an empty `.env` file if no config is found (default: `false`).
        pub fn create(mut self, v: bool) -> Self {
        self.create = v;
        self
    }

    /// Build and optionally load the configuration.
    ///
    /// # Errors
    /// Returns [`ConfigError::NotFound`] when no config file exists and
    /// `create` is `false`, [`ConfigError::Io`] on read failure, or
    /// [`ConfigError::Parse`] if the file cannot be parsed.
    pub fn build(self) -> Result<ConfigGet> {
        let mut cfg = ConfigGet {
            stem: self.stem.clone(),
            config_dir: self.config_dir.clone(),
            data: ConfigMap::new(),
            loaded_from: None,
        };

        if self.auto_load {
            if let Some(p) = self.explicit_path {
                cfg.load_from(&p)?;
            } else {
                match cfg.find() {
                    Some(p) => cfg.load_from(&p)?,
                    None if self.create => {
                        let created = cfg.create_default()?;
                        log::info!("Created empty config at {}", created.display());
                        cfg.loaded_from = Some(created);
                    }
                    None => {
                        return Err(ConfigError::NotFound(self.stem));
                    }
                }
            }
        }

        Ok(cfg)
    }
}

// ── ConfigGet ─────────────────────────────────────────────────────────────────

impl ConfigGet {
    // ── construction ──────────────────────────────────────────────────────────

    /// Create a builder for `stem`.
    pub fn builder(stem: impl Into<String>) -> ConfigGetBuilder {
        ConfigGetBuilder::new(stem)
    }

    /// Load directly from an explicit file path.
    ///
    /// # Errors
    /// Returns [`ConfigError::Io`] if the file cannot be read, or
    /// [`ConfigError::Parse`] if the content is malformed.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let stem = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let mut cfg = Self {
            stem: stem.clone(),
            config_dir: stem,
            data: ConfigMap::new(),
            loaded_from: None,
        };
        cfg.load_from(path)?;
        Ok(cfg)
    }

    /// Shortcut: auto-discover and load a `.env` file.
    ///
    /// # Errors
    /// See [`ConfigGetBuilder::build`].
    pub fn from_env(stem: impl Into<String>, config_dir: impl Into<String>) -> Result<Self> {
        Self::builder(stem).config_dir(config_dir).build()
    }

    /// Shortcut: load a `.ini` file from auto-discovered paths.
    ///
    /// # Errors
    /// See [`ConfigGetBuilder::build`].
    pub fn from_ini(stem: impl Into<String>, config_dir: impl Into<String>) -> Result<Self> {
        find_and_load_format(&stem.into(), &config_dir.into(), Format::Ini)
    }

    /// Shortcut: load a `.toml` file from auto-discovered paths.
    ///
    /// # Errors
    /// See [`ConfigGetBuilder::build`].
    pub fn from_toml(stem: impl Into<String>, config_dir: impl Into<String>) -> Result<Self> {
        find_and_load_format(&stem.into(), &config_dir.into(), Format::Toml)
    }

    /// Shortcut: load a `.json` file from auto-discovered paths.
    ///
    /// # Errors
    /// See [`ConfigGetBuilder::build`].
    pub fn from_json(stem: impl Into<String>, config_dir: impl Into<String>) -> Result<Self> {
        find_and_load_format(&stem.into(), &config_dir.into(), Format::Json)
    }

    /// Shortcut: load a `.yaml` / `.yml` file from auto-discovered paths.
    ///
    /// # Errors
    /// See [`ConfigGetBuilder::build`].
    pub fn from_yaml(stem: impl Into<String>, config_dir: impl Into<String>) -> Result<Self> {
        find_and_load_format(&stem.into(), &config_dir.into(), Format::Yaml)
    }

    // ── discovery ─────────────────────────────────────────────────────────────

    /// Return the list of candidate paths that would be searched, in order.
    #[must_use]
    pub fn search_paths(stem: &str, config_dir: &str) -> Vec<PathBuf> {
        let candidates = Format::candidates(stem);
        let dirs = search_dirs(config_dir);

        dirs.into_iter()
            .flat_map(|dir| {
                candidates
                    .iter()
                    .map(move |c| dir.join(c))
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    /// Find the first existing configuration file without loading it.
    #[must_use]
    pub fn find(&self) -> Option<PathBuf> {
        Self::search_paths(&self.stem, &self.config_dir)
            .into_iter()
            .find(|p| p.is_file())
    }

    // ── loading ───────────────────────────────────────────────────────────────

    /// (Re)load configuration from disk.
    ///
    /// Pass `None` to trigger auto-discovery; pass `Some(path)` to load a
    /// specific file.
    ///
    /// # Errors
    /// Returns [`ConfigError::NotFound`] when auto-discovery finds nothing,
    /// [`ConfigError::Io`] on read failure, or [`ConfigError::Parse`] on bad
    /// content.
    pub fn reload(&mut self, path: Option<&Path>) -> Result<()> {
        let p = match path {
            Some(p) => p.to_path_buf(),
            None => self
                .find()
                .ok_or_else(|| ConfigError::NotFound(self.stem.clone()))?,
        };
        self.load_from(&p)
    }

    fn load_from(&mut self, path: &Path) -> Result<()> {
        log::debug!("Loading config from {}", path.display());

        let format = Format::from_path(path).ok_or_else(|| ConfigError::Parse {
            path: path.display().to_string(),
            message: "unrecognised file extension".to_string(),
        })?;

        self.data = match format {
            Format::Env  => parsers::parse_env(path)?,
            Format::Ini  => parsers::parse_ini(path)?,
            Format::Toml => parsers::parse_toml(path)?,
            Format::Json => parsers::parse_json(path)?,
            Format::Yaml => parsers::parse_yaml(path)?,
        };

        self.loaded_from = Some(path.to_path_buf());
        Ok(())
    }

    fn create_default(&self) -> Result<PathBuf> {
        let dirs = search_dirs(&self.config_dir);
        let dir = dirs.first().ok_or_else(|| {
            ConfigError::Other("could not determine a directory to create config in".into())
        })?;
        std::fs::create_dir_all(dir).map_err(|e| ConfigError::Io {
            path: dir.display().to_string(),
            source: e,
        })?;
        let path = dir.join(".env");
        std::fs::write(&path, "# config-get generated empty config\n").map_err(|e| {
            ConfigError::Io {
                path: path.display().to_string(),
                source: e,
            }
        })?;
        Ok(path)
    }

    // ── getters ───────────────────────────────────────────────────────────────

    /// Look up a flat key, returning `None` if absent.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&str> {
        self.data.get_flat(key)
    }

    /// Look up a flat key with a default fallback value.
    #[must_use]
    pub fn get_or<'a>(&'a self, key: &str, fallback: &'a str) -> &'a str {
        self.get(key).unwrap_or(fallback)
    }

    /// Look up a flat key, returning `Err` if absent.
    ///
    /// # Errors
    /// Returns [`ConfigError::KeyNotFound`] when the key does not exist.
    pub fn require(&self, key: &str) -> Result<&str> {
        self.get(key)
            .ok_or_else(|| ConfigError::KeyNotFound(key.to_string()))
    }

    /// Look up `key` inside `section`, returning `None` if absent.
    #[must_use]
    pub fn get_in(&self, section: &str, key: &str) -> Option<&str> {
        self.data.get_in_section(section, key)
    }

    /// Look up `key` inside `section` with a default fallback value.
    #[must_use]
    pub fn get_in_or<'a>(&'a self, section: &str, key: &str, fallback: &'a str) -> &'a str {
        self.get_in(section, key).unwrap_or(fallback)
    }

    /// Look up `key` inside `section`, returning `Err` if absent.
    ///
    /// # Errors
    /// Returns [`ConfigError::KeyNotFound`] when the key does not exist.
    pub fn require_in(&self, section: &str, key: &str) -> Result<&str> {
        self.get_in(section, key)
            .ok_or_else(|| ConfigError::KeyNotFound(format!("{section}.{key}")))
    }

    /// Return a clone of all key/value pairs in `section`.
    ///
    /// # Errors
    /// Returns [`ConfigError::SectionNotFound`] when the section does not exist.
    pub fn get_section(&self, section: &str) -> Result<indexmap::IndexMap<String, String>> {
        self.data
            .get_section(section)
            .cloned()
            .ok_or_else(|| ConfigError::SectionNotFound(section.to_string()))
    }

    /// Return a clone of the entire configuration as a [`ConfigMap`].
    #[must_use]
    pub fn all(&self) -> ConfigMap {
        self.data.clone()
    }

    /// The path the configuration was loaded from, if any.
    #[must_use]
    pub fn loaded_from(&self) -> Option<&Path> {
        self.loaded_from.as_deref()
    }

    // ── typed helpers ─────────────────────────────────────────────────────────

    /// Parse the value at `key` into `T: FromStr`.
    ///
    /// # Errors
    /// Returns [`ConfigError::KeyNotFound`] if the key is missing, or
    /// [`ConfigError::Parse`] if the value cannot be parsed into `T`.
    pub fn parse<T>(&self, key: &str) -> Result<T>
    where
        T: std::str::FromStr,
        T::Err: std::fmt::Display,
    {
        let v = self.require(key)?;
        v.parse::<T>().map_err(|e| ConfigError::Parse {
            path: self
                .loaded_from
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default(),
            message: format!("key '{key}': {e}"),
        })
    }

    /// Parse the value at `section.key` into `T: FromStr`.
    ///
    /// # Errors
    /// Returns [`ConfigError::KeyNotFound`] if the key is missing, or
    /// [`ConfigError::Parse`] if the value cannot be parsed into `T`.
    pub fn parse_in<T>(&self, section: &str, key: &str) -> Result<T>
    where
        T: std::str::FromStr,
        T::Err: std::fmt::Display,
    {
        let v = self.require_in(section, key)?;
        v.parse::<T>().map_err(|e| ConfigError::Parse {
            path: self
                .loaded_from
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default(),
            message: format!("key '{section}.{key}': {e}"),
        })
    }

    // ── iteration ─────────────────────────────────────────────────────────────

    /// Iterate over all flat key/value pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.data.flat_iter()
    }

    /// Iterate over all section names.
    pub fn sections(&self) -> impl Iterator<Item = &str> {
        self.data.sections().map(|(s, _)| s)
    }

    /// Number of top-level entries (flat keys + section names).
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// True if there are no entries at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// True if the flat map or any section contains `key`.
    #[must_use]
    pub fn contains_key(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }
}

// ── std::ops::Index ───────────────────────────────────────────────────────────

impl std::ops::Index<&str> for ConfigGet {
    type Output = str;

    /// Index into the config by flat key. Panics if the key is missing.
    fn index(&self, key: &str) -> &Self::Output {
        self.get(key)
            .unwrap_or_else(|| panic!("config key not found: '{key}'"))
    }
}

// ── private helper ────────────────────────────────────────────────────────────

fn find_and_load_format(stem: &str, config_dir: &str, fmt: Format) -> Result<ConfigGet> {
    let ext = match fmt {
        Format::Env  => ".env".to_string(),
        Format::Ini  => format!("{stem}.ini"),
        Format::Toml => format!("{stem}.toml"),
        Format::Json => format!("{stem}.json"),
        Format::Yaml => format!("{stem}.yml"),
    };

    let path = search_dirs(config_dir)
        .into_iter()
        .map(|d| d.join(&ext))
        .find(|p| p.is_file())
        .ok_or_else(|| ConfigError::NotFound(format!("{stem} ({fmt})")))?;

    ConfigGet::from_file(path)
}
