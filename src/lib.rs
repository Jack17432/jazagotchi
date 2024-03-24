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

pub fn level_to_bool(level: Level) -> bool {
    level == Level::High
}

pub trait EventSet {
    fn is_none(&self) -> bool;
}

pub trait Events<ES>
where
    ES: PartialEq,
{
    fn set(event: ES) -> Result<(), ()>;
    fn is_set(&self, event: ES) -> bool;

    fn wait_for_any(&self) -> ES;
    fn wait_for_set(&self, events: ES) -> ES;
    fn wait_for_all(&self) -> ES;
}
