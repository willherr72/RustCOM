// Virtual COM Port support
// Platform-specific implementation for creating virtual serial port pairs

#[cfg(target_os = "windows")]
pub mod windows {
    use std::process::Command;

    pub fn create_virtual_pair() -> Result<(String, String), String> {
        match find_com0com_pairs() {
            Ok(pairs) if !pairs.is_empty() => {
                let (port1, port2) = &pairs[0];
                return Ok((port1.clone(), port2.clone()));
            }
            _ => {}
        }

        if let Ok(pair) = try_create_with_setupc() {
            return Ok(pair);
        }

        Err("com0com is installed but no port pairs are configured.\n\n\
             To create a virtual COM pair:\n\
             1. Run 'setupc.exe' as Administrator\n\
             2. Use command: install PortName=COM98 PortName=COM99\n\
             3. Or use the com0com GUI to create a pair\n\n\
             Then refresh and try again.\n\n\
             Alternative: The COM port pairs may already exist.\n\
             Check your device manager for 'com0com' ports.".to_string())
    }

    fn find_com0com_pairs() -> Result<Vec<(String, String)>, String> {
        let ports = serialport::available_ports().map_err(|e| e.to_string())?;

        let mut com0com_ports: Vec<String> = Vec::new();

        for port in ports {
            let port_name = port.port_name.clone();
            if port_name.starts_with("COM") {
                com0com_ports.push(port_name);
            }
        }

        if com0com_ports.len() >= 2 {
            com0com_ports.sort();
            let mut pairs = Vec::new();

            for i in (0..com0com_ports.len()).step_by(2) {
                if i + 1 < com0com_ports.len() {
                    pairs.push((com0com_ports[i].clone(), com0com_ports[i + 1].clone()));
                }
            }

            if !pairs.is_empty() {
                return Ok(pairs);
            }
        }

        Err("No com0com port pairs found".to_string())
    }

    fn try_create_with_setupc() -> Result<(String, String), String> {
        let possible_paths = vec![
            r"C:\Program Files\com0com\setupc.exe",
            r"C:\Program Files (x86)\com0com\setupc.exe",
            r"setupc.exe",
        ];

        for setupc_path in possible_paths {
            match Command::new(setupc_path)
                .args(["install", "PortName=COM98", "PortName=COM99"])
                .output()
            {
                Ok(output) => {
                    if output.status.success() {
                        return Ok(("COM98".to_string(), "COM99".to_string()));
                    }
                }
                Err(_) => continue,
            }
        }

        Err("Could not create COM pair with setupc".to_string())
    }

}

#[cfg(target_os = "linux")]
pub mod linux {
    use std::process::Command;

    pub fn create_virtual_pair() -> Result<(String, String), String> {
        let output = Command::new("socat")
            .arg("-d")
            .arg("-d")
            .arg("pty,raw,echo=0,link=/tmp/vcom0")
            .arg("pty,raw,echo=0,link=/tmp/vcom1")
            .spawn();

        match output {
            Ok(_) => Ok(("/tmp/vcom0".to_string(), "/tmp/vcom1".to_string())),
            Err(e) => Err(format!(
                "Failed to create virtual COM pair: {}. \
                 Make sure 'socat' is installed.",
                e
            )),
        }
    }
}

#[cfg(target_os = "macos")]
pub mod macos {
    pub fn create_virtual_pair() -> Result<(String, String), String> {
        Err("Virtual COM pairs on macOS require socat. \
             Install with: brew install socat"
            .to_string())
    }
}

// Cross-platform interface
pub fn create_loopback_pair() -> Result<(String, String), String> {
    #[cfg(target_os = "windows")]
    return windows::create_virtual_pair();

    #[cfg(target_os = "linux")]
    return linux::create_virtual_pair();

    #[cfg(target_os = "macos")]
    return macos::create_virtual_pair();

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    return Err("Virtual COM ports not supported on this platform".to_string());
}

