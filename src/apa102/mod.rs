use esp_idf_svc::hal::gpio::{Level, Output, Pin, PinDriver};
use esp_idf_svc::sys::EspError;

#[derive(Clone)]
pub struct Brightness(u8);

pub struct APA102<'d, 'l, P1: Pin, P2: Pin> {
    pin_clk: PinDriver<'d, P1, Output>,
    pin_do: PinDriver<'l, P2, Output>,
    led_states: Vec<LEDState>,
}

#[derive(Clone)]
pub struct LEDState {
    pub brightness: Brightness,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Brightness {
    pub const MAX: Self = Self(0b11111);
    pub const MIN: Self = Self(0b0);
    pub const OFF: Self = Self(0b0);

    pub fn new(val: u8) -> Result<Self, String> {
        if val > 0b11111 {
            return Err("Value brightness greater the 31".into());
        }

        Ok(Self(val))
    }

    pub fn value(&self) -> u8 {
        self.0
    }
}

impl<'d, 'l, P1: Pin, P2: Pin> APA102<'d, 'l, P1, P2> {
    pub fn new(
        num_led: u32,
        pin_clk: PinDriver<'d, P1, Output>,
        pin_do: PinDriver<'l, P2, Output>,
    ) -> Self {
        let led = LEDState {
            brightness: Brightness::OFF,
            red: 0,
            green: 0,
            blue: 0,
        };

        Self {
            pin_clk,
            pin_do,
            led_states: vec![led; num_led as usize],
        }
    }

    pub fn set_led(&mut self, led: LEDState, position: u32) -> Result<(), EspError> {
        self.led_states[position as usize] = led;

        self.send_led_states()
    }

    pub fn set_led_array(&mut self, leds: Vec<LEDState>) -> Result<(), EspError> {
        self.led_states = leds;

        self.send_led_states()
    }

    fn send_led_states(&mut self) -> Result<(), EspError> {
        self.send_start_of_frame()?;
        for idx in 0..self.led_states.len() {
            let led = self.led_states[idx].clone();
            self.send_led_frame(&led)?;
        }
        self.send_end_of_frame()?;

        Ok(())
    }

    fn send_led_frame(&mut self, led: &LEDState) -> Result<(), EspError> {
        self.send_byte(0b11100000 | led.brightness.value())?;
        self.send_byte(led.blue)?;
        self.send_byte(led.green)?;
        self.send_byte(led.red)?;

        Ok(())
    }

    fn send_start_of_frame(&mut self) -> Result<(), EspError> {
        // Needs to send 32 bits set to 0 to start a frame
        for _ in 0..4 {
            self.send_byte(0u8)?;
        }

        Ok(())
    }

    fn send_end_of_frame(&mut self) -> Result<(), EspError> {
        /* As we have learned above, the only function of the “End frame” is to supply
           strings up to 64 LEDs. This was first pointed out by Bernd in a comment. It
           should not matter, whether the end frame consists of ones or zeroes. Just
           don’t mix them.
        */

        for _ in 0..(self.led_states.len() / 16) {
            self.send_byte(1u8)?;
        }

        Ok(())
    }

    fn send_byte(&mut self, data: u8) -> Result<(), EspError> {
        for bit in (0..8).map(|i| data & (1u8 << i) != 0) {
            let level = Level::from(bit);

            self.pin_do.set_level(level)?;
            self.pin_clk.toggle()?;
            self.pin_clk.toggle()?;
        }

        Ok(())
    }
}
