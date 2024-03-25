use esp_idf_svc::hal::gpio::InterruptType;
#[allow(dead_code)]
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::gpio::{InputPin, Pull};
use esp_idf_svc::hal::{delay::FreeRtos, peripherals::Peripherals};
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::sys;
use jazagotchi::apa102::{Brightness, LEDState, APA102};
use jazagotchi::device::{DevicePowerState, PowerToggle};
use jazagotchi::rotary_encoder;
use jazagotchi::rotary_encoder::interface::ROTARY_ENCODER;
use std::sync::atomic::{AtomicBool, Ordering};

static FLAG: AtomicBool = AtomicBool::new(false);

fn main() -> anyhow::Result<()> {
    sys::link_patches();
    EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    {
        let pin_a = PinDriver::input(peripherals.pins.gpio2.downgrade_input()).unwrap();
        let pin_b = PinDriver::input(peripherals.pins.gpio1.downgrade_input()).unwrap();

        rotary_encoder::interface::init_rotary_encoder(pin_a, pin_b);
    }

    let pwr_pin = PinDriver::output(peripherals.pins.gpio46).unwrap();
    let mut power_controller = DevicePowerState::new(pwr_pin).unwrap();
    power_controller.wake().unwrap();

    let spi_clk = PinDriver::output(peripherals.pins.gpio45).unwrap();
    let spi_do = PinDriver::output(peripherals.pins.gpio42).unwrap();
    let mut leds = APA102::new(7, spi_clk, spi_do);

    let mut button = PinDriver::input(peripherals.pins.gpio0).unwrap();
    button.set_pull(Pull::Up).unwrap();

    button.set_interrupt_type(InterruptType::PosEdge).unwrap();
    unsafe {
        button.subscribe(gpio_int_callback).unwrap();
    }
    button.enable_interrupt().unwrap();

    loop {
        // log::info!("Hello From Main");
        _ = button.enable_interrupt();

        match ROTARY_ENCODER.read() {
            Ok(data) => _ = leds.set_led_array(turn_into_leds(-data.get_position())),
            Err(err) => log::error!("Failed to gain read lock for endcoder data, {}", err),
        };

        FreeRtos::delay_ms(10);
    }
}

fn turn_into_leds(val: i8) -> Vec<LEDState> {
    let mut vec: Vec<LEDState> = vec![];
    let state = FLAG.load(Ordering::Relaxed);

    for _ in 0..val {
        vec.push(LEDState {
            brightness: Brightness::MAX,
            blue: if state { 100 } else { 0 },
            red: if state { 0 } else { 50 },
            green: if state { 0 } else { 50 },
        });
    }
    for _ in val..8 {
        vec.push(LEDState {
            brightness: Brightness::MAX,
            blue: 0,
            red: if state { 100 } else { 0 },
            green: if state { 0 } else { 100 },
        });
    }

    vec
}

fn gpio_int_callback() {
    // Assert FLAG indicating a press button happened
    let tmp = FLAG.load(Ordering::Relaxed);
    FLAG.store(!tmp, Ordering::Relaxed);
}
