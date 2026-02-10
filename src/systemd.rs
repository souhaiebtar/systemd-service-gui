use std::process::Command;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub description: String,
    pub load_state: String,
    pub active_state: String,
    pub sub_state: String,
    pub unit_file_state: String,
    pub followed_by: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub name: String,
    pub active: bool,
    pub running: bool,
    pub pid: Option<u32>,
}

impl ServiceInfo {
    pub fn is_active(&self) -> bool {
        self.active_state == "active"
    }

    pub fn is_running(&self) -> bool {
        self.sub_state == "running"
    }
}

pub fn list_services() -> Result<Vec<ServiceInfo>, String> {
    let output = Command::new("systemctl")
        .args(&["list-units", "--type=service", "--all", "--no-pager", "--output=json"])
        .output()
        .map_err(|e| format!("Failed to execute systemctl: {}", e))?;

    if !output.status.success() {
        return Err(format!("systemctl command failed: {}", output.status));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let services: Vec<ServiceInfo> = serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    Ok(services)
}

pub fn get_service_status(service_name: &str) -> Result<ServiceStatus, String> {
    let output = Command::new("systemctl")
        .args(&["show", service_name, "--property=ActiveState,SubState,MainPID", "--no-pager"])
        .output()
        .map_err(|e| format!("Failed to execute systemctl: {}", e))?;

    if !output.status.success() {
        return Err(format!("systemctl command failed: {}", output.status));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut active = false;
    let mut running = false;
    let mut pid = None;

    for line in stdout.lines() {
        if line.starts_with("ActiveState=") {
            active = line.trim_start_matches("ActiveState=") == "active";
        } else if line.starts_with("SubState=") {
            running = line.trim_start_matches("SubState=") == "running";
        } else if line.starts_with("MainPID=") {
            let pid_str = line.trim_start_matches("MainPID=");
            if let Ok(p) = pid_str.parse::<u32>() {
                pid = Some(p);
            }
        }
    }

    Ok(ServiceStatus {
        name: service_name.to_string(),
        active,
        running,
        pid,
    })
}

pub fn start_service(service_name: &str) -> Result<(), String> {
    let output = Command::new("systemctl")
        .args(&["start", service_name])
        .output()
        .map_err(|e| format!("Failed to execute systemctl: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to start service: {}", stderr));
    }

    Ok(())
}

pub fn stop_service(service_name: &str) -> Result<(), String> {
    let output = Command::new("systemctl")
        .args(&["stop", service_name])
        .output()
        .map_err(|e| format!("Failed to execute systemctl: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to stop service: {}", stderr));
    }

    Ok(())
}

pub fn restart_service(service_name: &str) -> Result<(), String> {
    let output = Command::new("systemctl")
        .args(&["restart", service_name])
        .output()
        .map_err(|e| format!("Failed to execute systemctl: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to restart service: {}", stderr));
    }

    Ok(())
}

pub fn reload_service(service_name: &str) -> Result<(), String> {
    let output = Command::new("systemctl")
        .args(&["reload", service_name])
        .output()
        .map_err(|e| format!("Failed to execute systemctl: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to reload service: {}", stderr));
    }

    Ok(())
}
