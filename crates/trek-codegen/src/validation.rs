use crate::schema::{NuiSchema, TypeDefinition};
use anyhow::{Result, bail};

pub fn validate_schema(schema: &NuiSchema) -> Result<()> {
    if schema.version.trim().is_empty() || schema.resource.trim().is_empty() {
        bail!("schema version and resource must not be empty");
    }
    for event in &schema.events {
        validate_name(&event.name, "event")?;
        validate_type(&event.payload)?;
    }
    for endpoint in &schema.endpoints {
        validate_name(&endpoint.name, "endpoint")?;
        if let Some(request) = &endpoint.request {
            validate_type(request)?;
        }
        validate_type(&endpoint.response)?;
    }
    Ok(())
}

fn validate_name(name: &str, kind: &str) -> Result<()> {
    if name.trim().is_empty() {
        bail!("{kind} name must not be empty");
    }
    Ok(())
}

fn validate_type(ty: &TypeDefinition) -> Result<()> {
    match ty {
        TypeDefinition::Primitive(value) => match value.trim_end_matches('?') {
            "string" | "number" | "boolean" => Ok(()),
            other => bail!("unsupported type '{other}'; expected string, number, or boolean"),
        },
        TypeDefinition::Array(items) => {
            if items.len() != 1 {
                bail!("array types must contain exactly one item type");
            }
            validate_type(&items[0])
        }
        TypeDefinition::Object(fields) => fields.values().try_for_each(validate_type),
    }
}
