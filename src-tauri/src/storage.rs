use std::fs;
use std::path::PathBuf;

use serde::{de::DeserializeOwned, Serialize};

use crate::error::{AppError, AppResult};

pub fn data_dir() -> AppResult<PathBuf> {
    let base = dirs::data_local_dir()
        .ok_or_else(|| AppError::Io("could not resolve user data dir".to_string()))?;
    let dir = base.join("rustcom");
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| AppError::Io(e.to_string()))?;
    }
    Ok(dir)
}

pub fn read_json<T: DeserializeOwned + Default>(file_name: &str) -> AppResult<T> {
    let path = data_dir()?.join(file_name);
    if !path.exists() {
        return Ok(T::default());
    }
    let body = fs::read_to_string(&path).map_err(|e| AppError::Io(e.to_string()))?;
    serde_json::from_str::<T>(&body).map_err(|e| AppError::Invalid(e.to_string()))
}

pub fn write_json<T: Serialize>(file_name: &str, value: &T) -> AppResult<()> {
    let dir = data_dir()?;
    let final_path = dir.join(file_name);
    let tmp_path = dir.join(format!("{file_name}.tmp"));

    let body = serde_json::to_string_pretty(value).map_err(|e| AppError::Invalid(e.to_string()))?;
    fs::write(&tmp_path, body).map_err(|e| AppError::Io(e.to_string()))?;
    fs::rename(&tmp_path, &final_path).map_err(|e| AppError::Io(e.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Default, Serialize, Deserialize, PartialEq, Debug)]
    struct Sample { name: String, count: u32 }

    #[test]
    fn read_returns_default_when_file_missing() {
        // Use a unique file name so we don't collide with real data on the dev machine.
        let _: Sample = read_json("rustcom_test_missing_xyz_42.json").unwrap();
    }

    #[test]
    fn round_trip_value() {
        let v = Sample { name: "alpha".into(), count: 7 };
        write_json("rustcom_test_roundtrip.json", &v).unwrap();
        let back: Sample = read_json("rustcom_test_roundtrip.json").unwrap();
        assert_eq!(back, v);
        // Cleanup so repeated runs stay clean.
        let _ = fs::remove_file(data_dir().unwrap().join("rustcom_test_roundtrip.json"));
    }
}
