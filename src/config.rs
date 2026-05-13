use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorConfig {
    #[serde(default = "default_modified")]
    pub modified: String,
    #[serde(default = "default_deleted")]
    pub deleted: String,
    #[serde(default = "default_new_file")]
    pub new_file: String,
    #[serde(default = "default_renamed")]
    pub renamed: String,
    #[serde(default = "default_added")]
    pub added: String,
    #[serde(default = "default_untracked")]
    pub untracked: String,
    #[serde(default = "default_staged")]
    pub staged: String,
    #[serde(default = "default_not_staged")]
    pub not_staged: String,
    #[serde(default = "default_header")]
    pub header: String,
    #[serde(default = "default_branch")]
    pub branch: String,
    #[serde(default = "default_up_to_date")]
    pub up_to_date: String,
    #[serde(default = "default_ahead_behind")]
    pub ahead_behind: String,
    #[serde(default = "default_hint")]
    pub hint: String,
    #[serde(default = "default_cwd_label")]
    pub cwd_label: String,
    #[serde(default = "default_cwd_path")]
    pub cwd_path: String,
    #[serde(default = "default_remote_url")]
    pub remote_url: String,
    #[serde(default = "default_remote_pr")]
    pub remote_pr: String,
    #[serde(default = "default_remote_issue")]
    pub remote_issue: String,
    #[serde(default = "default_arrow")]
    pub arrow: String,
    #[serde(default = "default_tree_dir")]
    pub tree_dir: String,
    #[serde(default = "default_tree_file")]
    pub tree_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_tree_mode")]
    pub tree_mode: bool,
    #[serde(default)]
    pub colors: ColorConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            tree_mode: true,
            colors: ColorConfig::default(),
        }
    }
}

impl Default for ColorConfig {
    fn default() -> Self {
        Self {
            modified: default_modified(),
            deleted: default_deleted(),
            new_file: default_new_file(),
            renamed: default_renamed(),
            added: default_added(),
            untracked: default_untracked(),
            staged: default_staged(),
            not_staged: default_not_staged(),
            header: default_header(),
            branch: default_branch(),
            up_to_date: default_up_to_date(),
            ahead_behind: default_ahead_behind(),
            hint: default_hint(),
            cwd_label: default_cwd_label(),
            cwd_path: default_cwd_path(),
            remote_url: default_remote_url(),
            remote_pr: default_remote_pr(),
            remote_issue: default_remote_issue(),
            arrow: default_arrow(),
            tree_dir: default_tree_dir(),
            tree_file: default_tree_file(),
        }
    }
}

// Default values
fn default_modified() -> String { "#FF00FF".into() }
fn default_deleted() -> String { "#FF4444".into() }
fn default_new_file() -> String { "#00FF88".into() }
fn default_renamed() -> String { "#00FFFF".into() }
fn default_added() -> String { "#00FF88".into() }
fn default_untracked() -> String { "#AA55FF".into() }
fn default_staged() -> String { "#00FF88".into() }
fn default_not_staged() -> String { "#00FFFF".into() }
fn default_header() -> String { "#FFFF00".into() }
fn default_branch() -> String { "#00FFFF".into() }
fn default_up_to_date() -> String { "#FFFF00".into() }
fn default_ahead_behind() -> String { "#FFFF00".into() }
fn default_hint() -> String { String::new() }
fn default_cwd_label() -> String { "#0055FF".into() }
fn default_cwd_path() -> String { "#FFAAFF".into() }
fn default_remote_url() -> String { "#00FFFF".into() }
fn default_remote_pr() -> String { "#00FF88".into() }
fn default_remote_issue() -> String { "#FFAA00".into() }
fn default_arrow() -> String { "#FFFFFF".into() }
fn default_tree_dir() -> String { "#00FFFF".into() }
fn default_tree_file() -> String { "#FFFFFF".into() }
fn default_tree_mode() -> bool { true }

impl AppConfig {
    pub fn load() -> Result<Self> {
        let config_path = Self::find_config_path()?;
        
        if !config_path.exists() {
            tracing::info!("No config file found, using defaults");
            return Ok(Self::default());
        }

        tracing::info!("Loading config from: {:?}", config_path);
        let content = std::fs::read_to_string(&config_path)
            .context("Failed to read config file")?;
        
        let config: AppConfig = toml::from_str(&content)
            .context("Failed to parse config file")?;
        
        Ok(config)
    }

    fn find_config_path() -> Result<PathBuf> {
        // Check XDG_CONFIG_HOME first
        if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
            let path = PathBuf::from(xdg_config)
                .join("gits")
                .join("config.toml");
            if path.exists() {
                return Ok(path);
            }
        }

        // Check ~/.config/gits/config.toml
        if let Some(config_dir) = dirs::config_dir() {
            let path = config_dir.join("gits").join("config.toml");
            if path.exists() {
                return Ok(path);
            }
        }

        // Check ~/.gits.toml
        if let Some(home_dir) = dirs::home_dir() {
            let path = home_dir.join(".gits.toml");
            if path.exists() {
                return Ok(path);
            }
            
            // Default creation path
            return Ok(home_dir.join(".gits.toml"));
        }

        Err(anyhow::anyhow!("Cannot determine home directory").into())
    }

    pub fn dump(&self) -> Result<String> {
        toml::to_string_pretty(self)
            .map_err(|e| e.into())
    }
}