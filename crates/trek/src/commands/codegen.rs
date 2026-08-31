use std::{fs, path::Path};

use anyhow::{Context, Result};
use trek_codegen::{
    generators::{lua::generate_lua, typescript::generate_typescript},
    validation::validate_schema,
    write_output,
};

pub fn run(schema_path: &Path, ts_out: &Path, lua_out: &Path, init_schema: bool) -> Result<()> {
    if init_schema {
        return trek_codegen::initialize_schema(schema_path);
    }

    let input = fs::read_to_string(schema_path)
        .with_context(|| format!("failed to read schema '{}'", schema_path.display()))?;
    let schema: trek_codegen::schema::NuiSchema = serde_yml::from_str(&input)
        .with_context(|| format!("failed to parse schema '{}'", schema_path.display()))?;
    validate_schema(&schema)?;
    write_output(ts_out, &generate_typescript(&schema))?;
    write_output(lua_out, &generate_lua(&schema))?;
    println!(
        "Generated '{}' and '{}'.",
        ts_out.display(),
        lua_out.display()
    );
    Ok(())
}
