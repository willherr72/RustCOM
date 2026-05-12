use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("port not open: tab {0}")]
    PortNotOpen(u32),
    #[error("serial error: {0}")]
    Serial(String),
    #[error("invalid input: {0}")]
    Invalid(String),
    #[error("io error: {0}")]
    Io(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<serialport::Error> for AppError {
    fn from(e: serialport::Error) -> Self {
        AppError::Serial(e.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e.to_string())
    }
}

pub type AppResult<T> = std::result::Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_to_message_string() {
        let err = AppError::PortNotOpen(7);
        let json = serde_json::to_string(&err).unwrap();
        assert_eq!(json, "\"port not open: tab 7\"");
    }

    #[test]
    fn from_io_error_preserves_message() {
        let io = std::io::Error::new(std::io::ErrorKind::TimedOut, "boom");
        let app: AppError = io.into();
        assert!(app.to_string().contains("boom"));
    }
}
