use std::path::Path;

/// Supported configuration file formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Format {
    /// Shell-style `KEY=VALUE` / dotenv files (`.env`)
    Env,
    /// Windows-style `.ini` files with `[sections]`
    Ini,
    /// TOML files
    Toml,
    /// JSON files
    Json,
    /// YAML files (`.yml` / `.yaml`)
    Yaml,
}

impl Format {
    /// Detect format from file extension.
    ///
    /// Returns `None` if the extension is unrecognised.
    #[must_use]
    pub fn from_path(path: &Path) -> Option<Self> {
        let name = path.file_name()?.to_string_lossy();

        if name == ".env" || name.starts_with(".env.") || name.ends_with(".env") {
            return Some(Self::Env);
        }

        match path.extension()?.to_string_lossy().as_ref() {
            "ini" => Some(Self::Ini),
            "toml" => Some(Self::Toml),
            "json" => Some(Self::Json),
            "yml" | "yaml" => Some(Self::Yaml),
            _ => None,
        }
    }

    /// Human-readable label for this format.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Env => "env",
            Self::Ini => "ini",
            Self::Toml => "toml",
            Self::Json => "json",
            Self::Yaml => "yaml",
        }
    }

    /// Returns the canonical file names checked for each format, in priority order.
    #[must_use]
    pub fn candidates(stem: &str) -> Vec<String> {
        vec![
            ".env".to_string(),
            format!("{stem}.ini"),
            format!("{stem}.toml"),
            format!("{stem}.json"),
            format!("{stem}.yml"),
            format!("{stem}.yaml"),
        ]
    }
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
