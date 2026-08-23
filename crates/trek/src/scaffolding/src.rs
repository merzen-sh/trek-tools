use askama::Template;

#[derive(Template)]
#[template(path = "../templates/src/client/main.txt")]
pub struct ClientScript<'a> {
    pub resource_name: &'a str,
}

#[derive(Template)]
#[template(path = "../templates/src/server/main.txt")]
pub struct ServerScript<'a> {
    pub resource_name: &'a str,
}

#[derive(Template)]
#[template(path = "../templates/src/shared/utils.txt")]
pub struct SharedUtils<'a> {
    pub resource_name: &'a str,
    pub is_esx: bool,
    pub is_qbcore: bool,
    pub is_qbox: bool,
}

pub struct Src<'a> {
    pub client: ClientScript<'a>,
    pub server: ServerScript<'a>,
    pub shared: SharedUtils<'a>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_script_render() {
        let client = ClientScript {
            resource_name: "my_cool_res",
        };
        let rendered = client.render().expect("Failed to render client script");
        assert!(rendered.contains("Client Side Script: my_cool_res"));
        assert!(rendered.contains("RegisterNetEvent('my_cool_res:client:notify')"));
        assert!(rendered.contains("Utils.FrameworkName == 'qbcore'"));
    }

    #[test]
    fn test_server_script_render() {
        let server = ServerScript {
            resource_name: "my_cool_res",
        };
        let rendered = server.render().expect("Failed to render server script");
        assert!(rendered.contains("Server Side Script: my_cool_res"));
        assert!(rendered.contains("RegisterServerEvent('my_cool_res:server:callback')"));
        assert!(rendered.contains("Utils.FrameworkName == 'esx'"));
    }

    #[test]
    fn test_shared_utils_multi_framework_render() {
        let shared = SharedUtils {
            resource_name: "my_cool_res",
            is_esx: true,
            is_qbcore: true,
            is_qbox: true,
        };
        let rendered = shared.render().expect("Failed to render shared utils");
        assert!(rendered.contains("Utils.Framework = nil"));
        assert!(rendered.contains("GetResourceState('qb-core') == 'started'"));
        assert!(rendered.contains("GetResourceState('es_extended') == 'started'"));
        assert!(rendered.contains("GetResourceState('qbx_core') == 'started'"));
    }

    #[test]
    fn test_shared_utils_standalone_render() {
        let shared = SharedUtils {
            resource_name: "my_cool_res",
            is_esx: false,
            is_qbcore: false,
            is_qbox: false,
        };
        let rendered = shared.render().expect("Failed to render shared utils");
        assert!(rendered.contains("Running in standalone mode."));
        assert!(!rendered.contains("GetResourceState('qb-core')"));
    }
}
