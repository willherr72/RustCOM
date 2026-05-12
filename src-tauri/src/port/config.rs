use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DataBits { Five, Six, Seven, Eight }

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StopBits { One, Two }

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Parity { None, Even, Odd }

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FlowControl { None, Software, Hardware }

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LineEnding {
    None,
    Cr,
    Lf,
    #[serde(rename = "crlf")]
    CrLf,
}

impl LineEnding {
    pub fn as_bytes(&self) -> &'static [u8] {
        match self {
            LineEnding::None => b"",
            LineEnding::Cr => b"\r",
            LineEnding::Lf => b"\n",
            LineEnding::CrLf => b"\r\n",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SendMode { Ascii, Hex }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortConfig {
    pub port_name: String,
    pub baud_rate: u32,
    pub data_bits: DataBits,
    pub stop_bits: StopBits,
    pub parity: Parity,
    pub flow_control: FlowControl,
    #[serde(default)]
    pub auto_reconnect: bool,
    #[serde(default = "default_reconnect_delay")]
    pub reconnect_delay_ms: u64,
}

fn default_reconnect_delay() -> u64 { 2000 }

impl PortConfig {
    pub fn to_serial(&self) -> serialport::SerialPortBuilder {
        serialport::new(&self.port_name, self.baud_rate)
            .data_bits(match self.data_bits {
                DataBits::Five => serialport::DataBits::Five,
                DataBits::Six => serialport::DataBits::Six,
                DataBits::Seven => serialport::DataBits::Seven,
                DataBits::Eight => serialport::DataBits::Eight,
            })
            .stop_bits(match self.stop_bits {
                StopBits::One => serialport::StopBits::One,
                StopBits::Two => serialport::StopBits::Two,
            })
            .parity(match self.parity {
                Parity::None => serialport::Parity::None,
                Parity::Even => serialport::Parity::Even,
                Parity::Odd => serialport::Parity::Odd,
            })
            .flow_control(match self.flow_control {
                FlowControl::None => serialport::FlowControl::None,
                FlowControl::Software => serialport::FlowControl::Software,
                FlowControl::Hardware => serialport::FlowControl::Hardware,
            })
            .timeout(std::time::Duration::from_millis(10))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_ending_bytes() {
        assert_eq!(LineEnding::None.as_bytes(), b"");
        assert_eq!(LineEnding::Cr.as_bytes(), b"\r");
        assert_eq!(LineEnding::Lf.as_bytes(), b"\n");
        assert_eq!(LineEnding::CrLf.as_bytes(), b"\r\n");
    }

    #[test]
    fn config_serde_roundtrip() {
        let cfg = PortConfig {
            port_name: "COM3".to_string(),
            baud_rate: 115200,
            data_bits: DataBits::Eight,
            stop_bits: StopBits::One,
            parity: Parity::None,
            flow_control: FlowControl::None,
            auto_reconnect: false,
            reconnect_delay_ms: 2000,
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let back: PortConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.baud_rate, 115200);
        assert_eq!(back.data_bits, DataBits::Eight);
    }

    #[test]
    fn config_default_reconnect_delay() {
        let json = r#"{
            "port_name": "COM1",
            "baud_rate": 9600,
            "data_bits": "eight",
            "stop_bits": "one",
            "parity": "none",
            "flow_control": "none"
        }"#;
        let cfg: PortConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.reconnect_delay_ms, 2000);
        assert!(!cfg.auto_reconnect);
    }
}
