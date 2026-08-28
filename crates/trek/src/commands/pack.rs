use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Result, bail};
use console::style;
use zip::ZipWriter;
use zip::write::FileOptions;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackedEntry {
    pub relative_path: String,
    pub full_path: PathBuf,
    pub size_bytes: u64,
}

pub fn run(out_dir: &Path, report: bool, dry_run: bool, sha256: bool) -> Result<()> {
    let start_time = Instant::now();
    let current_dir = std::env::current_dir()?;
    let trek_pack_path = current_dir.join(".pack");

    let include_patterns = if trek_pack_path.exists() {
        let content = fs::read_to_string(&trek_pack_path)?;
        parse_include_patterns(&content)
    } else {
        bail!(
            "No .pack found in '{}'. Run 'trek generate' or create one with include patterns.",
            current_dir.display()
        );
    };

    let resource_name = current_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("resource");

    let zip_name = format!("{}.zip", resource_name);
    let zip_path = out_dir.join(&zip_name);

    let mut entries = Vec::new();
    collect_files(&current_dir, &current_dir, &include_patterns, &mut entries)?;

    let total_bytes: u64 = entries.iter().map(|e| e.size_bytes).sum();
    let formatted_total_size = format_size(total_bytes);

    let mut checksum: Option<String> = None;

    if !dry_run {
        if let Some(parent) = zip_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let file = fs::File::create(&zip_path)?;
        let mut zip = ZipWriter::new(file);

        let options = FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .compression_level(Some(6));

        for entry in &entries {
            let mut f = fs::File::open(&entry.full_path)?;
            let mut contents = Vec::with_capacity(entry.size_bytes as usize);
            f.read_to_end(&mut contents)?;

            zip.start_file(&entry.relative_path, options)?;
            zip.write_all(&contents)?;
        }

        zip.finish()?;

        if sha256 {
            checksum = Some(compute_sha256(&zip_path)?);
        }
    }

    let elapsed = start_time.elapsed();
    let elapsed_ms = elapsed.as_millis();

    if dry_run {
        println!(
            "{} [DRY RUN] Would pack {} files ({}) into '{}' ({}ms)",
            style("✓").yellow().bold(),
            entries.len(),
            formatted_total_size,
            zip_path.display(),
            elapsed_ms
        );
    } else {
        println!(
            "{} Packed {} files ({}) into '{}' ({}ms)",
            style("✓").green().bold(),
            entries.len(),
            formatted_total_size,
            zip_path.display(),
            elapsed_ms
        );

        if let Some(hash) = &checksum {
            println!("{} SHA256: {}", style("✓").green().bold(), hash);
        }
    }

    if report {
        let report_md = generate_markdown_report(
            resource_name,
            &zip_path,
            &entries,
            total_bytes,
            elapsed_ms,
            dry_run,
            checksum.as_deref(),
        );
        println!("\n{}", report_md);
    }

    Ok(())
}

fn compute_sha256(path: &Path) -> Result<String> {
    use sha2::{Digest, Sha256};

    let file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut std::io::BufReader::new(file), &mut hasher)?;
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

pub fn generate_markdown_report(
    resource_name: &str,
    zip_path: &Path,
    entries: &[PackedEntry],
    total_bytes: u64,
    elapsed_ms: u128,
    dry_run: bool,
    checksum: Option<&str>,
) -> String {
    let mut out = String::new();
    out.push_str(&format!("# Pack Report: {}\n\n", resource_name));
    out.push_str(&format!(
        "- **Status:** {}\n",
        if dry_run {
            "Dry Run (Simulated)"
        } else {
            "Complete"
        }
    ));
    out.push_str(&format!("- **Archive Output:** `{}`\n", zip_path.display()));
    out.push_str(&format!("- **Files Packed:** {}\n", entries.len()));
    out.push_str(&format!(
        "- **Total Size:** {} ({} bytes)\n",
        format_size(total_bytes),
        total_bytes
    ));
    out.push_str(&format!("- **Elapsed Time:** {}ms\n", elapsed_ms));

    if let Some(hash) = checksum {
        out.push_str(&format!("- **SHA256:** `{}`\n", hash));
    }

    out.push('\n');

    out.push_str("### Included Files\n\n");
    out.push_str("| File | Size |\n");
    out.push_str("| :--- | :--- |\n");

    for entry in entries {
        out.push_str(&format!(
            "| `{}` | {} |\n",
            entry.relative_path,
            format_size(entry.size_bytes)
        ));
    }

    out
}

fn parse_include_patterns(content: &str) -> Vec<String> {
    content
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.to_string())
        .collect()
}

fn matches_pattern(path_str: &str, pattern: &str) -> bool {
    let path_parts: Vec<&str> = path_str.split('/').collect();
    let pat_parts: Vec<&str> = pattern.split('/').collect();
    match_components(&pat_parts, &path_parts)
}

fn match_components(pat: &[&str], path: &[&str]) -> bool {
    match pat.split_first() {
        None => path.is_empty(),
        Some((&"**", rest)) => {
            if rest.is_empty() {
                true
            } else {
                (0..=path.len()).any(|i| match_components(rest, &path[i..]))
            }
        }
        Some((&first, rest)) => {
            if path.is_empty() || !segment_matches(first, path[0]) {
                false
            } else {
                match_components(rest, &path[1..])
            }
        }
    }
}

fn segment_matches(pat: &str, s: &str) -> bool {
    if pat == "*" {
        return true;
    }
    match pat.split_once('*') {
        None => pat == s,
        Some((pre, post)) => {
            s.starts_with(pre) && s.ends_with(post) && s.len() >= pre.len() + post.len()
        }
    }
}

fn should_include(path: &Path, base_dir: &Path, patterns: &[String]) -> bool {
    let relative = path.strip_prefix(base_dir).unwrap_or(path);
    let relative_str = relative.to_string_lossy().replace('\\', "/");

    for pattern in patterns {
        if matches_pattern(&relative_str, pattern) {
            return true;
        }
    }

    false
}

fn collect_files(
    dir: &Path,
    base_dir: &Path,
    include_patterns: &[String],
    entries: &mut Vec<PackedEntry>,
) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            collect_files(&path, base_dir, include_patterns, entries)?;
        } else if should_include(&path, base_dir, include_patterns) {
            let relative_path = path.strip_prefix(base_dir).unwrap_or(&path);
            let entry_name = relative_path.to_string_lossy().replace('\\', "/");
            let metadata = entry.metadata()?;

            entries.push(PackedEntry {
                relative_path: entry_name,
                full_path: path,
                size_bytes: metadata.len(),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_include_patterns() {
        let content = r#"
# Comment line
fxmanifest.lua
config/**/*.lua

src/**/*.lua
# Another comment
ui/**
"#;
        let patterns = parse_include_patterns(content);
        assert_eq!(
            patterns,
            vec!["fxmanifest.lua", "config/**/*.lua", "src/**/*.lua", "ui/**"]
        );
    }

    #[test]
    fn test_matches_pattern_exact() {
        assert!(matches_pattern("fxmanifest.lua", "fxmanifest.lua"));
        assert!(!matches_pattern("fxmanifest.lua", "config.lua"));
        assert!(!matches_pattern("src/fxmanifest.lua", "fxmanifest.lua"));
    }

    #[test]
    fn test_matches_pattern_wildcards() {
        assert!(matches_pattern("config/share.lua", "config/**/*.lua"));
        assert!(matches_pattern(
            "config/nested/deep/file.lua",
            "config/**/*.lua"
        ));
        assert!(!matches_pattern(
            "config/nested/deep/file.txt",
            "config/**/*.lua"
        ));
        assert!(matches_pattern("ui/index.html", "ui/**"));
        assert!(matches_pattern("ui/dist/assets/index.js", "ui/**"));
        assert!(!matches_pattern("server/ui/index.html", "ui/**"));
    }

    #[test]
    fn test_segment_matches() {
        assert!(segment_matches("*", "anything"));
        assert!(segment_matches("*.lua", "main.lua"));
        assert!(segment_matches("prefix_*.lua", "prefix_test.lua"));
        assert!(!segment_matches("prefix_*.lua", "other_test.lua"));
        assert!(segment_matches("exact", "exact"));
        assert!(!segment_matches("exact", "other"));
    }

    #[test]
    fn test_should_include() {
        let base = Path::new("/workspace/my_res");
        let patterns = vec!["fxmanifest.lua".to_string(), "src/**/*.lua".to_string()];

        let manifest = Path::new("/workspace/my_res/fxmanifest.lua");
        let client_lua = Path::new("/workspace/my_res/src/client/client.lua");
        let git_file = Path::new("/workspace/my_res/.git/config");
        let node_modules = Path::new("/workspace/my_res/ui/node_modules/foo.js");

        assert!(should_include(manifest, base, &patterns));
        assert!(should_include(client_lua, base, &patterns));
        assert!(!should_include(git_file, base, &patterns));
        assert!(!should_include(node_modules, base, &patterns));
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(1048576), "1.00 MB");
        assert_eq!(format_size(1073741824), "1.00 GB");
    }

    #[test]
    fn test_generate_markdown_report() {
        let entries = vec![
            PackedEntry {
                relative_path: "fxmanifest.lua".to_string(),
                full_path: PathBuf::from("/path/fxmanifest.lua"),
                size_bytes: 474,
            },
            PackedEntry {
                relative_path: "src/client/client.lua".to_string(),
                full_path: PathBuf::from("/path/src/client/client.lua"),
                size_bytes: 1024,
            },
        ];

        let md = generate_markdown_report(
            "test_res",
            Path::new("./test_res.zip"),
            &entries,
            1498,
            25,
            false,
            None,
        );

        assert!(md.contains("# Pack Report: test_res"));
        assert!(md.contains("- **Status:** Complete"));
        assert!(!md.contains("SHA256"));
        assert!(md.contains("- **Archive Output:** `./test_res.zip`"));
        assert!(md.contains("- **Files Packed:** 2"));
        assert!(md.contains("- **Elapsed Time:** 25ms"));
        assert!(md.contains("| `fxmanifest.lua` | 474 B |"));
        assert!(md.contains("| `src/client/client.lua` | 1.0 KB |"));
    }

    #[test]
    fn test_collect_files() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let base_path = temp_dir.path();

        // Create structure
        fs::create_dir_all(base_path.join("src/client")).unwrap();
        fs::create_dir_all(base_path.join("src/server")).unwrap();

        fs::write(base_path.join("fxmanifest.lua"), "fx_version 'cerulean'").unwrap();
        fs::write(base_path.join("src/client/main.lua"), "-- client").unwrap();
        fs::write(base_path.join("src/server/main.lua"), "-- server").unwrap();
        fs::write(base_path.join("ignored.txt"), "skip me").unwrap();

        let patterns = vec!["fxmanifest.lua".to_string(), "src/**/*.lua".to_string()];
        let mut entries = Vec::new();

        collect_files(base_path, base_path, &patterns, &mut entries).unwrap();

        assert_eq!(entries.len(), 3);
        let names: Vec<&str> = entries.iter().map(|e| e.relative_path.as_str()).collect();
        assert!(names.contains(&"fxmanifest.lua"));
        assert!(names.contains(&"src/client/main.lua"));
        assert!(names.contains(&"src/server/main.lua"));
        assert!(!names.contains(&"ignored.txt"));
    }
}
