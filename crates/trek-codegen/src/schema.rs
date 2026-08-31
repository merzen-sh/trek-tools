use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Root configuration schema for NUI (Network User Interface) contracts.
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
#[serde(deny_unknown_fields)]
pub struct NuiSchema {
    pub version: String,
    pub resource: String,
    #[serde(default)]
    pub events: Vec<EventDefinition>,
    #[serde(default)]
    pub endpoints: Vec<EndpointDefinition>,
}

/// Definition structure for one-way UI events.
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct EventDefinition {
    pub name: String,
    pub description: Option<String>,
    pub payload: TypeDefinition,
}

/// Definition structure for two-way NUI callback endpoints.
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct EndpointDefinition {
    pub name: String,
    #[serde(rename = "type")]
    pub endpoint_type: EndpointType,
    pub description: Option<String>,
    #[serde(default)]
    pub request: Option<TypeDefinition>,
    pub response: TypeDefinition,
}

/// Classification of RPC endpoints used to drive code generation.
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EndpointType {
    Query,
    Mutation,
}

/// Polymorphic representation of payload field types.
#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
#[serde(untagged)]
pub enum TypeDefinition {
    Primitive(String),
    Array(Vec<TypeDefinition>),
    Object(BTreeMap<String, TypeDefinition>),
}

impl TypeDefinition {
    pub fn is_optional(&self) -> bool {
        matches!(self, TypeDefinition::Primitive(value) if value.ends_with('?'))
    }
}
