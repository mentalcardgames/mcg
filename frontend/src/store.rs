#[derive(Clone, Default, Debug)]
pub struct ClientSettings {
    pub name: String,
    pub server_address: String,
}

#[derive(Clone, Debug)]
pub struct ClientState {
    pub settings: ClientSettings,
}

impl Default for ClientState {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientState {
    pub fn new() -> Self {
        let default_settings = ClientSettings {
            name: "Player".to_string(),
            server_address: "127.0.0.1:3000".to_string(),
        };
        ClientState {
            settings: default_settings,
        }
    }
}
