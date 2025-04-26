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

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;
use reqwest::blocking::Client;
use serde::Serialize;
use crate::config::{LogConfig, LogType, LogLevel};

pub trait LogOutput: Send + Sync {
    fn write_log(&mut self, 
                timestamp: &str,
                level: &LogLevel, 
                message: &str, 
                file: &str, 
                line: u32, 
                module: &str,
                context: Option<&str>) -> Result<(), String>;
}

pub struct ConsoleOutput;

impl LogOutput for ConsoleOutput {
    fn write_log(&mut self, 
                timestamp: &str,
                level: &LogLevel, 
                message: &str, 
                file: &str, 
                line: u32, 
                module: &str,
                context: Option<&str>) -> Result<(), String> {
        let log_line = if let Some(ctx) = context {
            format!("{} [{}] [{}:{}] [{}] {} | Context: {}", 
                timestamp, level.as_str(), file, line, module, message, ctx)
        } else {
            format!("{} [{}] [{}:{}] [{}] {}", 
                timestamp, level.as_str(), file, line, module, message)
        };
        
        if let Err(e) = writeln!(io::stdout(), "{}", log_line) {
            return Err(format!("Failed to write to console: {}", e));
        }
        
        Ok(())
    }
}

pub struct FileOutput {
    file_path: PathBuf,
    log_folder: String,
    max_file_size_bytes: u64,
    current_file: Option<File>,
    current_size: u64,
}

impl FileOutput {
    pub fn new(config: &LogConfig) -> Result<Self, String> {
        // Create log folder regardless of ensure_log_folder_exists
        let folder_path = Path::new(&config.log_folder);
        if !folder_path.exists() {
            println!("[Logger] Creating log directory: {:?}", folder_path);
            fs::create_dir_all(folder_path)
                .map_err(|e| format!("Failed to create log directory: {}", e))?;
        }
        
        let file_path = folder_path.join(&config.file_path);
        println!("[Logger] Log file will be created at: {:?}", file_path);
        
        let max_file_size_bytes = config.max_file_size_mb * 1024 * 1024;
        
        let mut output = FileOutput {
            file_path,
            log_folder: config.log_folder.clone(),
            max_file_size_bytes,
            current_file: None,
            current_size: 0,
        };
        
        // Immediately try to open the file to confirm it works
        output.open_or_rotate()?;
        println!("[Logger] Log file opened successfully");
        
        Ok(output)
    }

    fn open_or_rotate(&mut self) -> Result<(), String> {
        // Check if file exists and get its size
        let file_exists = self.file_path.exists();
        let current_size = if file_exists {
            fs::metadata(&self.file_path)
                .map_err(|e| format!("Failed to get file metadata: {}", e))?
                .len()
        } else {
            0
        };
        
        if file_exists && current_size >= self.max_file_size_bytes {
            self.rotate_logs()?;
            self.current_size = 0;
        } else {
            self.current_size = current_size;
        }
        
        // Ensure directory exists before opening file
        if let Some(parent) = self.file_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create directories for log file: {}", e))?;
            }
        }
        
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)
            .map_err(|e| format!("Failed to open log file: {}", e))?;
            
        self.current_file = Some(file);
        
        Ok(())
    }
    
    fn rotate_logs(&self) -> Result<(), String> {
        // Find the highest numbered backup file
        let file_stem = self.file_path.file_stem()
            .and_then(|s| s.to_str())
            .ok_or("Invalid file path")?;
            
        let extension = self.file_path.extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");
            
        let mut max_index = 0;
        
        if let Ok(entries) = fs::read_dir(&self.log_folder) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                    let backup_prefix = format!("{}.{}", file_stem, extension);
                    if file_name.starts_with(&backup_prefix) {
                        if let Some(index_str) = file_name.strip_prefix(&format!("{}.", backup_prefix)) {
                            if let Ok(index) = index_str.parse::<u32>() {
                                max_index = max_index.max(index);
                            }
                        }
                    }
                }
            }
        }
        
        // Rotate files
        let new_path = self.file_path.with_extension(format!("{}.{}", extension, max_index + 1));
        fs::rename(&self.file_path, new_path)
            .map_err(|e| format!("Failed to rotate log file: {}", e))?;
            
        Ok(())
    }
}

impl LogOutput for FileOutput {
    fn write_log(&mut self, 
                timestamp: &str,
                level: &LogLevel, 
                message: &str, 
                file: &str, 
                line: u32, 
                module: &str,
                context: Option<&str>) -> Result<(), String> {
        // Make sure we have a file open
        if self.current_file.is_none() {
            self.open_or_rotate()?;
        }
        
        let log_line = if let Some(ctx) = context {
            format!("{} [{}] [{}:{}] [{}] {} | Context: {}\n", 
                timestamp, level.as_str(), file, line, module, message, ctx)
        } else {
            format!("{} [{}] [{}:{}] [{}] {}\n", 
                timestamp, level.as_str(), file, line, module, message)
        };
        
        let bytes = log_line.as_bytes();
        
        // Fix borrowing issues by restructuring the code
        // First, check if we need to rotate
        let need_rotation = {
            if let Some(_file) = &self.current_file {
                self.current_size + bytes.len() as u64 > self.max_file_size_bytes
            } else {
                false
            }
        };
        
        // If needed, rotate logs and reopen the file
        if need_rotation {
            // Close the current file by replacing it with None
            self.current_file = None;
            self.rotate_logs()?;
            self.open_or_rotate()?;
        }
        
        // Write to the file
        if let Some(file) = &mut self.current_file {
            if let Err(e) = file.write_all(bytes) {
                return Err(format!("Failed to write to log file: {}", e));
            }
            
            if let Err(e) = file.flush() {
                return Err(format!("Failed to flush log file: {}", e));
            }
            
            self.current_size += bytes.len() as u64;
        }
        
        Ok(())
    }
}

#[derive(Serialize)]
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

pub struct HttpOutput {
    client: Client,
    endpoint: String,
}

impl HttpOutput {
    pub fn new(config: &LogConfig) -> Result<Self, String> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.http_timeout_seconds))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
            
        Ok(HttpOutput {
            client,
            endpoint: config.http_endpoint.clone(),
        })
    }
}

impl LogOutput for HttpOutput {
    fn write_log(&mut self, 
                timestamp: &str,
                level: &LogLevel, 
                message: &str, 
                file: &str, 
                line: u32, 
                module: &str,
                context: Option<&str>) -> Result<(), String> {
        let payload = LogPayload {
            timestamp,
            level: level.as_str(),
            message,
            file,
            line,
            module,
            context,
        };
        
        match self.client.post(&self.endpoint)
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

pub fn create_log_output(config: &LogConfig) -> Result<Box<dyn LogOutput>, String> {
    match config.log_type {
        LogType::Console => Ok(Box::new(ConsoleOutput {})),
        LogType::File => {
            let file_output = FileOutput::new(config)?;
            Ok(Box::new(file_output))
        },
        LogType::Http => {
            let http_output = HttpOutput::new(config)?;
            Ok(Box::new(http_output))
        }
    }
}
