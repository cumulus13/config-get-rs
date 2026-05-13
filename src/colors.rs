use colored::Color;

use crate::config::ColorConfig;

pub struct ColorResolver {
    config: ColorConfig,
}

impl ColorResolver {
    pub fn new(config: ColorConfig) -> Self {
        Self { config }
    }

    pub fn resolve(&self, hex: &str) -> Option<Color> {
        if hex.is_empty() {
            return None;
        }
        hex_to_color(hex)
    }

    pub fn modified(&self) -> Option<Color> {
        self.resolve(&self.config.modified)
    }

    pub fn deleted(&self) -> Option<Color> {
        self.resolve(&self.config.deleted)
    }

    pub fn new_file(&self) -> Option<Color> {
        self.resolve(&self.config.new_file)
    }

    pub fn renamed(&self) -> Option<Color> {
        self.resolve(&self.config.renamed)
    }

    pub fn added(&self) -> Option<Color> {
        self.resolve(&self.config.added)
    }

    pub fn untracked(&self) -> Option<Color> {
        self.resolve(&self.config.untracked)
    }

    pub fn staged(&self) -> Option<Color> {
        self.resolve(&self.config.staged)
    }

    pub fn not_staged(&self) -> Option<Color> {
        self.resolve(&self.config.not_staged)
    }

    pub fn header(&self) -> Option<Color> {
        self.resolve(&self.config.header)
    }

    pub fn branch(&self) -> Option<Color> {
        self.resolve(&self.config.branch)
    }

    pub fn cwd_path(&self) -> Option<Color> {
        self.resolve(&self.config.cwd_path)
    }

    pub fn tree_dir(&self) -> Option<Color> {
        self.resolve(&self.config.tree_dir)
    }

    pub fn tree_file(&self) -> Option<Color> {
        self.resolve(&self.config.tree_file)
    }
}

fn hex_to_color(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');
    
    if hex.len() == 3 {
        let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
        let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
        let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
        Some(Color::TrueColor { r, g, b })
    } else if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(Color::TrueColor { r, g, b })
    } else {
        None
    }
}