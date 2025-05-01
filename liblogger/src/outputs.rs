/*
 * Log output implementations
 * 
 * This module defines different logging backends:
 * - ConsoleOutput: Writes logs to stdout
 * - FileOutput: Writes logs to files with rotation support
 * - HttpOutput: Sends logs to a remote endpoint
 * 
 * Each output implements the LogOutput trait, which defines how
 * log messages are formatted and written. The module also provides
 * factory functions to create the appropriate output based on configuration.
 */

 use std::fs::{File, OpenOptions};
 use std::io::{self, Write};
 use std::path::Path;
 use std::sync::{Arc, Mutex};
 use std::time::Duration;
 use tokio::io::{AsyncWriteExt, stdout};
 use reqwest::{Client, blocking::Client as BlockingClient};
 use serde::{Serialize, Deserialize};
 use crate::config::{LogConfig, LogType};
 use async_trait::async_trait;
 
 // Original synchronous trait, kept for backward compatibility
 pub trait LogOutput: Send + Sync {
     fn write_log(&mut self, formatted_message: &str) -> Result<(), String>;
 }
 
 // Instead of using an async trait directly, define a trait with a function
 // that returns a future boxed to make it object-safe
 #[async_trait]
 pub trait AsyncLogOutputTrait: Send + Sync {
     async fn write_log_async(&mut self, formatted_message: &str) -> Result<(), String>;
 }
 
 // Enum to hold all possible output types
 pub enum AsyncLogOutput {
     Console(ConsoleOutput),
     File(AsyncFileOutput),
     Http(HttpOutput),
 }
 
 // Console output implementation
 pub struct ConsoleOutput;
 
 impl ConsoleOutput {
     pub fn new() -> Self {
         ConsoleOutput {}
     }
 }
 
 impl LogOutput for ConsoleOutput {
     fn write_log(&mut self, formatted_message: &str) -> Result<(), String> {
         if let Err(e) = writeln!(io::stdout(), "{}", formatted_message) {
             return Err(format!("Failed to write to console: {}", e));
         }
         
         Ok(())
     }
 }
 
 #[async_trait]
 impl AsyncLogOutputTrait for ConsoleOutput {
     async fn write_log_async(&mut self, formatted_message: &str) -> Result<(), String> {
         let mut stdout = stdout();
         let mut log_bytes = formatted_message.as_bytes().to_vec();
         log_bytes.push(b'\n');
         
         if let Err(e) = stdout.write_all(&log_bytes).await {
             return Err(format!("Failed to write to console: {}", e));
         }
         
         if let Err(e) = stdout.flush().await {
             return Err(format!("Failed to flush console output: {}", e));
         }
         
         Ok(())
     }
 }
 
 // Update the FileOutput struct to include force_flush flag
 pub struct FileOutput {
     file_handle: Arc<Mutex<File>>,
     force_flush: bool,
 }
 
 impl FileOutput {
     #[allow(dead_code)]
     pub fn new(file_path: &str, force_flush: bool) -> Result<Self, String> {
         // Create directory if it doesn't exist
         if let Some(parent) = Path::new(file_path).parent() {
             if !parent.exists() {
                 std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create log directory: {}", e))?;
             }
         }
         
         // Open the file once with append mode
         let file = OpenOptions::new()
             .create(true)
             .append(true)
             .open(file_path)
             .map_err(|e| format!("Failed to open log file: {}", e))?;
         
         // Wrap the file in Arc<Mutex<_>> for shared access
         let file_handle = Arc::new(Mutex::new(file));
         
         Ok(FileOutput {
             file_handle,
             force_flush,
         })
     }
 }
 
 impl LogOutput for FileOutput {
     fn write_log(&mut self, formatted_message: &str) -> Result<(), String> {
         // Lock the file handle and write to it
         let mut file = self.file_handle.lock()
             .map_err(|_| "Failed to lock file mutex".to_string())?;
         
         file.write_all(formatted_message.as_bytes())
             .map_err(|e| format!("Failed to write to log file: {}", e))?;
         file.write_all(b"\n")
             .map_err(|e| format!("Failed to write newline to log file: {}", e))?;
         
         // Only flush immediately if force_flush is true
         if self.force_flush {
             file.flush()
                 .map_err(|e| format!("Failed to flush log file: {}", e))?;
         }
         
         Ok(())
     }
 }
 
 // Update AsyncFileOutput to include force_flush flag
 pub struct AsyncFileOutput {
     file_handle: Arc<Mutex<File>>,
     force_flush: bool,
 }
 
 // Implementation of AsyncFileOutput
 impl AsyncFileOutput {
     #[allow(dead_code)]
     pub fn new(file_path: &str, force_flush: bool) -> Result<Self, String> {
         // Create directory if it doesn't exist
         if let Some(parent) = Path::new(file_path).parent() {
             if !parent.exists() {
                 std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create log directory: {}", e))?;
             }
         }
         
         // Open the file once with append mode
         let file = OpenOptions::new()
             .create(true)
             .append(true)
             .open(file_path)
             .map_err(|e| format!("Failed to open log file: {}", e))?;
             
         // Wrap the file in Arc<Mutex<_>> for shared access
         let file_handle = Arc::new(Mutex::new(file));
         
         Ok(AsyncFileOutput {
             file_handle,
             force_flush,
         })
     }
 }
 
 #[async_trait]
 impl AsyncLogOutputTrait for AsyncFileOutput {
     async fn write_log_async(&mut self, formatted_message: &str) -> Result<(), String> {
         // Lock the file handle and write to it
         let mut file = self.file_handle.lock()
             .map_err(|_| "Failed to lock file mutex".to_string())?;
             
         file.write_all(formatted_message.as_bytes())
             .map_err(|e| format!("Failed to write to log file: {}", e))?;
         file.write_all(b"\n")
             .map_err(|e| format!("Failed to write newline to log file: {}", e))?;
         
         // Only flush immediately if force_flush is true
         if self.force_flush {
             file.flush()
                 .map_err(|e| format!("Failed to flush log file: {}", e))?;
         }
         
         Ok(())
     }
 }
 
 // Update the create_file_output function to include force_flush
 pub fn create_file_output(file_path: &str, force_flush: bool) -> Result<(FileOutput, AsyncFileOutput), String> {
     // Open the file once 
     let file = OpenOptions::new()
         .create(true)
         .append(true)
         .open(file_path)
         .map_err(|e| format!("Failed to open log file: {}", e))?;
         
     // Create shared file handle
     let file_handle = Arc::new(Mutex::new(file));
     
     // Create both output instances with the same file handle and force_flush setting
     let file_output = FileOutput {
         file_handle: Arc::clone(&file_handle),
         force_flush,
     };
     
     let async_file_output = AsyncFileOutput {
         file_handle,
         force_flush,
     };
     
     Ok((file_output, async_file_output))
 }
 
 #[derive(Serialize, Deserialize)]
 struct LogPayload<'a> {
     timestamp: &'a str,
     level: &'a str,
     message: &'a str,
     file: &'a str,
     line: u32,
     module: &'a str,
     #[serde(skip_serializing_if = "Option::is_none")]
     context: Option<&'a str>,
 }
 
 // HTTP output implementation - updated to support async operations
 pub struct HttpOutput {
     blocking_client: BlockingClient,
     async_client: Client,
     endpoint: String,
 }
 
 impl HttpOutput {
     pub fn new(endpoint: &str, timeout_seconds: u64) -> Result<Self, String> {
         let blocking_client = BlockingClient::builder()
             .timeout(Duration::from_secs(timeout_seconds))
             .build()
             .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
             
         let async_client = Client::builder()
             .timeout(Duration::from_secs(timeout_seconds))
             .build()
             .map_err(|e| format!("Failed to create async HTTP client: {}", e))?;
             
         Ok(HttpOutput {
             blocking_client,
             async_client,
             endpoint: endpoint.to_string(),
         })
     }
 }
 
 impl LogOutput for HttpOutput {
     fn write_log(&mut self, formatted_message: &str) -> Result<(), String> {
         let payload: LogPayload = serde_json::from_str(formatted_message)
             .map_err(|e| format!("Failed to parse log payload: {}", e))?;
         
         match self.blocking_client.post(&self.endpoint)
             .json(&payload)
             .send() {
             Ok(response) => {
                 if !response.status().is_success() {
                     return Err(format!("HTTP log failed with status: {}", response.status()));
                 }
             },
             Err(e) => {
                 return Err(format!("Failed to send HTTP log: {}", e));
             }
         }
         
         Ok(())
     }
 }
 
 #[async_trait]
 impl AsyncLogOutputTrait for HttpOutput {
     async fn write_log_async(&mut self, formatted_message: &str) -> Result<(), String> {
         let payload: LogPayload = serde_json::from_str(formatted_message)
             .map_err(|e| format!("Failed to parse log payload: {}", e))?;
         
         let response = match self.async_client.post(&self.endpoint)
             .json(&payload)
             .send()
             .await {
                 Ok(resp) => resp,
                 Err(e) => return Err(format!("Failed to send HTTP log: {}", e))
             };
         
         if !response.status().is_success() {
             return Err(format!("HTTP log failed with status: {}", response.status()));
         }
         
         Ok(())
     }
 }
 
 // Implement AsyncLogOutputTrait for the AsyncLogOutput enum
 #[async_trait]
 impl AsyncLogOutputTrait for AsyncLogOutput {
     async fn write_log_async(&mut self, formatted_message: &str) -> Result<(), String> {
         match self {
             AsyncLogOutput::Console(output) => output.write_log_async(formatted_message).await,
             AsyncLogOutput::File(output) => output.write_log_async(formatted_message).await,
             AsyncLogOutput::Http(output) => output.write_log_async(formatted_message).await,
         }
     }
 }
 
 /// Creates a synchronous log output based on configuration
 pub fn create_log_output(log_type: &LogType) -> Result<Box<dyn LogOutput>, String> {
     match log_type {
         LogType::Console => Ok(Box::new(ConsoleOutput::new())),
         LogType::File => {
             // Get the config instance to retrieve settings
             let config = LogConfig::get_instance()?;
             
             // Get file path and combine with log folder if specified
             let file_path = config.file_path.as_ref()
                 .ok_or_else(|| "File path not specified in configuration".to_string())?;
                 
             // Construct the full path using the log_folder if provided
             let full_path = if let Some(folder) = &config.log_folder {
                 // Create the log directory if it doesn't exist
                 std::fs::create_dir_all(folder)
                     .map_err(|e| format!("Failed to create log directory '{}': {}", folder, e))?;
                 
                 // Use platform-specific path separator
                 let path = Path::new(folder).join(file_path);
                 path.to_string_lossy().into_owned()
             } else {
                 file_path.clone()
             };
             
             println!("Creating log file at: {}", full_path);
             
             // Use the force_flush directly since it's already a bool
             let force_flush = config.force_flush;
             
             let (file_output, _) = create_file_output(&full_path, force_flush)?;
             Ok(Box::new(file_output))
         },
         LogType::Http => {
             // Assuming the config is properly updated to include http_endpoint and http_timeout_seconds
             let config = LogConfig::get_instance()?;
             let endpoint = &config.http_endpoint.as_ref().ok_or_else(|| 
                 "HTTP endpoint not specified in configuration".to_string())?;
             let timeout = config.http_timeout_seconds.unwrap_or(30);
             Ok(Box::new(HttpOutput::new(endpoint, timeout)?))
         },
     }
 }
 
 /// Creates an asynchronous log output based on configuration
 pub fn create_async_log_output(log_type: &LogType) -> Result<AsyncLogOutput, String> {
     match log_type {
         LogType::Console => Ok(AsyncLogOutput::Console(ConsoleOutput::new())),
         LogType::File => {
             // Get the config instance to retrieve settings
             let config = LogConfig::get_instance()?;
             
             // Get file path and combine with log folder if specified
             let file_path = config.file_path.as_ref()
                 .ok_or_else(|| "File path not specified in configuration".to_string())?;
                 
             // Construct the full path using the log_folder if provided
             let full_path = if let Some(folder) = &config.log_folder {
                 // Create the log directory if it doesn't exist
                 std::fs::create_dir_all(folder)
                     .map_err(|e| format!("Failed to create log directory '{}': {}", folder, e))?;
                 
                 // Use platform-specific path separator
                 let path = Path::new(folder).join(file_path);
                 path.to_string_lossy().into_owned()
             } else {
                 file_path.clone()
             };
             
             // Use the force_flush directly since it's already a bool
             let force_flush = config.force_flush;
             
             let (_, async_file_output) = create_file_output(&full_path, force_flush)?;
             Ok(AsyncLogOutput::File(async_file_output))
         },
         LogType::Http => {
             // Get the config instance to retrieve HTTP settings
             let config = LogConfig::get_instance()?;
             let endpoint = &config.http_endpoint.as_ref().ok_or_else(|| 
                 "HTTP endpoint not specified in configuration".to_string())?;
             let timeout = config.http_timeout_seconds.unwrap_or(30);
             Ok(AsyncLogOutput::Http(HttpOutput::new(endpoint, timeout)?))
         },
     }
 }
 