pub mod config;
pub mod fxmanifest;
pub mod src;

use anyhow::Result;
use askama::Template;
use std::fs;
use std::path::Path;

use crate::scaffolding::config::{ClientConfig, ServerConfig, ShareConfig};
use crate::scaffolding::src::{ClientScript, ServerScript, SharedUtils};

pub struct Scaffolding<'a> {
    resource_name: &'a str,
    fxmanifest: fxmanifest::Fxmanifest<'a>,
    src: src::Src<'a>,
    config: config::Config<'a>,
}

const DEFAULT_TREK_PACK: &str = r#"# trek include patterns
# Only files matching these patterns will be packed

fxmanifest.lua
config/**/*.lua
src/**/*.lua"#;

const DEFAULT_STYLUA_TOML: &str = r#"column_width = 120
line_endings = "Unix"
indent_type = "Spaces"
indent_width = 4
quote_style = "AutoPreferDouble"
call_parentheses = "Always"
sort_requires = { enabled = true }"#;

impl<'a> Scaffolding<'a> {
    pub fn new(resource_name: &'a str, description: &'a str, frameworks: &'a [&'a str]) -> Self {
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
            fxmanifest: fxmanifest::Fxmanifest { description },
        }
    }

    fn write_file(&self, path: &Path, content: &str) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)?;
        Ok(())
    }

    pub fn generate_all(&self, base_dir: &Path) -> Result<()> {
        let base_path = base_dir.join(self.resource_name);

        self.write_file(
            &base_path.join("fxmanifest.lua"),
            &self.fxmanifest.render()?,
        )?;

        self.write_file(
            &base_path.join("src/client/client.lua"),
            &self.src.client.render()?,
        )?;

        self.write_file(
            &base_path.join("src/server/server.lua"),
            &self.src.server.render()?,
        )?;

        self.write_file(
            &base_path.join("src/shared/utils.lua"),
            &self.src.shared.render()?,
        )?;

        self.write_file(
            &base_path.join("config/share.lua"),
            &self.config.share_config.render()?,
        )?;

        self.write_file(
            &base_path.join("config/client.lua"),
            &self.config.client_config.render()?,
        )?;

        self.write_file(
            &base_path.join("config/server.lua"),
            &self.config.server_config.render()?,
        )?;

        self.write_file(&base_path.join(".pack"), DEFAULT_TREK_PACK)?;

        self.write_file(&base_path.join("stylua.toml"), DEFAULT_STYLUA_TOML)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scaffolding_generate_all() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let target_dir = temp_dir.path();

        let frameworks = &["ESX", "QBCore"];
        let scaffold = Scaffolding::new("test_resource", "A test resource description", frameworks);

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
        assert!(res_dir.join("stylua.toml").exists());

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
}
