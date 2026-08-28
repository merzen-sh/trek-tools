use std::fs;
use std::path::Path;

use anyhow::{Result, bail};
use console::style;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BumpType {
    Major,
    Minor,
    Patch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemVer {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub prerelease: Option<String>,
    pub build: Option<String>,
}

impl SemVer {
    pub fn parse(input: &str) -> Result<Self> {
        let s = input.trim();
        let s = s.strip_prefix('v').unwrap_or(s);

        // Extract build metadata (+build)
        let (s, build) = match s.split_once('+') {
            Some((rest, b)) => (rest, Some(b.to_string())),
            None => (s, None),
        };

        // Extract prerelease identifier (-alpha.1)
        let (s, prerelease) = match s.split_once('-') {
            Some((rest, p)) => (rest, Some(p.to_string())),
            None => (s, None),
        };

        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            bail!(
                "Invalid SemVer format '{}': expected MAJOR.MINOR.PATCH (e.g., '1.0.0')",
                input
            );
        }

        let major: u64 = parts[0]
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid major version '{}'", parts[0]))?;
        let minor: u64 = parts[1]
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid minor version '{}'", parts[1]))?;
        let patch: u64 = parts[2]
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid patch version '{}'", parts[2]))?;

        Ok(Self {
            major,
            minor,
            patch,
            prerelease,
            build,
        })
    }

    pub fn bump(&mut self, bump_type: BumpType) {
        match bump_type {
            BumpType::Major => {
                self.major += 1;
                self.minor = 0;
                self.patch = 0;
                self.prerelease = None;
            }
            BumpType::Minor => {
                self.minor += 1;
                self.patch = 0;
                self.prerelease = None;
            }
            BumpType::Patch => {
                self.patch += 1;
                self.prerelease = None;
            }
        }
    }

    pub fn to_version_string(&self) -> String {
        let mut out = format!("{}.{}.{}", self.major, self.minor, self.patch);
        if let Some(ref pre) = self.prerelease {
            out.push('-');
            out.push_str(pre);
        }
        if let Some(ref build) = self.build {
            out.push('+');
            out.push_str(build);
        }
        out
    }
}

pub fn run(manifest_path: &Path, bump_type: Option<BumpType>, ci: bool) -> Result<()> {
    if !manifest_path.exists() {
        bail!(
            "Manifest file not found at '{}'. Make sure you are in a FiveM resource directory.",
            manifest_path.display()
        );
    }

    let content = fs::read_to_string(manifest_path)?;

    match bump_type {
        Some(bump_type) => {
            let (updated_content, old_ver, new_ver) = update_manifest_version(&content, bump_type)?;

            fs::write(manifest_path, updated_content)?;

            if ci {
                println!("{new_ver}");
            } else {
                println!(
                    "{} Bumped version from {} -> {} in '{}'",
                    style("✓").green().bold(),
                    style(&old_ver).yellow(),
                    style(&new_ver).green().bold(),
                    manifest_path.display()
                );
            }
        }
        None => {
            let version = extract_version(&content)?;
            if ci {
                println!("{version}");
            } else {
                println!(
                    "{} Current version: {}",
                    style("•").cyan().bold(),
                    style(&version).green().bold()
                );
            }
        }
    }

    Ok(())
}

pub fn extract_version(content: &str) -> Result<String> {
    for line in content.lines() {
        if let Some((_, _, semver)) = try_extract_quoted_version(line)? {
            return Ok(semver.to_version_string());
        }
    }
    bail!("No valid 'version' declaration found in manifest content");
}

pub fn update_manifest_version(
    content: &str,
    bump_type: BumpType,
) -> Result<(String, String, String)> {
    let mut updated_lines = Vec::new();
    let mut found_old_ver = None;
    let mut found_new_ver = None;

    for line in content.lines() {
        if found_old_ver.is_none()
            && let Some((updated_line, old_v, new_v)) = try_bump_version_line(line, bump_type)?
        {
            updated_lines.push(updated_line);
            found_old_ver = Some(old_v);
            found_new_ver = Some(new_v);
            continue;
        }
        updated_lines.push(line.to_string());
    }

    match (found_old_ver, found_new_ver) {
        (Some(old_v), Some(new_v)) => {
            let mut result = updated_lines.join("\n");
            if content.ends_with('\n') {
                result.push('\n');
            }
            Ok((result, old_v, new_v))
        }
        _ => {
            bail!("No valid 'version' declaration found in manifest content");
        }
    }
}

fn try_extract_quoted_version(line: &str) -> Result<Option<(usize, usize, SemVer)>> {
    let trimmed = line.trim();

    // Skip comment lines
    if trimmed.starts_with("--") {
        return Ok(None);
    }

    // Must start with version identifier
    if !trimmed.starts_with("version") {
        return Ok(None);
    }

    let after_version = &trimmed["version".len()..];
    let next_char = after_version.chars().next();

    // Ensure it's not a different identifier like `version_check` or `versioning`
    if let Some(c) = next_char
        && !matches!(c, ' ' | '\t' | '(' | '=' | '\'' | '"')
    {
        return Ok(None);
    }

    // Find first quote delimiter in line
    let quote_pos = line.find('\'').or_else(|| line.find('"'));
    let Some(start_quote_idx) = quote_pos else {
        return Ok(None);
    };
    let quote_char = line.as_bytes()[start_quote_idx] as char;

    let after_start = &line[start_quote_idx + 1..];

    let Some(end_quote_offset) = after_start.find(quote_char) else {
        return Ok(None);
    };

    let end_quote_idx = start_quote_idx + 1 + end_quote_offset;

    let version_str = &line[start_quote_idx + 1..end_quote_idx];
    let semver = SemVer::parse(version_str)?;

    Ok(Some((start_quote_idx, end_quote_idx, semver)))
}

fn try_bump_version_line(
    line: &str,
    bump_type: BumpType,
) -> Result<Option<(String, String, String)>> {
    let Some((start_quote_idx, end_quote_idx, semver)) = try_extract_quoted_version(line)? else {
        return Ok(None);
    };

    let old_ver = semver.to_version_string();
    let mut new_semver = semver;
    new_semver.bump(bump_type);
    let new_ver = new_semver.to_version_string();

    let mut updated_line = String::with_capacity(line.len() + 4);
    updated_line.push_str(&line[..=start_quote_idx]);
    updated_line.push_str(&new_ver);
    updated_line.push_str(&line[end_quote_idx..]);

    Ok(Some((updated_line, old_ver, new_ver)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semver_parse_and_bumps() {
        let mut v = SemVer::parse("1.2.3").unwrap();
        v.bump(BumpType::Patch);
        assert_eq!(v.to_version_string(), "1.2.4");

        let mut v = SemVer::parse("1.2.3").unwrap();
        v.bump(BumpType::Minor);
        assert_eq!(v.to_version_string(), "1.3.0");

        let mut v = SemVer::parse("1.2.3").unwrap();
        v.bump(BumpType::Major);
        assert_eq!(v.to_version_string(), "2.0.0");
    }

    #[test]
    fn test_semver_parse_prerelease_and_build() {
        let mut v = SemVer::parse("1.0.0-beta.1+build.123").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 0);
        assert_eq!(v.patch, 0);
        assert_eq!(v.prerelease, Some("beta.1".to_string()));
        assert_eq!(v.build, Some("build.123".to_string()));

        v.bump(BumpType::Patch);
        assert_eq!(v.to_version_string(), "1.0.1+build.123");
    }

    #[test]
    fn test_manifest_attribute_syntax_single_quote() {
        let manifest = "fx_version 'cerulean'\nversion '1.0.0'\nauthor 'Trek'";
        let (updated, old_v, new_v) = update_manifest_version(manifest, BumpType::Patch).unwrap();
        assert_eq!(old_v, "1.0.0");
        assert_eq!(new_v, "1.0.1");
        assert_eq!(
            updated,
            "fx_version 'cerulean'\nversion '1.0.1'\nauthor 'Trek'"
        );
    }

    #[test]
    fn test_manifest_attribute_syntax_double_quote() {
        let manifest = "fx_version \"cerulean\"\nversion \"2.4.9\"\nauthor \"Trek\"";
        let (updated, old_v, new_v) = update_manifest_version(manifest, BumpType::Minor).unwrap();
        assert_eq!(old_v, "2.4.9");
        assert_eq!(new_v, "2.5.0");
        assert_eq!(
            updated,
            "fx_version \"cerulean\"\nversion \"2.5.0\"\nauthor \"Trek\""
        );
    }

    #[test]
    fn test_manifest_function_syntax() {
        let manifest = "fx_version('cerulean')\nversion('0.1.5')\nauthor('Trek')";
        let (updated, old_v, new_v) = update_manifest_version(manifest, BumpType::Major).unwrap();
        assert_eq!(old_v, "0.1.5");
        assert_eq!(new_v, "1.0.0");
        assert_eq!(
            updated,
            "fx_version('cerulean')\nversion('1.0.0')\nauthor('Trek')"
        );
    }

    #[test]
    fn test_manifest_assignment_syntax_with_comments() {
        let manifest = "version = '3.1.2' -- current release version";
        let (updated, old_v, new_v) = update_manifest_version(manifest, BumpType::Patch).unwrap();
        assert_eq!(old_v, "3.1.2");
        assert_eq!(new_v, "3.1.3");
        assert_eq!(updated, "version = '3.1.3' -- current release version");
    }

    #[test]
    fn test_extract_version_all_syntaxes() {
        let cases = [
            ("version '1.0.0'", "1.0.0"),
            ("version \"2.3.4\"", "2.3.4"),
            ("version('0.5.9')", "0.5.9"),
            ("version = '3.1.2' -- comment", "3.1.2"),
        ];

        for (content, expected) in cases {
            assert_eq!(extract_version(content).unwrap(), expected);
        }
    }

    #[test]
    fn test_extract_version_prefers_first_match_and_skips_comments() {
        let manifest =
            "fx_version 'cerulean'\n-- version '9.9.9'\nversion '1.2.3'\nversion '4.5.6'";
        assert_eq!(extract_version(manifest).unwrap(), "1.2.3");
    }

    #[test]
    fn test_no_version_found_errors() {
        let manifest = "fx_version 'cerulean'\nauthor 'Trek'";
        let result = update_manifest_version(manifest, BumpType::Patch);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_semver_errors() {
        let manifest = "version 'not_a_semver'";
        let result = update_manifest_version(manifest, BumpType::Patch);
        assert!(result.is_err());
    }

    #[test]
    fn test_semver_parse_to_string_round_trip() {
        let test_cases = [
            "0.0.0",
            "1.2.3",
            "10.20.30",
            "1.0.0-alpha",
            "1.0.0-alpha.1",
            "1.0.0-0.3.7",
            "1.0.0-x.7.z.92",
            "1.0.0+20130313144700",
            "1.0.0-beta+exp.sha.5114f85",
            "v1.2.3",
        ];

        for &case in &test_cases {
            let parsed = SemVer::parse(case).expect("Should parse valid SemVer");
            let serialized = parsed.to_version_string();
            let round_tripped = SemVer::parse(&serialized).expect("Should parse serialized SemVer");
            assert_eq!(parsed, round_tripped);
        }
    }

    #[test]
    fn test_manifest_sequential_round_trip_bumping() {
        let mut manifest = r#"# Generated by trek
fx_version 'cerulean'
game 'gta5'

author 'Trek Scripts'
description 'A test FiveM resource'
version '0.0.0'

client_scripts {
    'config/share.lua',
    'src/client/client.lua',
}

server_scripts {
    'config/share.lua',
    'src/server/server.lua',
}
"#
        .to_string();

        let lifecycle_bumps = [
            (BumpType::Patch, "0.0.0", "0.0.1"),
            (BumpType::Patch, "0.0.1", "0.0.2"),
            (BumpType::Minor, "0.0.2", "0.1.0"),
            (BumpType::Patch, "0.1.0", "0.1.1"),
            (BumpType::Major, "0.1.1", "1.0.0"),
            (BumpType::Minor, "1.0.0", "1.1.0"),
            (BumpType::Patch, "1.1.0", "1.1.1"),
            (BumpType::Major, "1.1.1", "2.0.0"),
        ];

        for (bump_type, expected_old, expected_new) in lifecycle_bumps {
            let (updated, old_v, new_v) =
                update_manifest_version(&manifest, bump_type).expect("Bumping should succeed");

            assert_eq!(old_v, expected_old);
            assert_eq!(new_v, expected_new);
            assert!(updated.contains(&format!("version '{}'", expected_new)));
            assert!(updated.contains("author 'Trek Scripts'"));
            assert!(updated.contains("client_scripts {"));

            manifest = updated;
        }

        assert!(manifest.contains("version '2.0.0'"));
    }

    #[test]
    fn test_all_syntax_variations_round_trip() {
        let test_cases = [
            ("version '1.0.0'", "version '1.0.1'"),
            ("version \"1.0.0\"", "version \"1.0.1\""),
            ("version('1.0.0')", "version('1.0.1')"),
            ("version(\"1.0.0\")", "version(\"1.0.1\")"),
            ("version( '1.0.0' )", "version( '1.0.1' )"),
            ("version = '1.0.0'", "version = '1.0.1'"),
            ("version = \"1.0.0\"", "version = \"1.0.1\""),
            (
                "    version '1.0.0' -- inline comment",
                "    version '1.0.1' -- inline comment",
            ),
        ];

        for (original, expected) in test_cases {
            let (updated, old_v, new_v) =
                update_manifest_version(original, BumpType::Patch).expect("Bumping should succeed");
            assert_eq!(old_v, "1.0.0");
            assert_eq!(new_v, "1.0.1");
            assert_eq!(updated, expected);
        }
    }
}
