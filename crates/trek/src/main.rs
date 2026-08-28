use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;
#[cfg(feature = "generate")]
mod scaffolding;
mod styles;

#[derive(Parser)]
#[command(name = "trek", version, about = styles::desc("A Trek Tool for FiveM dev."),styles = styles::get_styles()) ]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[cfg(feature = "generate")]
    /// Generate a new FiveM resource scaffold
    Generate {
        /// Resource name
        #[arg(short, long)]
        name: String,

        /// Resource description
        #[arg(short, long, default_value = "A FiveM resource")]
        description: String,

        /// Frameworks to include: ESX, QBCore, Qbox, None
        #[arg(short, long, num_args = 1..)]
        frameworks: Vec<String>,
    },
    /// Pack a FiveM resource into a zip archive
    Pack {
        /// Output directory for the zip file
        #[arg(short, default_value = ".")]
        out_dir: PathBuf,

        /// Output report in markdown format
        #[arg(long)]
        report: bool,

        /// Perform a dry run without creating the archive
        #[arg(long)]
        dry_run: bool,

        /// Print SHA-256 checksum of the archive and include it in the report
        #[arg(long)]
        sha256: bool,
    },
    /// Full release pipeline: validate, bump and pack the resource
    Release {
        /// Bump major version before packing (e.g., 1.2.3 -> 2.0.0)
        #[arg(long, group = "release_bump_type")]
        major: bool,

        /// Bump minor version before packing (e.g., 1.2.3 -> 1.3.0)
        #[arg(long, group = "release_bump_type")]
        minor: bool,

        /// Bump patch version before packing (e.g., 1.2.3 -> 1.2.4)
        #[arg(long, group = "release_bump_type")]
        patch: bool,

        /// Output directory for the zip archive
        #[arg(short, default_value = ".")]
        out_dir: PathBuf,

        /// Path to fxmanifest.lua
        #[arg(short = 'm', long, default_value = "fxmanifest.lua")]
        manifest: PathBuf,

        /// Print SHA-256 checksum of the archive and include it in the report
        #[arg(long)]
        sha256: bool,
    },
    /// Lint fxmanifest.lua for common problems
    Validate {
        /// Path to fxmanifest.lua
        #[arg(short, long, default_value = "fxmanifest.lua")]
        manifest: PathBuf,
    },
    /// Show or bump resource version in fxmanifest.lua according to SemVer
    Version {
        /// Bump major version (e.g., 1.2.3 -> 2.0.0)
        #[arg(long, group = "bump_type")]
        major: bool,

        /// Bump minor version (e.g., 1.2.3 -> 1.3.0)
        #[arg(long, group = "bump_type")]
        minor: bool,

        /// Bump patch version (e.g., 1.2.3 -> 1.2.4)
        #[arg(long, group = "bump_type")]
        patch: bool,

        /// Print only the raw version (machine-readable)
        #[arg(long)]
        ci: bool,

        /// Path to fxmanifest.lua
        #[arg(short, long, default_value = "fxmanifest.lua")]
        manifest: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        #[cfg(feature = "generate")]
        Commands::Generate {
            name,
            description,
            frameworks,
        } => {
            commands::generate::run(name, description, frameworks)?;
        }
        Commands::Pack {
            out_dir,
            report,
            dry_run,
            sha256,
        } => {
            commands::pack::run(out_dir, *report, *dry_run, *sha256)?;
        }
        Commands::Release {
            major,
            minor,
            patch,
            out_dir,
            manifest,
            sha256,
        } => {
            let bump_type = if *major {
                Some(commands::version::BumpType::Major)
            } else if *minor {
                Some(commands::version::BumpType::Minor)
            } else if *patch {
                Some(commands::version::BumpType::Patch)
            } else {
                None
            };
            commands::release::run(manifest, out_dir, bump_type, *sha256)?;
        }
        Commands::Validate { manifest } => {
            if commands::validate::run(manifest)? {
                std::process::exit(1);
            }
        }
        Commands::Version {
            major,
            minor,
            patch,
            ci,
            manifest,
        } => {
            let bump_type = if *major {
                Some(commands::version::BumpType::Major)
            } else if *minor {
                Some(commands::version::BumpType::Minor)
            } else if *patch {
                Some(commands::version::BumpType::Patch)
            } else {
                None
            };
            commands::version::run(manifest, bump_type, *ci)?;
        }
    }

    Ok(())
}
