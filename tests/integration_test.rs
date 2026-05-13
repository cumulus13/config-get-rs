use config_get::{ConfigError, ConfigGet, Result};
use std::io::Write;
use tempfile::NamedTempFile;

// ── helpers ───────────────────────────────────────────────────────────────────

fn tmp_file(ext: &str, contents: &str) -> NamedTempFile {
    let mut f = tempfile::Builder::new()
        .suffix(ext)
        .tempfile()
        .expect("tempfile");
    write!(f, "{contents}").expect("write");
    f
}

// ── .env ──────────────────────────────────────────────────────────────────────

#[test]
fn test_env_basic() -> Result<()> {
    let f = tmp_file(
        ".env",
        "DB_HOST=localhost\nDB_PORT=5432\nSECRET=\"my-secret\"\n",
    );
    let cfg = ConfigGet::from_file(f.path())?;

    assert_eq!(cfg.get("DB_HOST"), Some("localhost"));
    assert_eq!(cfg.get("DB_PORT"), Some("5432"));
    assert_eq!(cfg.get("SECRET"), Some("my-secret")); // quotes stripped
    Ok(())
}

#[test]
fn test_env_missing_key() -> Result<()> {
    let f = tmp_file(".env", "A=1\n");
    let cfg = ConfigGet::from_file(f.path())?;
    assert_eq!(cfg.get("MISSING"), None);
    Ok(())
}

#[test]
fn test_env_require_missing() {
    let f = tmp_file(".env", "A=1\n");
    let cfg = ConfigGet::from_file(f.path()).unwrap();
    let err = cfg.require("MISSING").unwrap_err();
    assert!(matches!(err, ConfigError::KeyNotFound(_)));
}

#[test]
fn test_env_parse_typed() -> Result<()> {
    let f = tmp_file(".env", "PORT=8080\nDEBUG=true\n");
    let cfg = ConfigGet::from_file(f.path())?;
    let port: u16 = cfg.parse("PORT")?;
    let debug: bool = cfg.parse("DEBUG")?;
    assert_eq!(port, 8080);
    assert!(debug);
    Ok(())
}

#[test]
fn test_env_comments_and_empty_lines() -> Result<()> {
    let f = tmp_file(".env", "# comment\n\nKEY=value\n");
    let cfg = ConfigGet::from_file(f.path())?;
    assert_eq!(cfg.get("KEY"), Some("value"));
    Ok(())
}

// ── .json ─────────────────────────────────────────────────────────────────────

#[test]
fn test_json_flat() -> Result<()> {
    let f = tmp_file(".json", r#"{"host":"localhost","port":"5432"}"#);
    let cfg = ConfigGet::from_file(f.path())?;
    assert_eq!(cfg.get("host"), Some("localhost"));
    assert_eq!(cfg.get("port"), Some("5432"));
    Ok(())
}

#[test]
fn test_json_sectioned() -> Result<()> {
    let f = tmp_file(
        ".json",
        r#"{"database":{"host":"db.local","port":5432},"server":{"debug":true}}"#,
    );
    let cfg = ConfigGet::from_file(f.path())?;
    assert_eq!(cfg.get_in("database", "host"), Some("db.local"));
    assert_eq!(cfg.get_in("database", "port"), Some("5432"));
    assert_eq!(cfg.get_in("server", "debug"), Some("true"));
    Ok(())
}

// ── .toml ─────────────────────────────────────────────────────────────────────

#[cfg(feature = "toml")]
#[test]
fn test_toml_sectioned() -> Result<()> {
    let f = tmp_file(
        ".toml",
        "[database]\nhost = \"localhost\"\nport = 5432\n\n[server]\ndebug = true\n",
    );
    let cfg = ConfigGet::from_file(f.path())?;
    assert_eq!(cfg.get_in("database", "host"), Some("localhost"));
    assert_eq!(cfg.get_in("database", "port"), Some("5432"));
    assert_eq!(cfg.get_in("server", "debug"), Some("true"));
    Ok(())
}

// ── .ini ──────────────────────────────────────────────────────────────────────

#[cfg(feature = "ini")]
#[test]
fn test_ini_sectioned() -> Result<()> {
    let f = tmp_file(
        ".ini",
        "[database]\nhost = localhost\nport = 5432\n\n[server]\ndebug = true\n",
    );
    let cfg = ConfigGet::from_file(f.path())?;
    assert_eq!(cfg.get_in("database", "host"), Some("localhost"));
    assert_eq!(cfg.get_in("server", "debug"), Some("true"));
    Ok(())
}

// ── .yaml ─────────────────────────────────────────────────────────────────────

#[cfg(feature = "yaml")]
#[test]
fn test_yaml_sectioned() -> Result<()> {
    let f = tmp_file(".yml", "database:\n  host: localhost\n  port: 5432\n");
    let cfg = ConfigGet::from_file(f.path())?;
    assert_eq!(cfg.get_in("database", "host"), Some("localhost"));
    assert_eq!(cfg.get_in("database", "port"), Some("5432"));
    Ok(())
}

// ── index operator ────────────────────────────────────────────────────────────

#[test]
fn test_index_operator() -> Result<()> {
    let f = tmp_file(".env", "GREETING=hello\n");
    let cfg = ConfigGet::from_file(f.path())?;
    assert_eq!(&cfg["GREETING"], "hello");
    Ok(())
}

#[test]
#[should_panic(expected = "config key not found")]
fn test_index_operator_panics_on_missing() {
    let f = tmp_file(".env", "A=1\n");
    let cfg = ConfigGet::from_file(f.path()).unwrap();
    let _ = &cfg["MISSING"];
}

// ── get_or / fallback ─────────────────────────────────────────────────────────

#[test]
fn test_get_or_fallback() -> Result<()> {
    let f = tmp_file(".env", "A=1\n");
    let cfg = ConfigGet::from_file(f.path())?;
    assert_eq!(cfg.get_or("A", "default"), "1");
    assert_eq!(cfg.get_or("MISSING", "default"), "default");
    Ok(())
}

// ── search_paths ──────────────────────────────────────────────────────────────

#[test]
fn test_search_paths_nonempty() {
    let paths = ConfigGet::search_paths("myapp", "myapp");
    assert!(!paths.is_empty());
    // Every candidate should end with a recognised filename.
    for p in &paths {
        let name = p.file_name().unwrap().to_string_lossy();
        assert!(
            name == ".env"
                || name.ends_with(".ini")
                || name.ends_with(".toml")
                || name.ends_with(".json")
                || name.ends_with(".yml")
                || name.ends_with(".yaml"),
            "unexpected candidate filename: {name}"
        );
    }
}

// ── reload ────────────────────────────────────────────────────────────────────

#[test]
fn test_reload() -> Result<()> {
    let mut f = tmp_file(".env", "KEY=original\n");
    let mut cfg = ConfigGet::from_file(f.path())?;
    assert_eq!(cfg.get("KEY"), Some("original"));

    // Overwrite the file.
    f.as_file_mut().set_len(0).unwrap();
    use std::io::Seek;
    f.as_file_mut().seek(std::io::SeekFrom::Start(0)).unwrap();
    write!(f, "KEY=updated\n").unwrap();

    cfg.reload(Some(f.path()))?;
    assert_eq!(cfg.get("KEY"), Some("updated"));
    Ok(())
}

// ── get_section ───────────────────────────────────────────────────────────────

#[test]
fn test_get_section() -> Result<()> {
    let f = tmp_file(".json", r#"{"server":{"host":"0.0.0.0","port":"80"}}"#);
    let cfg = ConfigGet::from_file(f.path())?;
    let section = cfg.get_section("server")?;
    assert_eq!(section.get("host").map(String::as_str), Some("0.0.0.0"));
    assert_eq!(section.get("port").map(String::as_str), Some("80"));
    Ok(())
}

#[test]
fn test_get_section_missing() -> Result<()> {
    let f = tmp_file(".json", r#"{"a":"1"}"#);
    let cfg = ConfigGet::from_file(f.path())?;
    let err = cfg.get_section("nonexistent").unwrap_err();
    assert!(matches!(err, ConfigError::SectionNotFound(_)));
    Ok(())
}
