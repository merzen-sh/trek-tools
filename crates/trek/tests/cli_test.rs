use std::fs;
use std::process::Command;

#[test]
fn test_cli_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_trek"))
        .arg("--help")
        .output()
        .expect("Failed to execute trek");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("trek"));
    assert!(stdout.contains("generate"));
    assert!(stdout.contains("pack"));
}

#[test]
fn test_cli_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_trek"))
        .arg("--version")
        .output()
        .expect("Failed to execute trek");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0.1.0"));
}

#[test]
fn test_cli_codegen_init_schema() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let schema = temp_dir.path().join("nui-schema.yaml");

    let output = Command::new(env!("CARGO_BIN_EXE_trek"))
        .args(["codegen", "--init-schema", "-s", schema.to_str().unwrap()])
        .output()
        .expect("Failed to run trek codegen --init-schema");

    assert!(output.status.success());
    assert!(schema.exists());
    assert!(temp_dir.path().join("trek-nui.schema.json").exists());
}

#[test]
fn test_cli_generate_and_pack_workflow() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let temp_path = temp_dir.path();

    // 1. Generate scaffold
    let gen_output = Command::new(env!("CARGO_BIN_EXE_trek"))
        .current_dir(temp_path)
        .args([
            "generate",
            "-n",
            "integration_res",
            "-d",
            "Integration Test Resource",
            "-f",
            "ESX",
            "QBCore",
        ])
        .output()
        .expect("Failed to run trek generate");

    assert!(gen_output.status.success());
    let res_path = temp_path.join("integration_res");
    assert!(res_path.exists());
    assert!(res_path.join("fxmanifest.lua").exists());
    assert!(res_path.join(".pack").exists());

    // 2. Pack the generated resource
    let dist_dir = temp_path.join("dist");
    let pack_output = Command::new(env!("CARGO_BIN_EXE_trek"))
        .current_dir(&res_path)
        .args(["pack", "-o", dist_dir.to_str().unwrap()])
        .output()
        .expect("Failed to run trek pack");

    assert!(pack_output.status.success());
    let zip_path = dist_dir.join("integration_res.zip");
    assert!(zip_path.exists());

    // 3. Verify zip archive contents
    let zip_file = fs::File::open(&zip_path).expect("Failed to open generated zip");
    let archive = zip::ZipArchive::new(zip_file).expect("Failed to read zip archive");
    let files: Vec<String> = archive.file_names().map(|s| s.to_string()).collect();

    assert!(files.contains(&"fxmanifest.lua".to_string()));
    assert!(files.contains(&"src/client/client.lua".to_string()));
    assert!(files.contains(&"src/server/server.lua".to_string()));
    assert!(files.contains(&"src/shared/utils.lua".to_string()));
    assert!(files.contains(&"config/share.lua".to_string()));
    assert!(files.contains(&"config/client.lua".to_string()));
    assert!(files.contains(&"config/server.lua".to_string()));
}

#[test]
fn test_cli_pack_dry_run_and_report() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let temp_path = temp_dir.path();

    // 1. Generate scaffold
    let gen_output = Command::new(env!("CARGO_BIN_EXE_trek"))
        .current_dir(temp_path)
        .args(["generate", "-n", "dry_res", "-f", "None"])
        .output()
        .expect("Failed to run trek generate");

    assert!(gen_output.status.success());
    let res_path = temp_path.join("dry_res");

    // 2. Pack with --dry-run and --report
    let dist_dir = temp_path.join("dist");
    let pack_output = Command::new(env!("CARGO_BIN_EXE_trek"))
        .current_dir(&res_path)
        .args([
            "pack",
            "-o",
            dist_dir.to_str().unwrap(),
            "--dry-run",
            "--report",
        ])
        .output()
        .expect("Failed to run trek pack with dry-run");

    assert!(pack_output.status.success());
    let stdout = String::from_utf8_lossy(&pack_output.stdout);

    // Verify dry run output and timing in ms
    assert!(stdout.contains("[DRY RUN] Would pack"));
    assert!(stdout.contains("ms)"));

    // Verify markdown report
    assert!(stdout.contains("# Pack Report: dry_res"));
    assert!(stdout.contains("- **Status:** Dry Run (Simulated)"));
    assert!(stdout.contains("| `fxmanifest.lua` |"));

    // Verify zip was NOT created on disk due to --dry-run
    let zip_path = dist_dir.join("dry_res.zip");
    assert!(!zip_path.exists());
}

#[test]
fn test_cli_version_bump_patch_minor_major() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let manifest_path = temp_dir.path().join("fxmanifest.lua");

    fs::write(
        &manifest_path,
        "fx_version 'cerulean'\nversion '1.0.0'\nauthor 'Trek'",
    )
    .unwrap();

    // 1. Bump patch (1.0.0 -> 1.0.1)
    let output = Command::new(env!("CARGO_BIN_EXE_trek"))
        .args(["version", "--patch", "-m", manifest_path.to_str().unwrap()])
        .output()
        .expect("Failed to run trek version --patch");
    assert!(output.status.success());
    let content = fs::read_to_string(&manifest_path).unwrap();
    assert!(content.contains("version '1.0.1'"));

    // 2. Bump minor (1.0.1 -> 1.1.0)
    let output = Command::new(env!("CARGO_BIN_EXE_trek"))
        .args(["version", "--minor", "-m", manifest_path.to_str().unwrap()])
        .output()
        .expect("Failed to run trek version --minor");
    assert!(output.status.success());
    let content = fs::read_to_string(&manifest_path).unwrap();
    assert!(content.contains("version '1.1.0'"));

    // 3. Bump major (1.1.0 -> 2.0.0)
    let output = Command::new(env!("CARGO_BIN_EXE_trek"))
        .args(["version", "--major", "-m", manifest_path.to_str().unwrap()])
        .output()
        .expect("Failed to run trek version --major");
    assert!(output.status.success());
    let content = fs::read_to_string(&manifest_path).unwrap();
    assert!(content.contains("version '2.0.0'"));
}

#[test]
fn test_cli_version_show_current() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let manifest_path = temp_dir.path().join("fxmanifest.lua");
    fs::write(
        &manifest_path,
        "fx_version 'cerulean'\nversion '4.2.7'\nauthor 'Trek'",
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_trek"))
        .args(["version", "-m", manifest_path.to_str().unwrap()])
        .output()
        .expect("Failed to run trek version");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("4.2.7"));

    // File must remain untouched (no bump flags given)
    let content = fs::read_to_string(&manifest_path).unwrap();
    assert!(content.contains("version '4.2.7'"));
}

#[test]
fn test_cli_version_show_current_ci() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let manifest_path = temp_dir.path().join("fxmanifest.lua");
    fs::write(
        &manifest_path,
        "fx_version 'cerulean'\nversion '4.2.7+build.9'\nauthor 'Trek'",
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_trek"))
        .args(["version", "--ci", "-m", manifest_path.to_str().unwrap()])
        .output()
        .expect("Failed to run trek version --ci");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, "4.2.7+build.9\n");
}

#[test]
fn test_cli_validate_generated_resource_passes() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");

    let gen_output = Command::new(env!("CARGO_BIN_EXE_trek"))
        .current_dir(temp_dir.path())
        .args(["generate", "-n", "valid_res", "-f", "None"])
        .output()
        .expect("Failed to run trek generate");
    assert!(gen_output.status.success());

    let output = Command::new(env!("CARGO_BIN_EXE_trek"))
        .args([
            "validate",
            "-m",
            temp_dir
                .path()
                .join("valid_res/fxmanifest.lua")
                .to_str()
                .unwrap(),
        ])
        .output()
        .expect("Failed to run trek validate");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No issues found"), "{stdout}");
}

#[test]
fn test_cli_validate_reports_problems() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let manifest_path = temp_dir.path().join("fxmanifest.lua");

    fs::write(
        &manifest_path,
        r#"game 'gta5'
author 'Trek'

client_scripts {
    'missing_client.lua',
    'missing_client.lua',
}

shared_script '@es_extended/imports.lua'
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_trek"))
        .args(["validate", "-m", manifest_path.to_str().unwrap()])
        .output()
        .expect("Failed to run trek validate");

    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("fx_version"), "{stdout}");
    assert!(
        stdout.contains("'missing_client.lua' does not exist"),
        "{stdout}"
    );
    assert!(stdout.contains("duplicate entry"), "{stdout}");
    assert!(stdout.contains("es_extended"), "{stdout}");
    assert!(stdout.contains("4 error(s)"), "{stdout}");
}

#[test]
fn test_cli_release_pipeline() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");

    let gen_output = Command::new(env!("CARGO_BIN_EXE_trek"))
        .current_dir(temp_dir.path())
        .args(["generate", "-n", "rel_res", "-f", "None"])
        .output()
        .expect("Failed to run trek generate");
    assert!(gen_output.status.success());

    let res_path = temp_dir.path().join("rel_res");
    let dist_dir = temp_dir.path().join("dist");

    let output = Command::new(env!("CARGO_BIN_EXE_trek"))
        .current_dir(&res_path)
        .args([
            "release",
            "--patch",
            "--sha256",
            "-o",
            dist_dir.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to run trek release");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("No issues found"), "{stdout}");
    assert!(
        stdout.contains("Bumped version from 0.0.0 -> 0.0.1"),
        "{stdout}"
    );
    assert!(stdout.contains("Packed"), "{stdout}");
    assert!(stdout.contains("SHA256"), "{stdout}");
    assert!(stdout.contains("- **SHA256:** `"), "{stdout}");
    assert!(stdout.contains("Release completed!"), "{stdout}");

    let zip_path = dist_dir.join("rel_res.zip");
    assert!(zip_path.exists());

    let manifest = fs::read_to_string(res_path.join("fxmanifest.lua")).unwrap();
    assert!(manifest.contains("version(\"0.0.1\")"));
}

#[test]
fn test_cli_validate_parse_error_shows_codeframe() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let manifest_path = temp_dir.path().join("fxmanifest.lua");
    fs::write(
        &manifest_path,
        "fx_version 'cerulean'\ngame 'gta5'\nversion = ~\n",
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_trek"))
        .args(["validate", "-m", manifest_path.to_str().unwrap()])
        .output()
        .expect("Failed to run trek validate");

    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("LexError") || stdout.contains("ParseError"),
        "{stdout}"
    );
    assert!(stdout.contains("--> "), "{stdout}");
    assert!(stdout.contains(":3:11"), "{stdout}");
    assert!(stdout.contains("^"), "{stdout}");
    assert!(stdout.contains("unexpected character '~'"), "{stdout}");
}

#[test]
fn test_cli_version_validation_errors() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let manifest_path = temp_dir.path().join("fxmanifest.lua");
    fs::write(&manifest_path, "fx_version 'cerulean'\nauthor 'Trek'").unwrap();

    // Error: no version entry
    let output = Command::new(env!("CARGO_BIN_EXE_trek"))
        .args(["version", "--patch", "-m", manifest_path.to_str().unwrap()])
        .output()
        .expect("Failed to run trek version");
    assert!(!output.status.success());

    // Error: multiple bump flags
    let output = Command::new(env!("CARGO_BIN_EXE_trek"))
        .args([
            "version",
            "--patch",
            "--minor",
            "-m",
            manifest_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to run trek version");
    assert!(!output.status.success());
}
