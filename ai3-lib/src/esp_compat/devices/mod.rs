use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ESPDeviceType {
    ESP32,
    ESP8266,
    ESP32S2,
    ESP32S3,
    ESP32C3,
}

impl ESPDeviceType {
    pub fn get_memory_limit(&self) -> usize {
        match self {
            ESPDeviceType::ESP32 => 320,      // 320KB RAM
            ESPDeviceType::ESP8266 => 80,     // 80KB RAM
            ESPDeviceType::ESP32S2 => 320,    // 320KB RAM
            ESPDeviceType::ESP32S3 => 512,    // 512KB RAM
            ESPDeviceType::ESP32C3 => 400,    // 400KB RAM
        }
    }

    pub fn get_compute_power(&self) -> u64 {
        match self {
            ESPDeviceType::ESP32 => 240,      // 240MHz dual core
            ESPDeviceType::ESP8266 => 80,     // 80MHz single core
            ESPDeviceType::ESP32S2 => 240,    // 240MHz single core
            ESPDeviceType::ESP32S3 => 240,    // 240MHz dual core
            ESPDeviceType::ESP32C3 => 160,    // 160MHz single core
        }
    }

    pub fn supports_floating_point(&self) -> bool {
        match self {
            ESPDeviceType::ESP32 => true,
            ESPDeviceType::ESP8266 => false,  // Limited FPU
            ESPDeviceType::ESP32S2 => true,
            ESPDeviceType::ESP32S3 => true,
            ESPDeviceType::ESP32C3 => true,
        }
    }
} 