use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub manufacturer: String,
    pub model: String,
    pub version: String,
    pub imei: String,
    pub capabilities: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionStatus {
    pub ipv4_stat: String,
    pub ipv4_err: String,
    pub ipv6_stat: String,
    pub ipv6_err: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigInfo {
    pub pcie_lan_mode: i32,
    pub pcie_pm_enabled: bool,
    pub auto_dial_enabled: bool,
    pub dial_mode: i32,
}
