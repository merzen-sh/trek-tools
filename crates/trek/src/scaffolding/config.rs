#![allow(unused)]
use askama::Template;

#[derive(Template)]
#[template(path = "../templates/config/client.txt")]
pub struct ClientConfig<'a> {
    pub resource_name: &'a str,
    pub is_esx: bool,
    pub is_qbcore: bool,
    pub is_qbox: bool,
}

#[derive(Template)]
#[template(path = "../templates/config/server.txt")]
pub struct ServerConfig<'a> {
    pub resource_name: &'a str,
    pub is_esx: bool,
    pub is_qbcore: bool,
    pub is_qbox: bool,
}

#[derive(Template)]
#[template(path = "../templates/config/share.txt")]
pub struct ShareConfig<'a> {
    pub resource_name: &'a str,
    pub is_esx: bool,
    pub is_qbcore: bool,
    pub is_qbox: bool,
}

pub struct Config<'a> {
    pub client_config: ClientConfig<'a>,
    pub server_config: ServerConfig<'a>,
    pub share_config: ShareConfig<'a>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_config_rendering() {
        let esx_cfg = ClientConfig {
            resource_name: "test_res",
            is_esx: true,
            is_qbcore: false,
            is_qbox: false,
        };
        let rendered = esx_cfg.render().expect("Failed to render client config");
        assert!(rendered.contains("ClientConfig = {}"));
        assert!(rendered.contains("ClientConfig.Locale"));

        let qb_cfg = ClientConfig {
            resource_name: "test_res",
            is_esx: false,
            is_qbcore: true,
            is_qbox: false,
        };
        let rendered_qb = qb_cfg.render().expect("Failed to render client config");
        assert!(rendered_qb.contains("ClientConfig.UseTarget = true"));
    }

    #[test]
    fn test_server_config_rendering() {
        let srv_cfg = ServerConfig {
            resource_name: "test_res",
            is_esx: false,
            is_qbcore: true,
            is_qbox: false,
        };
        let rendered = srv_cfg.render().expect("Failed to render server config");
        assert!(rendered.contains("ServerConfig.Framework = 'qbcore'"));
        assert!(rendered.contains("ServerConfig.EnableCallbacks = true"));
    }

    #[test]
    fn test_share_config_rendering() {
        let share_cfg = ShareConfig {
            resource_name: "test_res",
            is_esx: true,
            is_qbcore: false,
            is_qbox: false,
        };
        let rendered = share_cfg.render().expect("Failed to render share config");
        assert!(rendered.contains("Config.Framework = 'esx'"));
        assert!(rendered.contains("Config.DefaultSpawn"));
    }
}
