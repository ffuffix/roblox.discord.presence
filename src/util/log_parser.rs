use regex::Regex;
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::Read;
use std::path::PathBuf;
use std::time::SystemTime;

#[cfg(target_os = "windows")]
use std::os::windows::fs::OpenOptionsExt;

#[cfg(target_os = "windows")]
const FILE_SHARE_READ: u32 = 1;
#[cfg(target_os = "windows")]
const FILE_SHARE_WRITE: u32 = 2;
#[cfg(target_os = "windows")]
const FILE_SHARE_DELETE: u32 = 4;

use super::paths;

pub struct LogReader {
    file: File,
    buffer: String,
}

impl LogReader {
    pub fn new(log_path: &PathBuf) -> Result<Self, std::io::Error> {
        let mut opts = OpenOptions::new();
        opts.read(true);

        #[cfg(target_os = "windows")]
        opts.share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE);
        
        let file = opts.open(log_path)?;

        Ok(LogReader {
            file,
            buffer: String::new(),
        })
    }

    pub fn get_new_lines(&mut self) -> Vec<String> {
        let mut found_lines = Vec::new();
        let mut chunk = Vec::new();

        if let Ok(_) = self.file.read_to_end(&mut chunk) {
            if !chunk.is_empty() {
                let chunk_str = String::from_utf8_lossy(&chunk);
                self.buffer.push_str(&chunk_str);
            }
        }

        while let Some(newline_idx) = self.buffer.find('\n') {
            let line: String = self.buffer.drain(..=newline_idx).collect();
            found_lines.push(line.trim().to_string());
        }

        found_lines
    }
}

pub fn get_place_id_from_line(line: &str) -> Option<String> {
    let patterns = [
        r"Launching experience at (\d+)",
        r"! Joining game .* place (\d+)",
        r"Joining game .* place (\d+)",
        r"placeid:(\d+)",
        r"placeId:(\d+)",
        r"PlaceId=(\d+)",
        r"universeId:(\d+)",
    ];

    for pattern in &patterns {
        let re = Regex::new(pattern).unwrap();
        if let Some(caps) = re.captures(line) {
            if let Some(id) = caps.get(1) {
                return Some(id.as_str().to_string());
            }
        }
    }

    None
}

pub struct LogMonitor {
    reader: Option<(PathBuf, LogReader)>,
}

impl LogMonitor {
    pub fn new() -> Self {
        LogMonitor { reader: None }
    }

    pub fn check_latest_log(&mut self) -> Option<String> {
        let latest_path = get_latest_log_path()?;

        let needs_new_reader = match &self.reader {
            Some((path, _)) => path != &latest_path,
            None => true,
        };

        if needs_new_reader {
            println!("[LOGS] Switching to log file: {:?}", latest_path.file_name().unwrap());
            match LogReader::new(&latest_path) {
                Ok(reader) => {
                    self.reader = Some((latest_path.clone(), reader));
                }
                Err(e) => {
                    eprintln!("[LOGS] Failed to open log: {}", e);
                    return None;
                }
            }
        }

        if let Some((_, reader)) = self.reader.as_mut() {
            let lines = reader.get_new_lines();
            for line in lines {
                if let Some(id) = get_place_id_from_line(&line) {
                    return Some(id);
                }
            }
        }

        None
    }

    pub fn clear(&mut self) {
        self.reader = None;
    }
}

fn get_latest_log_path() -> Option<PathBuf> {
    let logs_dir = paths::roblox_logs()?;
    
    let mut entries: Vec<_> = fs::read_dir(logs_dir).ok()?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().extension().map(|e| e == "log").unwrap_or(false)
        })
        .collect();

    entries.sort_by_key(|entry| {
        entry.metadata().and_then(|m| m.modified()).unwrap_or(SystemTime::UNIX_EPOCH)
    });

    entries.last().map(|e| e.path())
}