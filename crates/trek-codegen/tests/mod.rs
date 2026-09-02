use trek_codegen::generators::lua::generate_lua;
use trek_codegen::generators::typescript::generate_typescript;
use trek_codegen::schema::NuiSchema;
use trek_codegen::validation::validate_schema;

#[test]
fn test_imports_events_only() {
    let schema: NuiSchema = serde_yml::from_str(
        "version: \"1.0\"\nresource: test\nevents:\n  - name: speedUpdated\n    payload: { speed: number }\n",
    ).unwrap();

    let ts = generate_typescript(&schema);
    assert!(ts.contains("import { useNUIEvent } from \"@trekscripts/core/query\";"));
    assert!(!ts.contains("useNUIQuery"));
    assert!(!ts.contains("useNUIMutation"));
    assert!(!ts.contains("NUIQueryOptions"));
}

#[test]
fn test_imports_query_only() {
    let schema: NuiSchema = serde_yml::from_str(
        "version: \"1.0\"\nresource: test\nendpoints:\n  - name: getInventory\n    type: query\n    response: { items: [string] }\n",
    ).unwrap();

    let ts = generate_typescript(&schema);
    assert!(
        ts.contains("import { useNUIQuery, NUIQueryOptions } from \"@trekscripts/core/query\";")
    );
    assert!(!ts.contains("useNUIMutation"));
    assert!(!ts.contains("useNUIEvent"));
}

#[test]
fn test_imports_mutation_only() {
    let schema: NuiSchema = serde_yml::from_str(
        "version: \"1.0\"\nresource: test\nendpoints:\n  - name: useItem\n    type: mutation\n    request: { item: string }\n    response: { success: boolean }\n",
    ).unwrap();

    let ts = generate_typescript(&schema);
    assert!(
        ts.contains("import { useNUIMutation, NUIQueryOptions } from \"@trekscripts/core/query\";")
    );
    assert!(!ts.contains("useNUIQuery"));
    assert!(!ts.contains("useNUIEvent"));
}

#[test]
fn test_imports_empty_schema() {
    let schema: NuiSchema = serde_yml::from_str("version: \"1.0\"\nresource: test\n").unwrap();

    let ts = generate_typescript(&schema);
    assert!(!ts.contains("import"));
    assert!(!ts.contains("@trekscripts/core/query"));
}

#[test]
fn test_imports_all_combined() {
    let schema: NuiSchema = serde_yml::from_str(
        r#"
version: "1.0"
resource: test
events:
  - name: onAlert
    payload: { msg: string }
endpoints:
  - name: getData
    type: query
    response: { ok: boolean }
  - name: doAction
    type: mutation
    response: { ok: boolean }
"#,
    )
    .unwrap();

    let ts = generate_typescript(&schema);
    assert!(ts.contains("import { useNUIQuery, useNUIMutation, useNUIEvent, NUIQueryOptions } from \"@trekscripts/core/query\";"));
}

#[test]
fn test_enum_typescript_generation() {
    let yaml = r#"
version: "1.0"
resource: test
enums:
  - name: PlayerState
    description: "Current player state"
    values:
      - idle
      - in_combat
      - fast-travel
      - DEAD
      - 1st_rank
events:
  - name: stateChanged
    payload:
      current: PlayerState
      previous: PlayerState?
      history: [PlayerState]
"#;
    let schema: NuiSchema = serde_yml::from_str(yaml).unwrap();
    validate_schema(&schema).unwrap();

    let ts = generate_typescript(&schema);
    assert!(ts.contains("export enum PlayerState {"));
    assert!(ts.contains("  Idle = \"idle\","));
    assert!(ts.contains("  InCombat = \"in_combat\","));
    assert!(ts.contains("  FastTravel = \"fast-travel\","));
    assert!(ts.contains("  Dead = \"DEAD\","));
    assert!(ts.contains("  _1stRank = \"1st_rank\","));
    assert!(ts.contains("export interface StateChangedPayload {"));
    assert!(ts.contains("  current: PlayerState;"));
    assert!(ts.contains("  previous?: PlayerState;"));
    assert!(ts.contains("  history: Array<PlayerState>;"));
}

#[test]
fn test_enum_lua_generation() {
    let yaml = r#"
version: "1.0"
resource: test
enums:
  - name: ItemRarity
    description: "Item rarity tiers"
    values:
      - common
      - rare
      - epic
      - legendary
endpoints:
  - name: getItem
    type: query
    request:
      rarity: ItemRarity
    response:
      rarities: [ItemRarity]
"#;
    let schema: NuiSchema = serde_yml::from_str(yaml).unwrap();
    validate_schema(&schema).unwrap();

    let lua = generate_lua(&schema);
    assert!(lua.contains("--- Item rarity tiers\n---@enum ItemRarity\nNUI.ItemRarity = {"));
    assert!(lua.contains("    Common = \"common\","));
    assert!(lua.contains("    Rare = \"rare\","));
    assert!(lua.contains("    Epic = \"epic\","));
    assert!(lua.contains("    Legendary = \"legendary\","));
    assert!(lua.contains(
        "---@param handler fun(data: { rarity: ItemRarity }): { rarities: ItemRarity[] }"
    ));
    assert!(lua.contains("function NUI.onGetItem(handler)"));
}

#[test]
fn test_validation_unsupported_type_fails() {
    let yaml = r#"
version: "1.0"
resource: test
events:
  - name: badEvent
    payload: { state: UnknownState }
"#;
    let schema: NuiSchema = serde_yml::from_str(yaml).unwrap();
    let result = validate_schema(&schema);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("unsupported type 'UnknownState'")
    );
}

#[test]
fn test_validation_duplicate_enum_name_fails() {
    let yaml = r#"
version: "1.0"
resource: test
enums:
  - name: State
    values: [a, b]
  - name: State
    values: [c, d]
"#;
    let schema: NuiSchema = serde_yml::from_str(yaml).unwrap();
    let result = validate_schema(&schema);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("duplicate enum name 'State'")
    );
}

#[test]
fn test_validation_enum_primitive_name_conflict_fails() {
    let yaml = r#"
version: "1.0"
resource: test
enums:
  - name: string
    values: [a, b]
"#;
    let schema: NuiSchema = serde_yml::from_str(yaml).unwrap();
    let result = validate_schema(&schema);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("conflicts with built-in primitive type")
    );
}

#[test]
fn test_validation_empty_enum_values_fails() {
    let yaml = r#"
version: "1.0"
resource: test
enums:
  - name: EmptyEnum
    values: []
"#;
    let schema: NuiSchema = serde_yml::from_str(yaml).unwrap();
    let result = validate_schema(&schema);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("must have at least one value")
    );
}

#[test]
fn test_validation_duplicate_enum_values_fails() {
    let yaml = r#"
version: "1.0"
resource: test
enums:
  - name: Status
    values: [active, active]
"#;
    let schema: NuiSchema = serde_yml::from_str(yaml).unwrap();
    let result = validate_schema(&schema);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("duplicate value 'active'")
    );
}
