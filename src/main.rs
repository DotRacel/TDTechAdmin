mod atservice;
mod commands;
mod models;
mod services;

use crate::atservice::ATService;
use crate::services::query_info::QueryInfo;

const DEFAULT_PORT: &str = "COM4";
const DEFAULT_BAUD_RATE: u32 = 115200;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut at_service = ATService::new(DEFAULT_PORT, DEFAULT_BAUD_RATE)?;
    let device_service = QueryInfo::new();
    
    let info = device_service.get_device_info(&mut at_service).await?;
    if info.model != "MT5700M-CN" { 
        eprintln!("Error: Unsupported model: {}", info.model);
        std::process::exit(1);
    }
    
    println!("Successfully validated AT service.\n\
        Manufacturer: {}\n\
        Model: {}\n\
        Version: {}\n\
        IMEI: {}\n\
        Capabilities: {}", info.manufacturer, info.model, info.version, info.imei, info.capabilities);
    
    let cfg = device_service.get_config_info(&mut at_service).await?;
    println!("PHY Speed: {}\nPCIE Power Manager: {}\nAuto dial: {}", 
            match cfg.pcie_lan_mode {
                1 => "1Gbps",
                2 => "2.5Gbps",
                _ => "unknown"
            },
            match cfg.pcie_pm_enabled {
                true => "enabled",
                false => "disabled"
            },
            match cfg.auto_dial_enabled{
                true => format!("enabled, Dial Mode: {}", 
                        match cfg.dial_mode {
                            0 => "modem",
                            1 => "host (usb)",
                            2 => "host (phy)",
                            _ => "unknown"
                        }),
                false => "disabled".to_string()
            }
    );

    println!("\nEntering interactive mode. Type 'exit' to quit.");
    println!("Enter AT commands:");

    let mut input = String::new();
    loop {
        input.clear();
        if std::io::stdin().read_line(&mut input).is_err() {
            eprintln!("Error reading input");
            continue;
        }

        let command = input.trim();
        if command.eq_ignore_ascii_case("exit") {
            println!("Exiting...");
            break;
        }

        match at_service.send_command(&format!("{}", command)) {
            Ok(response) => println!("{}", response),
            Err(e) => eprintln!("{}", e),
        }
    }

    Ok(())
}
