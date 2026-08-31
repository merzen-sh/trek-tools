use trek_codegen::generators::typescript::generate_typescript;
use trek_codegen::schema::NuiSchema;

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
