use std::path::Path;
use std::time::Instant;

use anyhow::{Ok, Result, bail};
use dialoguer::console::{Style, style};
use dialoguer::{Confirm, Input, MultiSelect, theme::ColorfulTheme};

use crate::scaffolding::Scaffolding;

const FRAMEWORK_OPTIONS: &[&str] = &["ESX", "QBCore", "Qbox", "None"];

pub fn run(
    name: Option<&str>,
    description: Option<&str>,
    frameworks: &[String],
    no_ui: bool,
    install: bool,
) -> Result<()> {
    let interactive = name.is_none();

    if install && no_ui {
        bail!("--install requires the web UI; remove --no-ui or drop --install");
    }

    let theme = ColorfulTheme {
        active_item_prefix: style(">> ".to_owned()).yellow(),
        checked_item_prefix: style("[x]".to_owned()).green().bold(),
        unchecked_item_prefix: style("[ ]".to_owned()),
        active_item_style: Style::new().green().bold(),
        inactive_item_style: Style::new().dim(),
        prompt_prefix: style("›".to_owned()).green(),
        success_prefix: style("✓".to_owned()).green(),
        ..ColorfulTheme::default()
    };

    let resource_name = match name {
        Some(n) => n.to_string(),
        None => Input::with_theme(&theme)
            .with_prompt("Enter Resource Name")
            .interact_text()?,
    };

    let description = match description {
        Some(d) => d.to_string(),
        None if interactive => Input::with_theme(&theme)
            .with_prompt("Enter Description")
            .default(format!("A FiveM resource for {}", resource_name))
            .interact_text()?,
        None => format!("A FiveM resource for {}", resource_name),
    };

    let selected_frameworks: Vec<&str> = if !frameworks.is_empty() {
        frameworks
            .iter()
            .filter_map(|f| {
                let lower = f.to_lowercase();
                FRAMEWORK_OPTIONS
                    .iter()
                    .find(|opt| opt.to_lowercase() == lower)
                    .copied()
            })
            .collect()
    } else if interactive {
        let mut defaults = vec![true; FRAMEWORK_OPTIONS.len()];
        if let Some(last) = defaults.last_mut() {
            *last = false;
        }

        let chosen_indices = MultiSelect::with_theme(&theme)
            .with_prompt("Choose your framework")
            .items(FRAMEWORK_OPTIONS)
            .defaults(&defaults)
            .interact()?;

        chosen_indices
            .into_iter()
            .map(|i| FRAMEWORK_OPTIONS[i])
            .collect()
    } else {
        vec!["None"]
    };

    let include_ui = if no_ui || !interactive {
        false
    } else {
        Confirm::with_theme(&theme)
            .with_prompt("Do you want to install web UI (Vite + Preact)")
            .default(true)
            .interact()?
    };

    let scaffold = Scaffolding::new(
        &resource_name,
        &description,
        &selected_frameworks,
        include_ui,
        install,
    );

    let start = Instant::now();
    scaffold.generate_all(Path::new("."))?;
    let elapsed_ms = start.elapsed().as_millis();

    println!(
        "\n{} Resource '{}' generated successfully! ({}ms)",
        style("✓").green().bold(),
        resource_name,
        elapsed_ms
    );

    Ok(())
}
