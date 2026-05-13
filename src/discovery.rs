use std::path::PathBuf;

/// Returns a prioritised list of directories to search for configuration files.
///
/// ### Windows
/// 1. `%APPDATA%\<config_dir>\`
/// 2. `%USERPROFILE%\<config_dir>\`
/// 3. `%APPDATA%\`
/// 4. `%USERPROFILE%\`
/// 5. Current working directory
///
/// ### Linux / macOS
/// 1. `~/<config_dir>/`
/// 2. `~/.config/<config_dir>/`
/// 3. `~/.config/`
/// 4. `~/`
/// 5. Current working directory
#[must_use]
pub fn search_dirs(config_dir: &str) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = Vec::new();

    #[cfg(target_os = "windows")]
    {
        if let Some(app_data) = dirs::data_dir() {
            dirs.push(app_data.join(config_dir));
            dirs.push(app_data);
        }
        if let Some(home) = dirs::home_dir() {
            dirs.push(home.join(config_dir));
            dirs.push(home);
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Some(home) = dirs::home_dir() {
            dirs.push(home.join(config_dir));
            dirs.push(home.join(".config").join(config_dir));
            dirs.push(home.join(".config"));
            dirs.push(home);
        }
    }

    if let Ok(cwd) = std::env::current_dir() {
        dirs.push(cwd);
    }

    let mut seen = std::collections::HashSet::new();
    dirs.retain(|p| seen.insert(p.clone()));

    dirs
}
