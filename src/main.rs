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
    let at_service = ATService::new(DEFAULT_PORT, DEFAULT_BAUD_RATE)?;
    let mut device_service = QueryInfo::new(at_service);

    let device_info = device_service.get_config_info().await?;
    println!("{:?}", device_info);

    Ok(())
}
