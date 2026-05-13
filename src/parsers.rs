use crate::error::{ConfigError, Result};
use crate::value::ConfigMap;
use std::path::Path;

// ── .env ──────────────────────────────────────────────────────────────────────

/// Parse a dotenv-style file into a flat [`ConfigMap`].
///
/// # Errors
/// Returns [`ConfigError::Io`] if the file cannot be read, or
/// [`ConfigError::Parse`] if the content is malformed.
pub fn parse_env(path: &Path) -> Result<ConfigMap> {
    #[cfg(feature = "dotenv")]
    {
        parse_env_dotenvy(path)
    }
    #[cfg(not(feature = "dotenv"))]
    {
        parse_env_builtin(path)
    }
}

#[cfg(feature = "dotenv")]
fn parse_env_dotenvy(path: &Path) -> Result<ConfigMap> {
    let mut map = ConfigMap::new();
    let iter = dotenvy::from_path_iter(path).map_err(|e| ConfigError::Parse {
        path: path.display().to_string(),
        message: e.to_string(),
    })?;
    for item in iter {
        let (k, v) = item.map_err(|e| ConfigError::Parse {
            path: path.display().to_string(),
            message: e.to_string(),
        })?;
        map.insert_flat(k, v);
    }
    Ok(map)
}

#[cfg(not(feature = "dotenv"))]
fn parse_env_builtin(path: &Path) -> Result<ConfigMap> {
    let content = read_file(path)?;
    let mut map = ConfigMap::new();
    for (line_no, raw) in content.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some(eq) = line.find('=') else {
            log::debug!(
                "{}:{}: skipping malformed line (no '=')",
                path.display(),
                line_no + 1
            );
            continue;
        };
        let key   = line[..eq].trim().to_string();
        let value = strip_quotes(line[eq + 1..].trim()).to_string();
        map.insert_flat(key, value);
    }
    Ok(map)
}

#[cfg(not(feature = "dotenv"))]
fn strip_quotes(s: &str) -> &str {
    for q in ['"', '\''] {
        if s.starts_with(q) && s.ends_with(q) && s.len() >= 2 {
            return &s[1..s.len() - 1];
        }
    }
    s
}

// ── .ini ──────────────────────────────────────────────────────────────────────

/// Parse an INI file into a sectioned [`ConfigMap`].
///
/// # Errors
/// Returns [`ConfigError::FeatureNotEnabled`] when the `ini` feature is off,
/// [`ConfigError::Io`] on read failure, or [`ConfigError::Parse`] on bad syntax.
pub fn parse_ini(path: &Path) -> Result<ConfigMap> {
    #[cfg(feature = "ini")]
    {
        parse_ini_impl(path)
    }
    #[cfg(not(feature = "ini"))]
    {
        let _ = path;
        Err(ConfigError::FeatureNotEnabled { feature: "ini" })
    }
}

#[cfg(feature = "ini")]
fn parse_ini_impl(path: &Path) -> Result<ConfigMap> {
    use ini::Ini;
    let conf = Ini::load_from_file(path).map_err(|e| ConfigError::Parse {
        path: path.display().to_string(),
        message: e.to_string(),
    })?;
    let mut map = ConfigMap::new();
    for (section, props) in &conf {
        for (key, value) in props {
            match section {
                Some(s) => map.insert_sectioned(s.to_string(), key.to_string(), value.to_string()),
                None    => map.insert_flat(key.to_string(), value.to_string()),
            }
        }
    }
    Ok(map)
}

// ── .toml ─────────────────────────────────────────────────────────────────────

/// Parse a TOML file into a [`ConfigMap`].
///
/// # Errors
/// Returns [`ConfigError::FeatureNotEnabled`] when the `toml` feature is off,
/// [`ConfigError::Io`] on read failure, or [`ConfigError::Parse`] on bad syntax.
pub fn parse_toml(path: &Path) -> Result<ConfigMap> {
    #[cfg(feature = "toml")]
    {
        parse_toml_impl(path)
    }
    #[cfg(not(feature = "toml"))]
    {
        let _ = path;
        Err(ConfigError::FeatureNotEnabled { feature: "toml" })
    }
}

#[cfg(feature = "toml")]
fn parse_toml_impl(path: &Path) -> Result<ConfigMap> {
    let content = read_file(path)?;
    let value: toml::Value = content.parse().map_err(|e: toml::de::Error| ConfigError::Parse {
        path: path.display().to_string(),
        message: e.to_string(),
    })?;
    let mut map = ConfigMap::new();
    flatten_toml_value(&value, None, &mut map);
    Ok(map)
}

#[cfg(feature = "toml")]
fn flatten_toml_value(value: &toml::Value, section: Option<&str>, map: &mut ConfigMap) {
    if let toml::Value::Table(table) = value {
        for (k, v) in table {
            if let toml::Value::Table(_) = v {
                flatten_toml_value(v, Some(k), map);
            } else {
                let str_val = toml_value_to_string(v);
                match section {
                    Some(s) => map.insert_sectioned(s.to_string(), k.clone(), str_val),
                    None    => map.insert_flat(k.clone(), str_val),
                }
            }
        }
    }
}

#[cfg(feature = "toml")]
fn toml_value_to_string(v: &toml::Value) -> String {
    match v {
        toml::Value::String(s)   => s.clone(),
        toml::Value::Integer(i)  => i.to_string(),
        toml::Value::Float(f)    => f.to_string(),
        toml::Value::Boolean(b)  => b.to_string(),
        toml::Value::Datetime(d) => d.to_string(),
        toml::Value::Array(a) => {
            let items: Vec<_> = a.iter().map(toml_value_to_string).collect();
            format!("[{}]", items.join(", "))
        }
        toml::Value::Table(_) => "[table]".to_string(),
    }
}

// ── .json ─────────────────────────────────────────────────────────────────────

/// Parse a JSON file into a [`ConfigMap`].
///
/// # Errors
/// Returns [`ConfigError::Io`] on read failure or [`ConfigError::Parse`] on
/// invalid JSON.
pub fn parse_json(path: &Path) -> Result<ConfigMap> {
    let content = read_file(path)?;
    let value: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| ConfigError::Parse {
            path: path.display().to_string(),
            message: e.to_string(),
        })?;
    let mut map = ConfigMap::new();
    flatten_json_value(&value, None, &mut map);
    Ok(map)
}

fn flatten_json_value(value: &serde_json::Value, section: Option<&str>, map: &mut ConfigMap) {
    if let serde_json::Value::Object(obj) = value {
        for (k, v) in obj {
            if let serde_json::Value::Object(_) = v {
                flatten_json_value(v, Some(k), map);
            } else {
                let str_val = json_value_to_string(v);
                match section {
                    Some(s) => map.insert_sectioned(s.to_string(), k.clone(), str_val),
                    None    => map.insert_flat(k.clone(), str_val),
                }
            }
        }
    }
}

fn json_value_to_string(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b)   => b.to_string(),
        serde_json::Value::Null      => String::new(),
        serde_json::Value::Array(a) => {
            let items: Vec<_> = a.iter().map(json_value_to_string).collect();
            format!("[{}]", items.join(", "))
        }
        serde_json::Value::Object(_) => "[object]".to_string(),
    }
}

// ── .yaml / .yml ──────────────────────────────────────────────────────────────

/// Parse a YAML file into a [`ConfigMap`].
///
/// # Errors
/// Returns [`ConfigError::FeatureNotEnabled`] when the `yaml` feature is off,
/// [`ConfigError::Io`] on read failure, or [`ConfigError::Parse`] on bad syntax.
pub fn parse_yaml(path: &Path) -> Result<ConfigMap> {
    #[cfg(feature = "yaml")]
    {
        parse_yaml_impl(path)
    }
    #[cfg(not(feature = "yaml"))]
    {
        let _ = path;
        Err(ConfigError::FeatureNotEnabled { feature: "yaml" })
    }
}

#[cfg(feature = "yaml")]
fn parse_yaml_impl(path: &Path) -> Result<ConfigMap> {
    let content = read_file(path)?;
    let value: serde_yaml::Value =
        serde_yaml::from_str(&content).map_err(|e| ConfigError::Parse {
            path: path.display().to_string(),
            message: e.to_string(),
        })?;
    let mut map = ConfigMap::new();
    flatten_yaml_value(&value, None, &mut map);
    Ok(map)
}

#[cfg(feature = "yaml")]
fn flatten_yaml_value(value: &serde_yaml::Value, section: Option<&str>, map: &mut ConfigMap) {
    if let serde_yaml::Value::Mapping(mapping) = value {
        for (k, v) in mapping {
            let key = yaml_key_to_string(k);
            if let serde_yaml::Value::Mapping(_) = v {
                flatten_yaml_value(v, Some(&key), map);
            } else {
                let str_val = yaml_value_to_string(v);
                match section {
                    Some(s) => map.insert_sectioned(s.to_string(), key, str_val),
                    None    => map.insert_flat(key, str_val),
                }
            }
        }
    }
}

#[cfg(feature = "yaml")]
fn yaml_key_to_string(v: &serde_yaml::Value) -> String {
    match v {
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Number(n) => n.to_string(),
        serde_yaml::Value::Bool(b)   => b.to_string(),
        _                            => format!("{v:?}"),
    }
}

#[cfg(feature = "yaml")]
fn yaml_value_to_string(v: &serde_yaml::Value) -> String {
    match v {
        serde_yaml::Value::String(s)     => s.clone(),
        serde_yaml::Value::Number(n)     => n.to_string(),
        serde_yaml::Value::Bool(b)       => b.to_string(),
        serde_yaml::Value::Null          => String::new(),
        serde_yaml::Value::Sequence(seq) => {
            let items: Vec<_> = seq.iter().map(yaml_value_to_string).collect();
            format!("[{}]", items.join(", "))
        }
        serde_yaml::Value::Mapping(_) => "[mapping]".to_string(),
        serde_yaml::Value::Tagged(t)  => yaml_value_to_string(&t.value),
    }
}

// ── shared helper ─────────────────────────────────────────────────────────────

fn read_file(path: &Path) -> Result<String> {
    std::fs::read_to_string(path).map_err(|e| ConfigError::Io {
        path: path.display().to_string(),
        source: e,
    })
}
