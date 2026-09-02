use crate::schema::{EnumDefinition, NuiSchema, TypeDefinition};
use anyhow::{Result, bail};
use std::collections::HashSet;

pub fn validate_schema(schema: &NuiSchema) -> Result<()> {
    if schema.version.trim().is_empty() || schema.resource.trim().is_empty() {
        bail!("schema version and resource must not be empty");
    }

    let mut enum_names = HashSet::new();
    for enum_def in &schema.enums {
        validate_enum(enum_def, &mut enum_names)?;
    }

    for event in &schema.events {
        validate_name(&event.name, "event")?;
        validate_type(&event.payload, &enum_names)?;
    }
    for endpoint in &schema.endpoints {
        validate_name(&endpoint.name, "endpoint")?;
        if let Some(request) = &endpoint.request {
            validate_type(request, &enum_names)?;
        }
        validate_type(&endpoint.response, &enum_names)?;
    }
    Ok(())
}

fn validate_name(name: &str, kind: &str) -> Result<()> {
    if name.trim().is_empty() {
        bail!("{kind} name must not be empty");
    }
    Ok(())
}

fn validate_enum(enum_def: &EnumDefinition, known_enums: &mut HashSet<String>) -> Result<()> {
    validate_name(&enum_def.name, "enum")?;
    let name = enum_def.name.trim();
    match name {
        "string" | "number" | "boolean" | "void" => {
            bail!("enum name '{name}' conflicts with built-in primitive type");
        }
        _ => {}
    }
    if !known_enums.insert(name.to_string()) {
        bail!("duplicate enum name '{name}'");
    }
    if enum_def.values.is_empty() {
        bail!("enum '{name}' must have at least one value");
    }
    let mut values = HashSet::new();
    for val in &enum_def.values {
        if val.trim().is_empty() {
            bail!("enum '{name}' contains an empty value");
        }
        if !values.insert(val.trim()) {
            bail!("duplicate value '{val}' in enum '{name}'");
        }
    }
    Ok(())
}

fn validate_type(ty: &TypeDefinition, known_enums: &HashSet<String>) -> Result<()> {
    match ty {
        TypeDefinition::Primitive(value) => {
            let base = value.trim_end_matches('?');
            match base {
                "string" | "number" | "boolean" => Ok(()),
                custom if known_enums.contains(custom) => Ok(()),
                other => bail!(
                    "unsupported type '{other}'; expected string, number, boolean, or a defined enum"
                ),
            }
        }
        TypeDefinition::Array(items) => {
            if items.len() != 1 {
                bail!("array types must contain exactly one item type");
            }
            validate_type(&items[0], known_enums)
        }
        TypeDefinition::Object(fields) => fields
            .values()
            .try_for_each(|f| validate_type(f, known_enums)),
    }
}
