// oobe_core/src/telemetry.rs

use std::time::Instant;
use std::fmt;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let level = match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info  => "INFO",
            LogLevel::Warn  => "WARN",
            LogLevel::Error => "ERROR",
        };
        write!(f, "{}", level)
    }
}

#[derive(Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub context: Option<String>,
}

pub trait LogDestination: Send + Sync {
    fn write_log(&self, entry: &LogEntry);
}

pub trait Logger: Send + Sync {
    fn log(&self, level: LogLevel, message: &str);
    fn trace(&self, message: &str) { self.log(LogLevel::Trace, message); }
    fn debug(&self, message: &str) { self.log(LogLevel::Debug, message); }
    fn info(&self, message: &str)  { self.log(LogLevel::Info, message); }
    fn warn(&self, message: &str)  { self.log(LogLevel::Warn, message); }
    fn error(&self, message: &str) { self.log(LogLevel::Error, message); }
}

// Implement Debug for dyn Logger
impl fmt::Debug for dyn Logger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Logger")
    }
}

#[derive(Serialize)]
pub struct MetricEvent {
    pub name: String,
    pub kind: String, // "counter", "gauge", "duration"
    pub value: f64,
    pub timestamp: String,
}

pub trait MetricsDestination: Send + Sync {
    fn emit(&self, event: &MetricEvent);
}

pub trait MetricsEmitter: Send + Sync {
    fn emit_counter(&self, name: &str, value: f64);
    fn emit_gauge(&self, name: &str, value: f64);
    fn emit_duration(&self, name: &str, start: Instant);
}
