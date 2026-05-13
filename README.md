# config-get

> Cross-platform configuration file locator and reader for Rust.

[![Crates.io](https://img.shields.io/crates/v/config-get.svg)](https://crates.io/crates/config-get)
[![Docs.rs](https://docs.rs/config-get/badge.svg)](https://docs.rs/config-get/latest/config_get/)
[![CI](https://github.com/cumulus13/config-get-rs/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/cumulus13/config-get-rs/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![MSRV: 1.80](https://img.shields.io/badge/rustc-1.80+-blue.svg)](https://blog.rust-lang.org/2023/06/01/Rust-1.80.0.html)

**config-get** automatically discovers and reads configuration files from standard OS-specific locations. Supports `.env`, `.ini`, `.toml`, `.json`, `.yml`, and `.yaml` formats — no manual path wrangling required.

---

## Features

- 🔍 **Auto-discovery** — searches platform-standard directories (`%APPDATA%`, `~/.config`, etc.)
- 📄 **Multi-format** — `.env`, `.ini`, `.toml`, `.json`, `.yml` / `.yaml`
- 🪟 **Cross-platform** — Windows, Linux, macOS (tested in CI)
- 🔗 **Minimal deps** — optional format features keep the dependency tree lean
- 🦀 **Idiomatic Rust** — builder pattern, typed parsing, `Index` operator, `thiserror`-based errors
- 🔄 **Reload support** — re-read config from disk at any time
- 🖥️  **Optional CLI** — inspect and query configs from the terminal

---

## Installation

```toml
[dependencies]
config-get = "0.1.1"
```

With optional format support:

```toml
# All formats (recommended)
config-get = { version = "0.1.1", features = ["all"] }

# Pick and choose
config-get = { version = "0.1.1", features = ["toml", "yaml", "dotenv", "ini"] }
```

| Feature  | Enables            | Crate         | Default |
|----------|--------------------|---------------|---------|
| `dotenv` | `.env` parsing     | `dotenvy`     | ✓       |
| `ini`    | `.ini` parsing     | `rust-ini`    | ✓       |
| `toml`   | `.toml` parsing    | `toml`        | ✓       |
| `yaml`   | `.yml`/`.yaml`     | `serde_yaml`  | ✓       |
| `cli`    | `config-get` binary| `clap`        | ✗       |
| `all`    | All of the above   | —             | ✗       |

---

## Quick Start

```rust
use config_get::ConfigGet;

fn main() -> config_get::Result<()> {
    // Auto-discover a config file for "myapp"
    let cfg = ConfigGet::builder("myapp")
        .config_dir("myapp")
        .build()?;

    // Flat key lookup
    let host = cfg.get("DB_HOST").unwrap_or("localhost");

    // Typed parsing
    let port: u16 = cfg.parse("DB_PORT")?;

    // Section-aware access (.ini / .toml / nested JSON/YAML)
    let debug = cfg.get_in_or("server", "debug", "false");

    // Require a key (returns Err if missing)
    let api_key = cfg.require("API_KEY")?;

    // Index operator (panics if missing)
    println!("greeting = {}", &cfg["GREETING"]);

    Ok(())
}
```

---

## Search Order

### Linux / macOS

| Priority | Path |
|----------|------|
| 1 | `~/<config_dir>/` |
| 2 | `~/.config/<config_dir>/` |
| 3 | `~/.config/` |
| 4 | `~/` |
| 5 | Current working directory |

### Windows

| Priority | Path |
|----------|------|
| 1 | `%APPDATA%\<config_dir>\` |
| 2 | `%USERPROFILE%\<config_dir>\` |
| 3 | `%APPDATA%\` |
| 4 | `%USERPROFILE%\` |
| 5 | Current working directory |

Within each directory, the following filenames are checked in order:

```
.env  →  <stem>.ini  →  <stem>.toml  →  <stem>.json  →  <stem>.yml  →  <stem>.yaml
```

---

## API Reference

### Builder

```rust
let cfg = ConfigGet::builder("myapp")
    .config_dir("myapp")     // sub-directory to search (default: same as stem)
    .auto_load(true)         // load on build() (default: true)
    .create(false)           // create an empty .env if not found (default: false)
    .build()?;
```

### Shortcut constructors

```rust
ConfigGet::from_file("path/to/config.toml")?;   // explicit path
ConfigGet::from_env("myapp", "myapp")?;          // .env shortcut
ConfigGet::from_ini("myapp", "myapp")?;
ConfigGet::from_toml("myapp", "myapp")?;
ConfigGet::from_json("myapp", "myapp")?;
ConfigGet::from_yaml("myapp", "myapp")?;
```

### Reading values

| Method | Description |
|--------|-------------|
| `cfg.get("KEY")` | Flat lookup → `Option<&str>` |
| `cfg.get_or("KEY", "default")` | Flat lookup with fallback |
| `cfg.require("KEY")` | Flat lookup, `Err` if absent |
| `cfg.get_in("section", "key")` | Section + key → `Option<&str>` |
| `cfg.get_in_or("section", "key", "default")` | Section + key with fallback |
| `cfg.require_in("section", "key")` | Section + key, `Err` if absent |
| `cfg.get_section("section")` | Entire section as `IndexMap` |
| `cfg.parse::<T>("KEY")` | Flat key parsed into `T: FromStr` |
| `cfg.parse_in::<T>("section", "key")` | Section key parsed into `T` |
| `cfg.all()` | Clone of entire `ConfigMap` |
| `cfg.reload(None)` | Re-read from disk (auto-discover) |
| `cfg.reload(Some(path))` | Re-read from explicit path |
| `cfg.loaded_from()` | Path the config was loaded from |

### Discovery helpers

```rust
// Inspect candidate paths without loading
let paths = ConfigGet::search_paths("myapp", "myapp");
for p in &paths {
    println!("{}", p.display());
}

// Module-level helper
use config_get::get_config_file;
if let Some(path) = get_config_file("myapp", "myapp") {
    println!("Found: {}", path.display());
}
```

### Iteration

```rust
// Flat entries
for (key, value) in cfg.iter() {
    println!("{key} = {value}");
}

// Section names
for section in cfg.sections() {
    println!("[{section}]");
}

// Membership
if cfg.contains_key("API_KEY") { ... }
println!("total entries: {}", cfg.len());
```

---

## Format Examples

### `.env`

```env
DB_HOST=localhost
DB_PORT=5432
SECRET_KEY="my-secret"
```

```rust
let cfg = ConfigGet::from_file(".env")?;
println!("{}", cfg["DB_HOST"]);   // localhost
```

### `.ini`

```ini
[database]
host = localhost
port = 5432

[server]
debug = true
```

```rust
let cfg = ConfigGet::from_file("app.ini")?;
println!("{}", cfg.get_in_or("database", "host", "localhost"));
let section = cfg.get_section("server")?;
```

### `.toml`

```toml
[database]
host = "localhost"
port = 5432
```

```rust
let cfg = ConfigGet::from_file("config.toml")?;
println!("{}", cfg.get_in_or("database", "host", "localhost"));
```

### `.json`

```json
{
  "database": { "host": "localhost", "port": 5432 },
  "server":   { "debug": true }
}
```

```rust
let cfg = ConfigGet::from_file("config.json")?;
let port: u16 = cfg.parse_in("database", "port")?;
```

### `.yaml` / `.yml`

```yaml
database:
  host: localhost
  port: 5432
```

```rust
let cfg = ConfigGet::from_file("config.yaml")?;
println!("{}", cfg.get_in_or("database", "host", "localhost"));
```

---

## Error Handling

All errors implement `std::error::Error` via [`thiserror`](https://crates.io/crates/thiserror):

```rust
use config_get::{ConfigGet, ConfigError};

match ConfigGet::builder("myapp").config_dir("myapp").build() {
    Ok(cfg) => { /* ... */ }
    Err(ConfigError::NotFound(name)) => eprintln!("No config found for {name}"),
    Err(ConfigError::Parse { path, message }) => eprintln!("Parse error in {path}: {message}"),
    Err(ConfigError::KeyNotFound(key)) => eprintln!("Missing key: {key}"),
    Err(e) => eprintln!("Error: {e}"),
}
```

---

## CLI

Enable the `cli` feature and the `config-get` binary is built:

```bash
cargo install config-get --features cli
```

```
config-get find myapp --dir myapp
config-get dump myapp --dir myapp
config-get get  myapp DB_HOST --dir myapp --fallback localhost
config-get get  myapp server.debug --dir myapp
config-get paths myapp --dir myapp
```

---

## Logging

`config-get` uses the standard [`log`](https://crates.io/crates/log) facade.
Wire up any compatible backend (e.g. `env_logger`) and set `RUST_LOG=debug` to
see which files are being searched and loaded.

```toml
[dev-dependencies]
env_logger = "0.11"
```

```rust
env_logger::init();
// Now config-get emits debug-level messages to stderr.
```

---

## MSRV

Minimum Supported Rust Version: **1.70** (tested in CI against stable, beta, and 1.70).

---

## License

MIT © [Hadi Cahyadi](https://github.com/cumulus13)

## Author

[Hadi Cahyadi](mailto:cumulus13@gmail.com) — [@cumulus13](https://github.com/cumulus13)

[![Buy Me a Coffee](https://www.buymeacoffee.com/assets/img/custom_images/orange_img.png)](https://www.buymeacoffee.com/cumulus13)
[![Ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/cumulus13)
[Support on Patreon](https://www.patreon.com/cumulus13)
