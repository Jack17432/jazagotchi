use esp_idf_svc::hal::gpio::InterruptType;
#[allow(dead_code)]
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::gpio::Pull;
use esp_idf_svc::hal::{delay::FreeRtos, peripherals::Peripherals};
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::sys;
use jazagotchi::apa102::{Brightness, LEDState, APA102};
use jazagotchi::device::{DevicePowerState, PowerToggle};
use jazagotchi::rotary_encoder::{LatchMode, RotaryEncoder};
use std::sync::atomic::{AtomicBool, Ordering};

static FLAG: AtomicBool = AtomicBool::new(false);

fn main() -> anyhow::Result<()> {
    sys::link_patches();
    EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    let pwr_pin = PinDriver::output(peripherals.pins.gpio46).unwrap();
    let mut power_controller = DevicePowerState::new(pwr_pin).unwrap();
    power_controller.wake().unwrap();

    let spi_clk = PinDriver::output(peripherals.pins.gpio45).unwrap();
    let spi_do = PinDriver::output(peripherals.pins.gpio42).unwrap();
    let mut leds = APA102::new(7, spi_clk, spi_do);

    let pin_encoder_a = PinDriver::input(peripherals.pins.gpio2).unwrap();
    let pin_encoder_b = PinDriver::input(peripherals.pins.gpio1).unwrap();
    let mut encoder = RotaryEncoder::new(pin_encoder_a, pin_encoder_b, LatchMode::TWO3);

    let mut button = PinDriver::input(peripherals.pins.gpio0).unwrap();
    button.set_pull(Pull::Up).unwrap();

    button.set_interrupt_type(InterruptType::PosEdge).unwrap();
    unsafe {
        button.subscribe(gpio_int_callback).unwrap();
    }
    button.enable_interrupt().unwrap();

    // std::thread::Builder::new()
    //     .name("button ISR".into())
    //     .stack_size(5000)
    //     .spawn(move || {
    //         block_on(async {
    //             loop {
    //                 button.wait_for_rising_edge().await.unwrap();
    //                 let tmp_flag = FLAG.load(Ordering::Relaxed);
    //                 FLAG.store(!tmp_flag, Ordering::Relaxed);
    //             }
    //         })
    //     })
    //     .unwrap();

    loop {
        // log::info!("Hello From Main");
        encoder.update();
        let _ = button.enable_interrupt();

        let _ = leds.set_led_array(turn_into_leds(-encoder.get_position()));

        log::info!("{}", FLAG.load(Ordering::Relaxed));

        FreeRtos::delay_ms(100);
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
