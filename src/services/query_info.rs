use crate::atservice::ATService;
use crate::models::*;
pub struct QueryInfo {
}

impl QueryInfo {
    pub fn new() -> Self {
        QueryInfo {}
    }

    pub async fn get_device_info(&self, at_service: &mut ATService) -> Result<DeviceInfo, Box<dyn std::error::Error>> {
        let resp = at_service.send_command("ATI")?;

        let mut manufacturer = String::new();
        let mut model = String::new();
        let mut version = String::new();
        let mut imei = String::new();
        let mut capabilities = String::new();

        for line in resp.lines() {
            if line.starts_with("Manufacturer: ") {
                manufacturer = line.strip_prefix("Manufacturer: ").unwrap().to_string();
            } else if line.starts_with("Model: ") {
                model = line.strip_prefix("Model: ").unwrap().to_string();
            } else if line.starts_with("Revision: ") {
                version = line.strip_prefix("Revision: ").unwrap().to_string();
            } else if line.starts_with("IMEI: ") {
                imei = line.strip_prefix("IMEI: ").unwrap().to_string();
            } else if line.starts_with("+GCAP: ") {
                capabilities = line.strip_prefix("+GCAP: ").unwrap().to_string();
            }
        }

        Ok(DeviceInfo {
            manufacturer,
            model,
            version,
            imei,
            capabilities,
        })
    }

    pub async fn get_connstat(&mut self, at_service: &mut ATService) -> Result<ConnectionStatus, Box<dyn std::error::Error>> {
        let resp = at_service.send_command("AT^NDISSTATQRY=8")?;
        
        let mut ipv4_stat = String::new(); // 0: disconnected, 1: connected, 2: connecting, 3: disconnecting
        let mut ipv4_err = String::new(); // 0: unknown_err
        let mut ipv6_stat = String::new();
        let mut ipv6_err = String::new();

        for line in resp.lines() {
            if line.starts_with("^NDISSTATQRY:") {
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() >= 8 {
                    ipv4_stat = parts[0].trim_start_matches("^NDISSTATQRY:").trim().to_string();
                    ipv4_err = parts[1].trim().to_string();
                    ipv6_stat = parts[4].trim().to_string();
                    ipv6_err = parts[5].trim().to_string();
                }
            }
        }

        Ok(ConnectionStatus {
            ipv4_stat,
            ipv4_err,
            ipv6_stat,
            ipv6_err,
        })
    }

    pub async fn get_config_info(&self, at_service: &mut ATService) -> Result<ConfigInfo, Box<dyn std::error::Error>> {
        let pcie_cfg = at_service.send_command("AT^TDPCIELANCFG?")?;
        let pm_cfg = at_service.send_command("AT^TDPMCFG?")?;
        let autodial_cfg = at_service.send_command("AT^SETAUTODIAL?")?;

        let mut pcie_lan_mode = 1;
        let mut pcie_pm_enabled = false;
        let mut auto_dial_enabled = false;
        let mut dial_mode = 0;

        /*
        pcie_lan_mode:
            1: supports 1Gbps
            2: supports 2Gbps
         */
        for line in pcie_cfg.lines() {
            if line.starts_with("^TDPCIELANCFG:") {
                if let Some(mode) = line.trim_start_matches("^TDPCIELANCFG:").trim().parse::<i32>().ok() {
                    pcie_lan_mode = mode;
                }
            }
        }

        for line in pm_cfg.lines() {
            if line.starts_with("^TDPMCFG:") {
                let parts: Vec<&str> = line.trim_start_matches("^TDPMCFG:").split(',').collect();
                if !parts.is_empty() {
                    pcie_pm_enabled = parts[0].trim() == "1";
                }
            }
        }

        /*
        dial_mode:
            0: modem
            1: host (USB)
            2: host (phy)
        */
        for line in autodial_cfg.lines() {
            if line.starts_with("^SETAUTODAIL:") {
                let parts: Vec<&str> = line.trim_start_matches("^SETAUTODAIL:").split(',').collect();
                if parts.len() >= 2 {
                    auto_dial_enabled = parts[0].trim() == "1";
                    if let Ok(mode) = parts[1].trim().parse::<i32>() {
                        dial_mode = mode;
                    }
                }
            }
        }

        Ok(ConfigInfo {
            pcie_lan_mode,
            pcie_pm_enabled,
            auto_dial_enabled,
            dial_mode,
        })
    }
}
