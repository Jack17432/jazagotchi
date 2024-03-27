use esp_idf_svc::hal::gpio::InputPin;
use esp_idf_svc::hal::gpio::OutputPin;
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::{delay::FreeRtos, peripherals::Peripherals};
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::sys;
use jazagotchi::apa102::interface::{led_init, LEDInterface};
use jazagotchi::apa102::{Brightness, LEDState};
use jazagotchi::button_interface::{button_init, ButtonInterface};
use jazagotchi::device::{DevicePowerState, PowerToggle};
use jazagotchi::rotary_encoder::interface::{rotary_encoder_init, rotary_interface};
use jazagotchi::tft::tft_init;
use std::cmp::{max, min};

fn main() -> anyhow::Result<()> {
    sys::link_patches();
    EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    let pwr_pin = PinDriver::output(peripherals.pins.gpio46).unwrap();
    let mut power_controller = DevicePowerState::new(pwr_pin).unwrap();
    power_controller.wake().unwrap();

    {
        let encoder_pin_a = PinDriver::input(peripherals.pins.gpio2.downgrade_input()).unwrap();
        let encoder_pin_b = PinDriver::input(peripherals.pins.gpio1.downgrade_input()).unwrap();
        rotary_encoder_init(encoder_pin_a, encoder_pin_b);
    }

    {
        let button = PinDriver::input(peripherals.pins.gpio0).unwrap();
        button_init(button);
    }

    {
        let led_clk = PinDriver::output(peripherals.pins.gpio45.downgrade_output()).unwrap();
        let led_do = PinDriver::output(peripherals.pins.gpio42.downgrade_output()).unwrap();
        led_init(led_clk, led_do);
    }

    {
        let lcd_bl = peripherals.pins.gpio15.downgrade_output();
        let lcd_dc = peripherals.pins.gpio13.downgrade_output();
        let lcd_cs = peripherals.pins.gpio10.downgrade_output();
        let lcd_clk = peripherals.pins.gpio12.downgrade_output();
        let lcd_sdo = peripherals.pins.gpio11.downgrade_output();
        let lcd_rst = peripherals.pins.gpio9.downgrade_output();

        tft_init(
            peripherals.spi2,
            lcd_clk,
            lcd_sdo,
            lcd_cs,
            lcd_bl,
            lcd_dc,
            lcd_rst,
        );
    }

    loop {
        led_circle_thingy();

        FreeRtos::delay_ms(10);
    }
}

fn led_circle_thingy() {
    let mut vec: Vec<LEDState> = vec![];

    let val = match rotary_interface::get_position() {
        Ok(data) => -data,
        Err(err) => {
            log::error!("{}", err);
            0
        }
    };

    let val = max(min(val, 7), 0);

    let state = ButtonInterface::get_toggle_state();

    for _ in 0..val {
        vec.push(LEDState {
            brightness: Brightness::MAX,
            blue: if state { 100 } else { 0 },
            red: if state { 0 } else { 50 },
            green: if state { 0 } else { 50 },
        });
    }
    for _ in val..7 {
        vec.push(LEDState {
            brightness: Brightness::MAX,
            blue: 0,
            red: if state { 100 } else { 0 },
            green: if state { 0 } else { 100 },
        });
    }

    match LEDInterface::set_led_vec(vec) {
        Ok(_) => {}
        Err(err) => log::error!("{}", err),
    };
}
