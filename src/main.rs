#[cfg(feature = "cli")]
mod cli {
    use clap::{Parser, Subcommand};
    use config_get::{get_config_file, ConfigGet};

    #[derive(Parser)]
    #[command(
        name = "config-get",
        about = "Cross-platform configuration file locator and reader",
        version,
        author = "Hadi Cahyadi <cumulus13@gmail.com>"
    )]
    struct Cli {
        #[command(subcommand)]
        command: Commands,
    }

    #[derive(Subcommand)]
    enum Commands {
        /// Find and print the path of a config file.
        Find {
            /// Config stem (e.g. "myapp")
            stem: String,
            /// Sub-directory to search in
            #[arg(short, long)]
            dir: Option<String>,
        },

        /// Print all key/value pairs from a config file.
        Dump {
            /// Config stem or explicit file path
            stem: String,
            #[arg(short, long)]
            dir: Option<String>,
            /// Treat `stem` as an explicit file path
            #[arg(short, long)]
            file: bool,
        },

        /// Get a single value from a config file.
        Get {
            /// Config stem or explicit file path
            stem: String,
            /// Key to look up (use `section.key` for sectioned configs)
            key: String,
            #[arg(short, long)]
            dir: Option<String>,
            #[arg(short, long)]
            file: bool,
            /// Value to print if key is absent
            #[arg(long)]
            fallback: Option<String>,
        },

        /// Print candidate search paths without loading.
        Paths {
            stem: String,
            #[arg(short, long)]
            dir: Option<String>,
        },
    }

    pub fn run() {
        let cli = Cli::parse();

        match cli.command {
            Commands::Find { stem, dir } => {
                let d = dir.as_deref().unwrap_or(&stem);
                if let Some(p) = get_config_file(&stem, d) {
                    println!("{}", p.display());
                } else {
                    eprintln!("No config file found for '{stem}'");
                    std::process::exit(1);
                }
            }

            Commands::Dump { stem, dir, file } => {
                let cfg = load_cfg(&stem, dir.as_deref(), file);
                for (k, v) in cfg.iter() {
                    println!("{k}={v}");
                }
                for section in cfg.sections() {
                    if let Ok(kv) = cfg.get_section(section) {
                        for (k, v) in &kv {
                            println!("[{section}] {k}={v}");
                        }
                    }
                }
            }

            Commands::Get {
                stem,
                key,
                dir,
                file,
                fallback,
            } => {
                let cfg = load_cfg(&stem, dir.as_deref(), file);
                let value = if key.contains('.') {
                    let mut parts = key.splitn(2, '.');
                    let section = parts.next().unwrap_or_default();
                    let k = parts.next().unwrap_or_default();
                    cfg.get_in(section, k).or_else(|| cfg.get(&key))
                } else {
                    cfg.get(&key)
                };
                if let Some(v) = value.or(fallback.as_deref()) {
                    println!("{v}");
                } else {
                    eprintln!("Key '{key}' not found");
                    std::process::exit(1);
                }
            }

            Commands::Paths { stem, dir } => {
                let d = dir.as_deref().unwrap_or(&stem);
                for p in ConfigGet::search_paths(&stem, d) {
                    let exists = if p.exists() { " [EXISTS]" } else { "" };
                    println!("{}{exists}", p.display());
                }
            }
        }
    }

    fn load_cfg(stem: &str, dir: Option<&str>, is_file: bool) -> ConfigGet {
        let result = if is_file {
            ConfigGet::from_file(stem)
        } else {
            ConfigGet::builder(stem)
                .config_dir(dir.unwrap_or(stem))
                .build()
        };
        result.unwrap_or_else(|e| {
            eprintln!("Error: {e}");
            std::process::exit(1);
        })
    }
}

fn main() {
    #[cfg(feature = "cli")]
    {
        log::set_max_level(log::LevelFilter::Info);
        cli::run();
    }

    #[cfg(not(feature = "cli"))]
    {
        eprintln!("Recompile with `--features cli` to enable the CLI binary.");
        std::process::exit(1);
    }
}
