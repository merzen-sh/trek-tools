pub mod config;
pub mod fxmanifest;
pub mod src;

use anyhow::{Result, bail};
use askama::Template;
use indicatif::{ProgressBar, ProgressStyle};
use std::io::Write;
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use std::{fs, process::Command};

use crate::scaffolding::config::{ClientConfig, ServerConfig, ShareConfig};
use crate::scaffolding::src::{ClientScript, ServerScript, SharedUtils};

pub struct Scaffolding<'a> {
    resource_name: &'a str,
    include_ui: bool,
    install_deps: bool,
    fxmanifest: fxmanifest::Fxmanifest<'a>,
    src: src::Src<'a>,
    config: config::Config<'a>,
}

const DEFAULT_TREK_PACK: &str = r#"# trek include patterns
# Only files matching these patterns will be packed

fxmanifest.lua
config/**/*.lua
src/**/*.lua
ui/dist/**"#;

impl<'a> Scaffolding<'a> {
    pub fn new(
        resource_name: &'a str,
        description: &'a str,
        frameworks: &'a [&'a str],
        include_ui: bool,
        install_deps: bool,
    ) -> Self {
        let is_esx = frameworks.contains(&"ESX");
        let is_qbcore = frameworks.contains(&"QBCore");
        let is_qbox = frameworks.contains(&"Qbox");

        Self {
            config: config::Config {
                share_config: ShareConfig {
                    resource_name,
                    is_esx,
                    is_qbcore,
                    is_qbox,
                },
                client_config: ClientConfig {
                    resource_name,
                    is_esx,
                    is_qbcore,
                    is_qbox,
                },
                server_config: ServerConfig {
                    resource_name,
                    is_esx,
                    is_qbcore,
                    is_qbox,
                },
            },
            src: src::Src {
                client: ClientScript { resource_name },
                server: ServerScript { resource_name },
                shared: SharedUtils {
                    resource_name,
                    is_esx,
                    is_qbcore,
                    is_qbox,
                },
            },
            resource_name,
            include_ui,
            install_deps,
            fxmanifest: fxmanifest::Fxmanifest {
                description,
                include_ui,
            },
        }
    }

    fn write_file(&self, path: &Path, content: &str) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)?;
        Ok(())
    }

    fn bun_command(args: &[&str]) -> Command {
        if cfg!(target_os = "windows") {
            let mut c = Command::new("cmd");
            c.arg("/C").arg("bun").args(args);
            c
        } else {
            let mut c = Command::new("bun");
            c.args(args);
            c
        }
    }

    fn run_bun_create_vite(&self, base_dir: &Path) -> Result<()> {
        let mut cmd = Self::bun_command(&["create", "vite", "ui", "--template", "preact"]);

        let mut child = cmd
            .current_dir(base_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(b"\n");
        }

        let status = child.wait()?;

        if !status.success() {
            bail!("Failed to execute 'bun create vite'");
        }

        Ok(())
    }

    fn run_bun_install(&self, ui_dir: &Path) -> Result<()> {
        let mut cmd = Self::bun_command(&["install"]);

        let status = cmd
            .current_dir(ui_dir)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;

        if !status.success() {
            bail!("Failed to execute 'bun install'");
        }

        Ok(())
    }

    pub fn generate_all(&self, base_dir: &Path) -> Result<()> {
        let base_path = base_dir.join(self.resource_name);

        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
                .template("{spinner:.green} {msg}")?,
        );
        pb.enable_steady_tick(Duration::from_millis(80));

        pb.set_message("Generating fxmanifest.lua...");
        self.write_file(
            &base_path.join("fxmanifest.lua"),
            &self.fxmanifest.render()?,
        )?;

        pb.set_message("Generating src/client/client.lua...");
        self.write_file(
            &base_path.join("src/client/client.lua"),
            &self.src.client.render()?,
        )?;

        pb.set_message("Generating src/server/server.lua...");
        self.write_file(
            &base_path.join("src/server/server.lua"),
            &self.src.server.render()?,
        )?;

        pb.set_message("Generating src/shared/utils.lua...");
        self.write_file(
            &base_path.join("src/shared/utils.lua"),
            &self.src.shared.render()?,
        )?;

        pb.set_message("Generating config/share.lua...");
        self.write_file(
            &base_path.join("config/share.lua"),
            &self.config.share_config.render()?,
        )?;

        pb.set_message("Generating config/client.lua...");
        self.write_file(
            &base_path.join("config/client.lua"),
            &self.config.client_config.render()?,
        )?;

        pb.set_message("Generating config/server.lua...");
        self.write_file(
            &base_path.join("config/server.lua"),
            &self.config.server_config.render()?,
        )?;

        pb.set_message("Generating .pack...");
        self.write_file(&base_path.join(".pack"), DEFAULT_TREK_PACK)?;

        if self.include_ui {
            pb.set_message("Creating Vite + Preact project with Bun...");
            self.run_bun_create_vite(&base_path)?;

            if self.install_deps {
                pb.set_message("Installing dependencies with 'bun install'...");
                self.run_bun_install(&base_path.join("ui"))?;
            }
        }

        pb.finish_and_clear();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scaffolding_generate_all_without_ui() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let target_dir = temp_dir.path();

        let frameworks = &["ESX", "QBCore"];
        let scaffold = Scaffolding::new(
            "test_resource",
            "A test resource description",
            frameworks,
            false,
            false,
        );

        let result = scaffold.generate_all(target_dir);
        assert!(result.is_ok());

        let res_dir = target_dir.join("test_resource");
        assert!(res_dir.exists());

        // Verify generated files
        assert!(res_dir.join("fxmanifest.lua").exists());
        assert!(res_dir.join("src/client/client.lua").exists());
        assert!(res_dir.join("src/server/server.lua").exists());
        assert!(res_dir.join("src/shared/utils.lua").exists());
        assert!(res_dir.join("config/share.lua").exists());
        assert!(res_dir.join("config/client.lua").exists());
        assert!(res_dir.join("config/server.lua").exists());
        assert!(res_dir.join(".pack").exists());
        assert!(!res_dir.join("ui").exists());

        // Verify content
        let manifest_content = fs::read_to_string(res_dir.join("fxmanifest.lua")).unwrap();
        assert!(manifest_content.contains("A test resource description"));

        let trek_pack_content = fs::read_to_string(res_dir.join(".pack")).unwrap();
        assert!(trek_pack_content.contains("fxmanifest.lua"));

        let shared_content = fs::read_to_string(res_dir.join("src/shared/utils.lua")).unwrap();
        assert!(shared_content.contains("Utils = {}"));
        assert!(shared_content.contains("GetResourceState('qb-core')"));
        assert!(shared_content.contains("GetResourceState('es_extended')"));
    }

    #[test]
    fn test_bun_command_builder() {
        let cmd = Scaffolding::bun_command(&["--version"]);
        let program = cmd.get_program().to_string_lossy();
        if cfg!(target_os = "windows") {
            assert_eq!(program, "cmd");
        } else {
            assert_eq!(program, "bun");
        }
    }
}
