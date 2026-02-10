use std::process::Command;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    let json: Value = serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    let rows = match json {
        Value::Array(rows) => rows,
        Value::Object(mut obj) => match obj.remove("units") {
            Some(Value::Array(rows)) => rows,
            _ => return Err("Unexpected JSON format from systemctl".to_string()),
        },
        _ => return Err("Unexpected JSON format from systemctl".to_string()),
    };

    let mut services = Vec::with_capacity(rows.len());

    for row in rows {
        let name = extract_string(
            &row,
            &["name", "unit", "Unit", "id", "Id", "names", "Names"],
        );
        if name.is_empty() {
            continue;
        }

        services.push(ServiceInfo {
            name,
            description: extract_string(&row, &["description", "Description"]),
            load_state: extract_string(&row, &["load_state", "load", "LoadState", "Load"]),
            active_state: extract_string(&row, &["active_state", "active", "ActiveState", "Active"]),
            sub_state: extract_string(&row, &["sub_state", "sub", "SubState", "Sub"]),
            unit_file_state: extract_string(
                &row,
                &["unit_file_state", "unit_file", "UnitFileState", "UnitFile"],
            ),
            followed_by: extract_string_vec(
                &row,
                &["followed_by", "followed", "following", "FollowedBy", "Following"],
            ),
        });
    }

    Ok(services)
}

fn extract_string(row: &Value, keys: &[&str]) -> String {
    for key in keys {
        if let Some(value) = row.get(*key) {
            match value {
                Value::String(s) => {
                    if !s.is_empty() {
                        return s.clone();
                    }
                }
                Value::Number(n) => return n.to_string(),
                Value::Bool(b) => return b.to_string(),
                _ => {}
            }
        }
    }

    String::new()
}

fn extract_string_vec(row: &Value, keys: &[&str]) -> Vec<String> {
    for key in keys {
        if let Some(value) = row.get(*key) {
            match value {
                Value::Array(values) => {
                    let out: Vec<String> = values
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();
                    if !out.is_empty() {
                        return out;
                    }
                }
                Value::String(s) => {
                    if !s.is_empty() {
                        return vec![s.clone()];
                    }
                }
                _ => {}
            }
        }
    }

    Vec::new()
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
