use std::sync::Mutex;
use once_cell::sync::OnceCell;
// use crate::PluginContext;
use ws_server::ws_client::WsClient;

/*
USAGE:
tokio::spawn(async move {
    ws_utils::init_ws_client(ctx, "plugin_task_agent").await;
});
*/
pub static WS_CLIENT: OnceCell<Mutex<WsClient>> = OnceCell::new();

pub async fn init_ws_client_from_config(client_name: &str, config: &str) {
    let config_str = config.trim();

    let ws_url = if let Some(url) = config_str.split(';').find(|kv| kv.starts_with("ws=")) {
        url.trim_start_matches("ws=").to_string()
    } else {
        "ws://localhost:8081/ws".to_string()
    };

    println!("[{}] Connecting to {}", client_name, ws_url);

    let client = WsClient::connect(client_name, &ws_url).await.unwrap();
    WS_CLIENT.set(Mutex::new(client)).ok();

    println!("[{}] WebSocket client initialized", client_name);
}

