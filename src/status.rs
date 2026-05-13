use std::path::PathBuf;
use std::process::Command;

use regex::Regex;

use crate::colors::ColorResolver;
use crate::config::AppConfig;
use crate::error::Result;
use crate::icons;
use crate::tree::TreeRenderer;

pub struct Status {
    config: AppConfig,
    colors: ColorResolver,
}

impl Status {
    pub fn new(config: AppConfig) -> Self {
        let colors = ColorResolver::new(config.colors.clone());
        Self { config, colors }
    }

    pub fn colorize_git_status(&self, cwd: &str) -> Result<bool> {
        // Print current directory
        let cwd_path = std::fs::canonicalize(cwd)
            .unwrap_or_else(|_| PathBuf::from(cwd));
        
        print!("{} ", icons::Icons::FOLDER);
        print!("chdir: ");
        print!("{}", colored::Colorize::color(
            cwd_path.display().to_string().as_str(),
            self.colors.cwd_path().unwrap_or(colored::Color::White)
        ));
        println!();

        // Run git status
        let output = Command::new("git")
            .args(["-c", "color.status=never", "status"])
            .current_dir(cwd)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("{} {}", icons::Icons::ERROR, stderr);
            return Ok(false);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().collect();

        // Compile regexes outside loop
        let branch_re = Regex::new(r"^On branch (.+)$").unwrap();
        let header_patterns = [
            (Regex::new(r"^\s*Changes to be committed:").unwrap(), "staged"),
            (Regex::new(r"^\s*Changes not staged for commit:").unwrap(), "not_staged"),
            (Regex::new(r"^\s*Untracked files:").unwrap(), "untracked"),
        ];
        let file_re = Regex::new(r"^(\s*)(modified|deleted|new file|renamed|added):\s+(.+)$").unwrap();
        let indent_re = Regex::new(r"^(\s+)(.+)$").unwrap();

        let mut context = "";
        let mut untracked_files: Vec<String> = Vec::new();
        let mut in_untracked = false;

        for line in &lines {
            let line = line.trim_end_matches('\r');

            // Branch line
            if let Some(caps) = branch_re.captures(line) {
                let branch = caps.get(1).unwrap().as_str();
                print!("{} On branch ", icons::Icons::INFO);
                print!("{} {}", icons::Icons::GIT, colored::Colorize::color(
                    branch,
                    self.colors.branch().unwrap_or(colored::Color::Cyan)
                ));
                println!();
                context = "";
                in_untracked = false;
                continue;
            }

            // HEAD detached
            if line.contains("HEAD detached") {
                println!("{} {}", icons::Icons::WARNING, line);
                continue;
            }

            // No commits yet
            if line.contains("No commits yet") {
                println!("{} {}", icons::Icons::WARNING, line);
                continue;
            }

            // Up to date
            if line.contains("Your branch is up to date") {
                println!("{} {}", icons::Icons::SUCCESS, line);
                context = "";
                in_untracked = false;
                continue;
            }

            // Ahead/behind/diverged
            if line.contains("ahead") || line.contains("behind") || line.contains("diverged") {
                println!("{}", line);
                context = "";
                in_untracked = false;
                continue;
            }

            // Headers
            let mut found_header = false;
            for (re, header_type) in &header_patterns {
                if re.is_match(line) {
                    if in_untracked && self.config.tree_mode {
                        let renderer = TreeRenderer::new(ColorResolver::new(self.config.colors.clone()));
                        renderer.render_untracked(&untracked_files, cwd)?;
                        untracked_files.clear();
                    }
                    println!("    {}", colored::Colorize::color(
                        line,
                        self.colors.header().unwrap_or(colored::Color::Yellow)
                    ));
                    context = header_type;
                    in_untracked = *header_type == "untracked";
                    found_header = true;
                    break;
                }
            }
            if found_header {
                continue;
            }

            // Hints
            if line.trim().starts_with("(use \"git ") {
                if in_untracked && self.config.tree_mode {
                    continue;
                }
                println!("    {}", colored::Colorize::color(line.trim(), colored::Color::BrightBlack));
                continue;
            }

            // Terminal status lines
            let lower = line.trim().to_lowercase();
            let is_terminal = lower.starts_with("nothing to commit") ||
                lower.starts_with("nothing added to commit") ||
                lower.starts_with("no changes added to commit") ||
                lower.contains("clean working tree");

            if is_terminal {
                if in_untracked && self.config.tree_mode {
                    let renderer = TreeRenderer::new(ColorResolver::new(self.config.colors.clone()));
                    renderer.render_untracked(&untracked_files, cwd)?;
                    untracked_files.clear();
                    in_untracked = false;
                }
                println!("{} {}", icons::Icons::SUCCESS, line);
                context = "";
                continue;
            }

            // Collect untracked files
            if context == "untracked" && self.config.tree_mode {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    untracked_files.push(trimmed.to_string());
                }
                continue;
            }

            // Normal file line
            self.print_file_line_with_regex(line, context, &file_re, &indent_re);
        }

        // Flush remaining untracked
        if in_untracked && self.config.tree_mode && !untracked_files.is_empty() {
            let renderer = TreeRenderer::new(ColorResolver::new(self.config.colors.clone()));
            renderer.render_untracked(&untracked_files, cwd)?;
        }

        Ok(true)
    }

    fn print_file_line_with_regex(&self, line: &str, context: &str, file_re: &Regex, indent_re: &Regex) {
        if let Some(caps) = file_re.captures(line) {
            let indent = caps.get(1).unwrap().as_str();
            let status = caps.get(2).unwrap().as_str();
            let rest = caps.get(3).unwrap().as_str();

            print!("{}", indent);
            print!("      {}", status);
            print!(": ");

            if rest.contains("->") {
                let parts: Vec<&str> = rest.split("->").collect();
                let left = parts[0].trim();
                let right = parts[1].trim();
                
                let status_color = self.get_status_color(status);
                print!("{}", colored::Colorize::color(left, status_color));
                print!(" -> ");
                print!("{}", colored::Colorize::color(right, self.colors.renamed().unwrap_or(colored::Color::Cyan)));
            } else {
                let status_color = self.get_status_color(status);
                print!("{}", colored::Colorize::color(rest, status_color));
            }
            println!();
        } else if let Some(caps) = indent_re.captures(line) {
            let indent = caps.get(1).unwrap().as_str();
            let payload = caps.get(2).unwrap().as_str();
            
            print!("{}", indent);
            print!("      ");
            
            let color = match context {
                "untracked" => self.colors.untracked(),
                "staged" => self.colors.staged(),
                "not_staged" => self.colors.not_staged(),
                _ => None,
            };
            
            if let Some(color) = color {
                print!("{}", colored::Colorize::color(payload, color));
            } else {
                print!("{}", payload);
            }
            println!();
        } else {
            println!("{}", line);
        }
    }

    fn get_status_color(&self, status: &str) -> colored::Color {
        match status {
            "modified" => self.colors.modified().unwrap_or(colored::Color::Magenta),
            "deleted" => self.colors.deleted().unwrap_or(colored::Color::Red),
            "new file" => self.colors.new_file().unwrap_or(colored::Color::Green),
            "renamed" => self.colors.renamed().unwrap_or(colored::Color::Cyan),
            "added" => self.colors.added().unwrap_or(colored::Color::Green),
            _ => colored::Color::White,
        }
    }
}