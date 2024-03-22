#[allow(dead_code)]
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::{delay::FreeRtos, peripherals::Peripherals};
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::sys;
use jazagotchi::apa102::{Brightness, LEDState, APA102};
use jazagotchi::device::{DevicePowerState, PowerToggle};

fn main() -> anyhow::Result<()> {
    sys::link_patches();
    EspLogger::initialize_default();

    // Example on multi threading
    // std::thread::Builder::new()
    //     .name("test1".into())
    //     .stack_size(2000)
    //     .spawn(move || {
    //         loop {
    //             log::info!("hello from the other thread!");
    //             FreeRtos::delay_ms(1000);
    //             assert!(false);
    //         }
    //     }).unwrap();

    let peripherals = Peripherals::take().unwrap();

    let pwr_pin = PinDriver::output(peripherals.pins.gpio46).unwrap();
    let mut power_controller = DevicePowerState::new(pwr_pin).unwrap();
    power_controller.wake().unwrap();

    let spi_clk = PinDriver::output(peripherals.pins.gpio45).unwrap();
    let spi_do = PinDriver::output(peripherals.pins.gpio42).unwrap();
    let mut leds = APA102::new(7, spi_clk, spi_do);

    loop {
        log::info!("Hello From Main");
        FreeRtos::delay_ms(2000);

        let led = LEDState {
            brightness: Brightness::MAX,
            red: 0,
            green: 0,
            blue: 255,
        };
        leds.set_led(led, 1).unwrap();
    }
}
