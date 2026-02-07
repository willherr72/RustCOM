use std::fs::OpenOptions;
use std::io::Write as IoWrite;
use chrono::Local;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Received,
    Sent,
}

#[derive(Debug, Clone)]
pub struct DataLogEntry {
    pub timestamp: String,
    pub direction: Direction,
    pub data: Vec<u8>,
}

pub fn create_log_entry(direction: Direction, data: &[u8]) -> DataLogEntry {
    DataLogEntry {
        timestamp: Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
        direction,
        data: data.to_vec(),
    }
}

pub fn save_log(entries: &[DataLogEntry], path: &str) -> Result<String, String> {
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path)
        .map_err(|_| "Failed to save log".to_string())?;

    for entry in entries {
        let dir_str = match entry.direction {
            Direction::Received => "RX",
            Direction::Sent => "TX",
        };
        let data_str = String::from_utf8_lossy(&entry.data);
        let line = format!("[{}] {}: {}\n", entry.timestamp, dir_str, data_str);
        let _ = file.write_all(line.as_bytes());
    }
    Ok(format!("Log saved to {}", path))
}

pub fn save_buffer(display: &str) {
    let filename = format!("capture_{}.txt", Local::now().format("%Y%m%d_%H%M%S"));
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&filename)
    {
        let _ = file.write_all(display.as_bytes());
    }
}
