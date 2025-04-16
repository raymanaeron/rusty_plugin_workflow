use std::fs::{OpenOptions, rename};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use serde_json;
use crate::log_contracts::LogDestination;
use crate::log_contracts::LogEntry;

pub struct FileLogDestination {
    file: Mutex<std::fs::File>,
    path: PathBuf,
    max_size: u64,
}

impl FileLogDestination {
    pub fn new(path: PathBuf, max_size: u64) -> Self {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .expect("Unable to open log file");

        Self {
            file: Mutex::new(file),
            path,
            max_size,
        }
    }

    fn rotate(&self) {
        let file = self.file.lock().unwrap();
        if let Ok(metadata) = file.metadata() {
            if metadata.len() > self.max_size {
                drop(file);
                let _ = rename(&self.path, self.path.with_extension("log.bak"));
                let new_file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&self.path)
                    .expect("Failed to open new log file");
                *self.file.lock().unwrap() = new_file;
            }
        }
    }
}

impl LogDestination for FileLogDestination {
    fn write_log(&self, entry: &LogEntry) {
        self.rotate();
        let json = serde_json::to_string(entry).unwrap();
        let mut file = self.file.lock().unwrap();
        let _ = writeln!(file, "{}", json);
    }
}
