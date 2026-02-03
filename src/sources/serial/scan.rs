use regex::Regex;

use crate::core::{AppError, AppResult};

pub fn list_available_ports(regex: Option<String>) -> AppResult<Vec<String>> {
    let all_ports = tokio_serial::available_ports().map_err(|e| AppError::Scan(e.to_string()))?;
    let mut matching_ports = Vec::new();

    let user_re = if let Some(pattern) = regex {
        Some(Regex::new(&pattern).map_err(|e| AppError::Scan(e.to_string()))?)
    } else {
        None
    };

    for p in all_ports {
        let name = p.port_name;

        let keep = match &user_re {
            Some(re) => re.is_match(&name),
            None => is_system_port(&name),
        };

        if keep {
            matching_ports.push(name);
        }
    }

    matching_ports.sort();
    Ok(matching_ports)
}

#[cfg(target_os = "macos")]
fn is_system_port(name: &str) -> bool {
    if !name.starts_with("/dev/tty.") && !name.starts_with("/dev/cu.") {
        return false;
    }
    let re = Regex::new(r"(?i)(usbserial|usbmodem|jlink)").unwrap();
    re.is_match(name)
}

#[cfg(target_os = "linux")]
fn is_system_port(name: &str) -> bool {
    let re = Regex::new(r"/dev/(ttyACM\d+|ttyUSB\d+)").unwrap();
    re.is_match(name)
}

#[cfg(target_os = "windows")]
fn is_system_port(name: &str) -> bool {
    let re = Regex::new(r"(?i)^COM\d+$").unwrap();
    re.is_match(name)
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn is_system_port(_name: &str) -> bool {
    true
}
