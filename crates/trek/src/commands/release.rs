use std::path::Path;
use std::time::Instant;

use anyhow::{Result, bail};
use console::style;

use crate::commands;
use crate::commands::version::BumpType;

pub fn run(
    manifest: &Path,
    out_dir: &Path,
    bump_type: Option<BumpType>,
    sha256: bool,
) -> Result<()> {
    let start = Instant::now();

    println!(
        "{} Starting release pipeline for '{}'...",
        style("→").cyan().bold(),
        manifest.display()
    );

    if commands::validate::run(manifest)? {
        bail!("Validation failed; release aborted.");
    }

    match bump_type {
        Some(bump_type) => commands::version::run(manifest, Some(bump_type), false)?,
        None => println!(
            "{} No bump flag given, keeping current version",
            style("•").cyan().bold()
        ),
    }

    commands::pack::run(out_dir, true, false, sha256)?;

    println!(
        "{} Release completed! ({}ms)",
        style("✓").green().bold(),
        start.elapsed().as_millis()
    );

    Ok(())
}
