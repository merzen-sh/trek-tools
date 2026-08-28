use std::path::Path;

use anyhow::{Result, bail};
use dialoguer::console::style;
use trek_fxmanifest::{Diagnostic, Key, Severity, parse};

fn severity_symbol(s: Severity) -> String {
    match s {
        Severity::Error => style("✗").red().bold().to_string(),
        Severity::Warning => style("⚠").yellow().bold().to_string(),
    }
}

pub fn run(manifest_path: &Path) -> Result<bool> {
    if !manifest_path.exists() {
        bail!(
            "Manifest file not found at '{}'. Make sure you are in a FiveM resource directory.",
            manifest_path.display()
        );
    }

    let content = trek_fxmanifest::Source::open(manifest_path)?;
    let base_dir = manifest_path.parent().unwrap_or(Path::new("."));

    let start = std::time::Instant::now();
    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    let manifest = match parse(&content) {
        Ok(manifest) => manifest,
        Err(err) => {
            report_parse_error(manifest_path, &content, &err)?;
            return Ok(true);
        }
    };

    diagnostics.extend(check_declarations(&manifest));
    diagnostics.extend(check_script_files(base_dir, &manifest));
    diagnostics.extend(check_duplicates(&manifest));
    diagnostics.extend(check_framework_dependencies(&manifest));

    let elapsed_ms = start.elapsed().as_millis();

    fn report_parse_error(
        manifest_path: &Path,
        content: &str,
        err: &trek_fxmanifest::Error,
    ) -> Result<()> {
        let (line, col, category, message) = match err {
            trek_fxmanifest::Error::Lex(e) => (e.line, e.col, "LexError", e.message.clone()),
            trek_fxmanifest::Error::Parse(e) => (e.line, e.col, "ParseError", e.message.clone()),
        };

        let frame = trek_fxmanifest::Codeframe::new(line, col);
        let rendered = frame.render_diagnostic(
            content,
            Some(manifest_path),
            Severity::Error,
            category,
            &message,
        );

        for rendered_line in rendered.lines() {
            if rendered_line.contains('^') || rendered_line.contains('~') {
                println!("{}", style(rendered_line).red().bold());
            } else if rendered_line.contains(" --> ") || rendered_line.trim_end().ends_with('|') {
                println!("{}", style(rendered_line).dim());
            } else if rendered_line.starts_with('[') {
                println!(
                    "{} {}",
                    severity_symbol(Severity::Error),
                    rendered_line[rendered_line.find(']').map(|i| i + 1).unwrap_or(0)..]
                        .trim_start()
                );
            } else {
                println!("{rendered_line}");
            }
        }

        Ok(())
    }

    if diagnostics.is_empty() {
        println!(
            "{} No issues found in '{}' ({}ms)",
            style("✓").green().bold(),
            manifest_path.display(),
            elapsed_ms
        );
        return Ok(false);
    }

    let errors = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .count();
    let warnings = diagnostics.len() - errors;

    for diagnostic in &diagnostics {
        println!(
            "{} {}",
            severity_symbol(diagnostic.severity),
            diagnostic.message
        );
    }

    println!(
        "\n{} {} error(s), {} warning(s) in '{}' ({}ms)",
        style("✗").red().bold(),
        errors,
        warnings,
        manifest_path.display(),
        elapsed_ms
    );

    Ok(errors > 0)
}

fn check_declarations(manifest: &trek_fxmanifest::Manifest) -> Vec<Diagnostic> {
    let mut out = Vec::new();

    match &manifest.fx_version {
        None => out.push(Diagnostic::error(
            "missing required declaration 'fx_version'",
        )),
        Some(version) if version != "cerulean" => out.push(Diagnostic::warning(format!(
            "unknown fx_version '{version}' (expected \"cerulean\")"
        ))),
        Some(_) => {}
    }

    match &manifest.game {
        None => out.push(Diagnostic::error("missing required declaration 'game'")),
        Some(trek_fxmanifest::Game::Other(name)) => {
            out.push(Diagnostic::warning(format!("unknown game '{name}'")));
        }
        Some(_) => {}
    }

    if manifest.get(&Key::Lua54).map(str::trim) != Some("yes") {
        out.push(Diagnostic::warning(
            "'lua54' is not enabled; FiveM expects lua54 'yes'",
        ));
    }

    out
}

fn check_script_files(base_dir: &Path, manifest: &trek_fxmanifest::Manifest) -> Vec<Diagnostic> {
    let mut out = Vec::new();

    for key in [
        Key::ClientScripts,
        Key::ServerScripts,
        Key::SharedScripts,
        Key::Files,
    ] {
        let mut seen = Vec::new();
        for entry in manifest.values(&key) {
            if entry.starts_with('@') || is_glob(entry) {
                continue;
            }

            if !base_dir.join(entry).is_file() {
                out.push(Diagnostic::error(format!(
                    "'{}' entry '{entry}' does not exist on disk",
                    key.as_str()
                )));
            }

            if seen.contains(&entry) {
                out.push(Diagnostic::error(format!(
                    "duplicate entry '{entry}' in '{}'",
                    key.as_str()
                )));
            }
            seen.push(entry);
        }
    }

    out
}

fn check_duplicates(manifest: &trek_fxmanifest::Manifest) -> Vec<Diagnostic> {
    let mut counts: Vec<(String, usize)> = Vec::new();
    let mut out = Vec::new();

    for statement in &manifest.statements {
        if !matches!(statement, trek_fxmanifest::Statement::Scalar { .. }) {
            continue;
        }
        let name = statement.key().as_str().to_string();
        if let Some((_, count)) = counts.iter_mut().find(|(k, _)| *k == name) {
            *count += 1;
        } else {
            counts.push((name, 1));
        }
    }

    for (name, count) in counts.into_iter().filter(|(_, c)| *c > 1) {
        out.push(Diagnostic::warning(format!(
            "declaration '{name}' appears {count} times"
        )));
    }

    out
}

fn check_framework_dependencies(manifest: &trek_fxmanifest::Manifest) -> Vec<Diagnostic> {
    const FRAMEWORK_PREFIXES: &[(&str, &str)] = &[
        ("@es_extended/", "es_extended"),
        ("@qb-core/", "qb-core"),
        ("@qbx_core/", "qbx_core"),
    ];

    let referenced: Vec<String> = FRAMEWORK_PREFIXES
        .iter()
        .filter(|(prefix, _)| {
            [Key::ClientScripts, Key::ServerScripts, Key::SharedScripts]
                .iter()
                .any(|key| manifest.values(key).iter().any(|v| v.starts_with(prefix)))
        })
        .map(|(_, resource)| resource.to_string())
        .collect();

    if referenced.is_empty() {
        return Vec::new();
    }

    let mut declared = manifest.values(&Key::Dependency);
    declared.extend(manifest.values(&Key::Dependencies));

    referenced
        .into_iter()
        .filter(|resource| {
            !declared
                .iter()
                .any(|dep| dep.trim().eq_ignore_ascii_case(resource))
        })
        .map(|resource| {
            Diagnostic::warning(format!(
                "'{resource}' is referenced by scripts but not declared under dependency/dependencies"
            ))
        })
        .collect()
}

fn is_glob(entry: &str) -> bool {
    entry.contains('*') || entry.contains('?')
}
