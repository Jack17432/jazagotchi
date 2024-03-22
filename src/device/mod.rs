use esp_idf_svc::hal::gpio::{Level, Output, Pin, PinDriver};
use esp_idf_svc::sys::EspError;

#[derive(Copy, Clone, PartialEq)]
pub enum State {
    Off,
    On,
}

pub trait PowerToggle {
    fn get_state(&self) -> State;
    fn wake(&mut self) -> Result<(), EspError>;
    fn sleep(&mut self) -> Result<(), EspError>;
    fn toggle(&mut self) -> Result<(), EspError> {
        if self.get_state() == State::On {
            self.sleep()?;
        } else {
            self.wake()?;
        }

        Ok(())
    }
}

pub struct DevicePowerState<'d, P1: Pin> {
    peripheral_power: State,
    peripheral_power_pin: PinDriver<'d, P1, Output>,
}

impl<'d, P1: Pin> DevicePowerState<'d, P1> {
    pub fn new(mut peripheral_power_pin: PinDriver<'d, P1, Output>) -> Result<Self, EspError> {
        peripheral_power_pin.set_level(Level::Low)?;

        Ok(Self {
            peripheral_power: State::Off,
            peripheral_power_pin,
        })
    }
}

impl<'d, P1: Pin> PowerToggle for DevicePowerState<'d, P1> {
    fn get_state(&self) -> State {
        self.peripheral_power
    }

    fn wake(&mut self) -> Result<(), EspError> {
        self.peripheral_power_pin.set_high()?;
        self.peripheral_power = State::On;

        Ok(())
    }

    fn sleep(&mut self) -> Result<(), EspError> {
        self.peripheral_power_pin.set_low()?;
        self.peripheral_power = State::Off;

        Ok(())
    }
}
