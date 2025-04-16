use std::sync::Arc;
use chrono::Utc;
use crate::log_contracts::Logger;
use crate::log_contracts::LogEntry;
use crate::log_contracts::LogDestination;
use crate::LogLevel;

thread_local! {
    static MDC_CONTEXT: std::cell::RefCell<Option<String>> = std::cell::RefCell::new(None);
}

pub struct LogWriter {
    threshold: LogLevel,
    destination: Arc<dyn LogDestination>,
}

impl LogWriter {
    pub fn new(threshold: LogLevel, destination: Arc<dyn LogDestination>) -> Self {
        Self { threshold, destination }
    }

    pub fn set_context(context: String) {
        MDC_CONTEXT.with(|ctx| *ctx.borrow_mut() = Some(context));
    }

    fn get_context() -> Option<String> {
        MDC_CONTEXT.with(|ctx| ctx.borrow().clone())
    }

    fn build_entry(&self, level: LogLevel, message: &str) -> LogEntry {
        LogEntry {
            timestamp: Utc::now().to_rfc3339(),
            level: level.to_string(),
            message: message.to_string(),
            context: Self::get_context(),
        }
    }
}

impl Logger for LogWriter {
    fn log(&self, level: LogLevel, message: &str) {
        if level < self.threshold {
            return;
        }
        let entry = self.build_entry(level, message);
        self.destination.write_log(&entry);
    }
}
