use anyhow::{Context, Result, bail};
use schemars::schema_for;
use std::fs;
use std::path::Path;

pub mod generators;
pub mod schema;
pub mod validation;

pub const STARTER_SCHEMA: &str = "# yaml-language-server: $schema=https://raw.githubusercontent.com/merzen-sh/trek-tools/refs/heads/main/trek-nui.schema.json\n\nversion: \"1.0\"\nresource: \"my_resource\"\nevents: []\nendpoints: []\n";

pub fn initialize_schema(schema_path: &Path) -> Result<()> {
    if schema_path.exists() {
        bail!("schema '{}' already exists", schema_path.display());
    }
    let json_path = schema_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("trek-nui.schema.json");
    if json_path.exists() {
        bail!("JSON Schema '{}' already exists", json_path.display());
    }
    write_output(schema_path, STARTER_SCHEMA)?;
    let json = serde_json::to_string_pretty(&schema_for!(crate::schema::NuiSchema))?;
    write_output(&json_path, &format!("{json}\n"))?;
    println!(
        "Created '{}' and '{}'.",
        schema_path.display(),
        json_path.display()
    );
    Ok(())
}

pub fn write_output(path: &Path, contents: &str) -> Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create output directory '{}'", parent.display()))?;
    }
    fs::write(path, contents).with_context(|| format!("failed to write '{}'", path.display()))
}
