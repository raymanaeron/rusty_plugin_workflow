/*
 * Logger implementation module for Rusty Logger v2
 * 
 * This file implements the core Logger functionality which includes:
 * - Creation and initialization of the global logger instance
 * - Configuration of the logger from TOML files or programmatically
 * - Asynchronous logging through Tokio with message passing
 * - Automatic fallback to synchronous logging when needed
 * - Thread-safe logging with proper synchronization
 * 
 * The Logger uses a singleton pattern with lazy initialization via OnceCell
 * to ensure there's only one logger instance throughout the application.
 */

 use once_cell::sync::OnceCell;
 use std::sync::{Arc, Mutex, atomic::{AtomicU64, Ordering}};
 use std::path::Path;
 use chrono::Utc;
 use std::io::{self, Write};
 use tokio::sync::{mpsc::{self, Sender, Receiver}, oneshot};
 use tokio::runtime::Runtime;
 use tokio::time::{timeout, Duration as TokioDuration};
 
 use crate::config::{LogConfig, LogLevel};
 use crate::outputs::{LogOutput, create_log_output, create_async_log_output, AsyncLogOutputTrait};
 use crate::outputs::AsyncLogOutput;
 
 // Global logger instance
 static LOGGER_INSTANCE: OnceCell<Arc<Mutex<LoggerInner>>> = OnceCell::new();
 static RUNTIME: OnceCell<Runtime> = OnceCell::new();
 
 // Message structure for async logging channel
 struct LogMessage {
     timestamp: String,
     level: LogLevel,
     message: String,
     context: Option<String>,
     file: String,
     line: u32,
     module: String,
 }
 
 // Command enum for controlling the background worker
 enum LogCommand {
     Entry(LogMessage),
     Shutdown(oneshot::Sender<()>),
 }
 
 struct LoggerInner {
     initialized: bool,
     config: Option<LogConfig>,
     output: Option<Box<dyn LogOutput>>,
     // Channel sender for async logging
     async_sender: Option<Sender<LogCommand>>,
     /// Flag to indicate if asynchronous logging is enabled
     /// When false, all logging operations will be synchronous
     async_enabled: bool,
     /// Counter for messages dropped due to channel backpressure
     dropped_logs: AtomicU64,
     /// Counter to track when to report dropped logs
     log_counter: AtomicU64,
 }
 
 impl LoggerInner {
     /// Creates a new uninitialized logger inner structure
     fn new() -> Self {
         LoggerInner {
             initialized: false,
             config: None,
             output: None,
             async_sender: None,
             async_enabled: false,
             dropped_logs: AtomicU64::new(0),
             log_counter: AtomicU64::new(0),
         }
     }
 
     /// Initializes the logger with the provided configuration
     fn init_with_config(&mut self, config: LogConfig) -> Result<(), String> {
         println!("Setting up logger with log type: {:?}", config.log_type);
         
         // Create the appropriate log output based on configuration
         let output = create_log_output(&config.log_type)?;
         self.output = Some(output);
         
         // Set up async logging if enabled
         if config.async_logging {
             // Create Tokio runtime if not already initialized
             let runtime = RUNTIME.get_or_init(|| {
                 Runtime::new().expect("Failed to create Tokio runtime")
             });
             
             // Create channel for async logging with LogCommand instead of LogMessage
             let (tx, rx) = mpsc::channel::<LogCommand>(100);
             self.async_sender = Some(tx);
             
             // Create the async output
             let async_output = create_async_log_output(&config.log_type)?;
             
             // Spawn a task to process log messages
             runtime.spawn(async move {
                 process_log_commands(rx, async_output).await
                     .unwrap_or_else(|e| eprintln!("Async logging failed: {}", e));
             });
         }
         
         // Store the configuration
         self.config = Some(config.clone());
         self.async_enabled = config.async_logging;
         self.initialized = true;
         
         Ok(())
     }
 
     /// Log a message with the configured output
     fn log(&mut self, level: LogLevel, message: &str, context: Option<&str>, file: &str, line: u32, module: &str) {
         // Check if we're initialized with a configuration
         if let Some(ref config) = self.config {
             // Skip logging if level is below threshold
             if (level.clone() as usize) < (config.threshold.clone() as usize) {
                 return;
             }
             
             // Format timestamp
             let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
             
             // Increment log counter
             let count = self.log_counter.fetch_add(1, Ordering::Relaxed) + 1;
             
             // Check if we need to report dropped logs (every 100 logs)
             if count % 100 == 0 {
                 self.report_dropped_logs();
             }
             
             // Try async logging first if enabled
             if self.async_enabled {
                 if let Some(ref sender) = self.async_sender {
                     // Create a log message for the async channel
                     let log_message = LogMessage {
                         timestamp: timestamp.clone(),
                         level: level.clone(),
                         message: message.to_string(),
                         context: context.map(|s| s.to_string()),
                         file: file.to_string(),
                         line,
                         module: module.to_string(),
                     };
                     
                     // Send to the async channel as a LogCommand::Entry, fallback to sync if channel is full
                     if let Err(_) = sender.try_send(LogCommand::Entry(log_message)) {
                         // Increment dropped logs counter before falling back to sync
                         self.dropped_logs.fetch_add(1, Ordering::Relaxed);
                         
                         // Channel full or closed, fallback to sync logging
                         self.log_sync(&timestamp, &level, message, context, file, line, module);
                     }
                 } else {
                     // Async sender not initialized, fallback to sync logging
                     self.log_sync(&timestamp, &level, message, context, file, line, module);
                 }
             } else {
                 // Async logging disabled, use sync logging
                 self.log_sync(&timestamp, &level, message, context, file, line, module);
             }
         } else {
             // Fallback to stderr for uninitialized logger
             let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
             self.log_sync(&timestamp, &level, message, context, file, line, module);
         }
     }
     
     /// Report dropped logs if any
     fn report_dropped_logs(&mut self) {
         let dropped = self.dropped_logs.load(Ordering::Relaxed);
         if dropped > 0 {
             // Reset the counter first to avoid multiple reports of the same drops
             let actual_dropped = self.dropped_logs.swap(0, Ordering::Relaxed);
             
             // Log a warning about dropped messages
             let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
             let warning_message = format!("WARNING: {} log messages were dropped due to backpressure", actual_dropped);
             self.log_sync(
                 &timestamp, 
                 &LogLevel::Warn, 
                 &warning_message, 
                 None,
                 "logger.rs",
                 0,
                 "liblogger"
             );
         }
     }
 
     /// Synchronous logging fallback
     fn log_sync(&mut self, timestamp: &str, level: &LogLevel, message: &str, 
                 context: Option<&str>, file: &str, line: u32, module: &str) {
         if let Some(ref mut output) = self.output {
             // Format the log message
             let formatted_message = format_log_message(timestamp, level, message, context, file, line, module);
             
             // Write the log
             if let Err(e) = output.write_log(&formatted_message) {
                 eprintln!("Failed to write log: {}", e);
             }
         } else {
             // No output configured, write to stderr
             let level_str = level.as_str();
             let log_line = match context {
                 Some(ctx) => format!("{} [{}] [{}:{}] [{}] {} | {}\n", 
                     timestamp, level_str, file, line, module, message, ctx),
                 None => format!("{} [{}] [{}:{}] [{}] {}\n",
                     timestamp, level_str, file, line, module, message),
             };
             
             let _ = io::stderr().write_all(log_line.as_bytes());
         }
     }
 }
 
 // Format a log message for output
 fn format_log_message(timestamp: &str, level: &LogLevel, message: &str, 
                     context: Option<&str>, file: &str, line: u32, module: &str) -> String {
     let level_str = level.as_str();
     match context {
         Some(ctx) => format!("{} [{}] [{}:{}] [{}] {} | {}", 
             timestamp, level_str, file, line, module, message, ctx),
         None => format!("{} [{}] [{}:{}] [{}] {}",
             timestamp, level_str, file, line, module, message),
     }
 }
 
 // Async function to process log commands from the channel
 async fn process_log_commands(mut receiver: Receiver<LogCommand>, mut output: AsyncLogOutput) -> Result<(), String> {
     while let Some(cmd) = receiver.recv().await {
         match cmd {
             LogCommand::Entry(msg) => {
                 // Format the log message
                 let formatted_message = format_log_message(
                     &msg.timestamp, &msg.level, &msg.message, 
                     msg.context.as_deref(), &msg.file, msg.line, &msg.module);
                 
                 // Write using the async output
                 if let Err(e) = output.write_log_async(&formatted_message).await {
                     eprintln!("Async logging error: {}", e);
                 }
             },
             LogCommand::Shutdown(completion_sender) => {
                 // Final log message before shutdown
                 let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
                 let message = "Logger shutdown initiated, ensuring all logs are flushed";
                 let formatted_message = format_log_message(
                     &timestamp, &LogLevel::Info, message, None, "logger.rs", 0, "liblogger");
                 
                 // Final flush before shutdown
                 if let Err(e) = output.write_log_async(&formatted_message).await {
                     eprintln!("Error writing final log message: {}", e);
                 }
                 
                 // Notify that shutdown is complete
                 let _ = completion_sender.send(());
                 
                 // Break the loop to end the task
                 break;
             }
         }
     }
     
     Ok(())
 }
 
 pub struct Logger;
 
 impl Logger {
     /// Initialize the logger with default configuration file "app_config.toml"
     pub fn init() {
         let _ = Self::init_with_config_file("app_config.toml");
     }
 
     /// Initialize the logger with a specific configuration file
     pub fn init_with_config_file(config_path: &str) -> Result<(), String> {
         let config = LogConfig::from_file(config_path)?;
         Self::init_with_config(config)
     }
 
     /// Initialize the logger with a LogConfig struct
     pub fn init_with_config(config: LogConfig) -> Result<(), String> {
         println!("Setting up logger with log type: {:?}", config.log_type);
         
         let logger = LOGGER_INSTANCE.get_or_init(|| Arc::new(Mutex::new(LoggerInner::new())));
         let mut logger_guard = match logger.lock() {
             Ok(guard) => guard,
             Err(poisoned) => {
                 println!("Logger mutex was poisoned, recovering...");
                 poisoned.into_inner()
             }
         };
         
         match logger_guard.init_with_config(config) {
             Ok(_) => {
                 println!("Logger initialized successfully");
                 Ok(())
             },
             Err(e) => {
                 println!("Failed to initialize logger: {}", e);
                 Err(e)
             }
         }
     }
 
     /// Log a debug message
     pub fn debug(message: &str, context: Option<String>, file: &'static str, line: u32, module: &'static str) {
         Self::log_with_metadata(LogLevel::Debug, message, context, file, line, module)
     }
 
     /// Log an info message
     pub fn info(message: &str, context: Option<String>, file: &'static str, line: u32, module: &'static str) {
         Self::log_with_metadata(LogLevel::Info, message, context, file, line, module)
     }
 
     /// Log a warning message
     pub fn warn(message: &str, context: Option<String>, file: &'static str, line: u32, module: &'static str) {
         Self::log_with_metadata(LogLevel::Warn, message, context, file, line, module)
     }
 
     /// Log an error message
     pub fn error(message: &str, context: Option<String>, file: &'static str, line: u32, module: &'static str) {
         Self::log_with_metadata(LogLevel::Error, message, context, file, line, module)
     }
 
     fn log_with_metadata(level: LogLevel, message: &str, context: Option<String>, file: &str, line: u32, module: &str) {
         // Extract just the filename from the path
         let file_name = Path::new(file)
             .file_name()
             .and_then(|n| n.to_str())
             .unwrap_or(file);
 
         let logger = LOGGER_INSTANCE.get_or_init(|| Arc::new(Mutex::new(LoggerInner::new())));
         
         // Use a block to limit the scope of the mutex lock
         {
             if let Ok(mut logger) = logger.lock() {
                 logger.log(level, message, context.as_deref(), file_name, line, module);
             } else {
                 // If the mutex is poisoned, log to stderr
                 let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
                 let level_str = level.as_str();
                 let log_line = format!("{} [{}] [{}:{}] [{}] {} | MUTEX POISONED\n",
                     timestamp, level_str, file_name, line, module, message);
                 let _ = io::stderr().write_all(log_line.as_bytes());
             }
         }
     }
 
     /// Shutdown the logger gracefully, ensuring all pending logs are written
     pub fn shutdown() -> Result<(), String> {
         // Try to get the runtime
         if let Some(rt) = RUNTIME.get() {
             // Check if we have an async logger initialized
             if let Some(logger) = LOGGER_INSTANCE.get() {
                 if let Ok(mut logger_guard) = logger.lock() {
                     // Report any dropped logs before shutdown
                     logger_guard.report_dropped_logs();
                     
                     if logger_guard.async_enabled {
                         if let Some(sender) = &logger_guard.async_sender {
                             // Create a oneshot channel for completion notification
                             let (completion_tx, completion_rx) = oneshot::channel();
                             
                             // Clone sender outside of task to avoid reference issues
                             let sender_clone = sender.clone();
                             
                             // Send shutdown command
                             // Use block to release the mutex guard before the blocking operation
                             drop(logger_guard);
                             
                             // Spawn a Tokio task to send the shutdown command
                             let handle = rt.spawn(async move {
                                 if let Err(e) = sender_clone.send(LogCommand::Shutdown(completion_tx)).await {
                                     eprintln!("Failed to send shutdown command: {}", e);
                                     return false;
                                 }
                                 
                                 // Wait for completion with timeout
                                 match timeout(TokioDuration::from_secs(5), completion_rx).await {
                                     Ok(Ok(())) => {
                                         println!("Logger shutdown completed successfully");
                                         true
                                     },
                                     Ok(Err(_)) => {
                                         eprintln!("Shutdown completion channel was closed");
                                         false
                                     },
                                     Err(_) => {
                                         eprintln!("Logger shutdown timed out after 5 seconds");
                                         false
                                     }
                                 }
                             });
                             
                             // Wait for the shutdown to complete
                             match rt.block_on(handle) {
                                 Ok(true) => return Ok(()),
                                 Ok(false) => return Err("Logger shutdown failed".to_string()),
                                 Err(e) => return Err(format!("Logger shutdown task panicked: {}", e)),
                             }
                         }
                     }
                 }
             }
             
             // If we can't do an async shutdown, still try to flush any file outputs
             if let Some(logger) = LOGGER_INSTANCE.get() {
                 if let Ok(mut guard) = logger.lock() {
                     if let Some(ref mut output) = guard.output {
                         // For non-async loggers, write an empty message which will trigger a flush
                         let _ = output.write_log("");
                     }
                 }
             }
             
             println!("Logger shutdown completed");
             Ok(())
         } else {
             // No runtime means no async logging was initialized
             println!("No async logger to shutdown");
             Ok(())
         }
     }
     
     /// Get the number of dropped log messages due to backpressure
     pub fn get_dropped_log_count() -> u64 {
         if let Some(logger) = LOGGER_INSTANCE.get() {
             if let Ok(logger_guard) = logger.lock() {
                 return logger_guard.dropped_logs.load(Ordering::Relaxed);
             }
         }
         0
     }
 }
 
 // Ensure the logger is properly shutdown when the program exits
 impl Drop for Logger {
     fn drop(&mut self) {
         let _ = Self::shutdown();
     }
 }
 