use std::path::Path;
use std::time::Instant;

use anyhow::Result;
use console::style;

use crate::scaffolding::Scaffolding;

const FRAMEWORK_OPTIONS: &[&str] = &["ESX", "QBCore", "Qbox", "None"];

pub fn run(name: &str, description: &str, frameworks: &[String]) -> Result<()> {
    let selected_frameworks: Vec<&str> = frameworks
        .iter()
        .filter_map(|f| {
            let lower = f.to_lowercase();
            FRAMEWORK_OPTIONS
                .iter()
                .find(|opt| opt.to_lowercase() == lower)
                .copied()
        })
        .collect();

    let scaffold = Scaffolding::new(name, description, &selected_frameworks);

    let start = Instant::now();
    scaffold.generate_all(Path::new("."))?;
    let elapsed_ms = start.elapsed().as_millis();

    println!(
        "\n{} Resource '{}' generated successfully! ({}ms)",
        style("✓").green().bold(),
        name,
        elapsed_ms
    );

    Ok(())
}
