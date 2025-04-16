use reqwest::blocking::Client;
use std::sync::Mutex;
use crate::log_contracts::LogDestination;
use crate::log_contracts::LogEntry;

pub struct HttpLogDestination {
    endpoint: String,
    client: Mutex<Client>,
}

impl HttpLogDestination {
    pub fn new(endpoint: &str) -> Self {
        Self {
            endpoint: endpoint.to_string(),
            client: Mutex::new(Client::new()),
        }
    }
}

impl LogDestination for HttpLogDestination {
    fn write_log(&self, entry: &LogEntry) {
        let client = self.client.lock().unwrap();
        let _ = client.post(&self.endpoint).json(entry).send();
    }
}
