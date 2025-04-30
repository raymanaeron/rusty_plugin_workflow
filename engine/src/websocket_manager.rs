use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::sync::mpsc::UnboundedSender;
use once_cell::sync::{Lazy, OnceCell};
use ws_server::ws_client::WsClient;

pub type Subscribers = Arc<Mutex<HashMap<String, Vec<UnboundedSender<String>>>>>;

/// WebSocket subscribers for the engine.
pub static WS_SUBSCRIBERS: Lazy<Subscribers> = Lazy::new(|| {
    Arc::new(Mutex::new(HashMap::new()))
});

/// WebSocket client for the engine.
pub static ENGINE_WS_CLIENT: OnceCell<Arc<Mutex<WsClient>>> = OnceCell::new();

/// Topic for receiving network connected messages.
pub static NETWORK_CONNECTED: &str = "NetworkConnected";

/// Topic for receiving switch route messages.
pub static SWITCH_ROUTE: &str = "SwitchRoute";

/// Topic for receiving welcome completed messages.
pub static WELCOME_COMPLETED: &str = "WelcomeCompleted";

/// Topic for receiving network connected messages.
pub static WIFI_COMPLETED : &str = "WifiCompleted";

/// Topic for receiving execution plan completed messages.
pub static EXECPLAN_COMPLETED : &str = "ExecutionPlanCompleted";

/// Topic for receiving login completed messages.
pub static LOGIN_COMPLETED : &str = "LoginCompleted";

/// Topic for provision completed messages.
pub static PROVISION_COMPLETED : &str = "ProvisionCompleted";