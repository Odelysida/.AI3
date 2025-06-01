use serde::{Deserialize, Serialize};
use crate::esp_compat::devices::ESPDeviceType;

/// ESP mining configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ESPMiningConfig {
    pub device_type: ESPDeviceType,
    pub max_memory_kb: usize,
    pub clock_speed_mhz: u32,
    pub wifi_ssid: String,
    pub server_address: String,
    pub server_port: u16,
    pub mining_intensity: u8, // 1-10 scale
    pub power_save_mode: bool,
}

impl Default for ESPMiningConfig {
    fn default() -> Self {
        Self {
            device_type: ESPDeviceType::ESP32,
            max_memory_kb: 320,
            clock_speed_mhz: 240,
            wifi_ssid: "TribeChain_Mining".to_string(),
            server_address: "192.168.1.100".to_string(),
            server_port: 8333,
            mining_intensity: 5,
            power_save_mode: false,
        }
    }
} 