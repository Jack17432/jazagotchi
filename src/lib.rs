use esp_idf_svc::hal::gpio::Level;

pub mod apa102;
pub mod device;
pub mod rotary_encoder;
pub mod tft;

pub fn level_into_u8(level: Level) -> u8 {
    if level == Level::High {
        return 1u8;
    }

    0u8
}
