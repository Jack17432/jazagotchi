use byte_slice_cast::*;
use embedded_graphics::pixelcolor::raw::RawU16;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::*;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::{AnyIOPin, AnyOutputPin, Level, Output, PinDriver};
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::hal::prelude::FromValueType;
use esp_idf_svc::hal::spi::config::Config;
use esp_idf_svc::hal::spi::{SpiAnyPins, SpiDeviceDriver, SpiDriver, SpiDriverConfig};

const DISPLAY_OFFSET_X: u16 = 0;
const DISPLAY_OFFSET_Y: u16 = 35; // hardware bug?

#[repr(u8)]
#[derive(Copy, Clone)]
#[allow(dead_code)]
enum ST7789Instructions {
    /// No operation
    NOP = 0x00,
    /// Software reset
    SWRESET = 0x01,
    /// Read display ID
    RDDID = 0x04,
    /// Read display status
    RDDST = 0x09,
    /// Read display power
    RDDPM = 0x0A,
    /// Read display
    RDDMADCTL = 0x0B,
    /// Read display pixel
    RDDCOLMOD = 0x0C,
    /// Read display image
    RDDIM = 0x0D,
    /// Read display signal
    RDDSM = 0x0E,
    /// Read display self-diagnostic result
    RDDSDR = 0x0F,
    /// Sleep in
    SLPIN = 0x10,
    /// Sleep out
    SLPOUT = 0x11,
    /// Partial mode on
    PTLON = 0x12,
    /// Partial off (Normal)
    NORON = 0x13,
    /// Display inversion off
    INVOFF = 0x20,
    /// Display inversion on
    INVON = 0x21,
    /// Gamma set
    GAMSET = 0x26,
    /// Display off
    DISPOFF = 0x28,
    /// Display on
    DISPON = 0x29,
    /// Column address set
    CASET = 0x2A,
    /// Row address set
    RASET = 0x2B,
    /// Memory write
    RAMWR = 0x2C,
    /// Memory read
    RAMRD = 0x2E,
    /// Partial start/end address set
    PTLAR = 0x30,
    /// Vertical scrolling definition
    VSCRDEF = 0x33,
    /// Tearing effect line off
    TEOFF = 0x34,
    /// Tearing effect line on
    TEON = 0x35,
    /// Memory data access control
    MADCTL = 0x36,
    /// Vertical scrolling start address
    VSCRSADD = 0x37,
    /// Idle mode off
    IDMOFF = 0x38,
    /// Idle mode on
    IDMON = 0x39,
    /// Interface pixel format
    COLMOD = 0x3A,
    /// Memory write continue
    RAMWRC = 0x3C,
    /// Memory read continue
    RAMRDC = 0x3E,
    /// Set tear scanline
    TESCAN = 0x44,
    /// Get scanline
    RDTESCAN = 0x45,
    /// Write display brightness
    WRDISBV = 0x51,
    /// Read display brightness value
    RDDISBV = 0x52,
    /// Write CTRL display
    WRCTRLD = 0x53,
    /// Read CTRL value display
    RDCTRLD = 0x54,
    /// Write content adaptive brightness control and Color enhancement
    WRCACE = 0x55,
    /// Read content adaptive brightness control
    RDCABC = 0x56,
    /// Write CABC minimum brightness
    WRCABCMB = 0x5E,
    /// Read CABC minimum brightness
    RDCABCMB = 0x5F,
    /// Read Automatic Brightness Control Self-Diagnostic Result
    RDABCSDR = 0x68,
    /// Read ID1
    RDID1 = 0xDA,
    /// Read ID2
    RDID2 = 0xDB,
    /// Read ID3
    RDID3 = 0xDC,
}
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum Orientation {
    Portrait = 0b0000_0000,         // no inverting
    Landscape = 0b0110_0000,        // invert column and page/column order
    PortraitSwapped = 0b1100_0000,  // invert page and column order
    LandscapeSwapped = 0b1010_0000, // invert page and page/column order
}

#[allow(dead_code)]
pub struct ST7789 {
    display_interface: DisplaySpiInterface,
    rst: PinDriver<'static, AnyOutputPin, Output>, // Reset pin
    bl: PinDriver<'static, AnyOutputPin, Output>,  // Backlight

    size_x: u16,
    size_y: u16,
    orientation: Orientation,
}

impl ST7789 {
    fn init(
        display_interface: DisplaySpiInterface,
        rst: PinDriver<'static, AnyOutputPin, Output>,
        bl: PinDriver<'static, AnyOutputPin, Output>,
        size_x: u16,
        size_y: u16,
        orientation: Orientation,
    ) -> Self {
        let mut lcd = Self {
            display_interface,
            rst,
            bl,
            size_x,
            size_y,
            orientation,
        };

        lcd.startup_sequence();
        lcd.set_orientation(orientation);
        lcd.set_tearing_effect(TearingEffect::Vertical);
        lcd.clear(Rgb565::BLACK).unwrap();
        lcd
    }

    fn startup_sequence(&mut self) {
        self.hard_rst();

        self.set_backlight(Level::Low);
        self.set_backlight(Level::High);

        self.display_interface
            .send_command(ST7789Instructions::SWRESET); // reset display
        FreeRtos::delay_ms(150);
        self.display_interface
            .send_command(ST7789Instructions::SLPOUT); // turn off sleep
        FreeRtos::delay_ms(10);
        self.display_interface
            .send_command(ST7789Instructions::INVOFF); // turn off invert
        self.display_interface
            .send_command(ST7789Instructions::VSCRDEF); // vertical scroll definition
        self.display_interface
            .send_data_u8(&[0u8, 0u8, 0x14u8, 0u8, 0u8, 0u8]); // 0 TSA, 320 VSA, 0 BSA
        self.display_interface
            .send_command(ST7789Instructions::MADCTL); // left -> right, bottom -> top RGB
        self.display_interface.send_data_u8(&[0b00000]);
        self.display_interface
            .send_command(ST7789Instructions::COLMOD); // 16bit 65k colors
        self.display_interface.send_data_u8(&[0b0101_0101]);
        self.display_interface
            .send_command(ST7789Instructions::INVON); // hack?
        FreeRtos::delay_ms(10);
        self.display_interface
            .send_command(ST7789Instructions::NORON); // turn on display
        FreeRtos::delay_ms(10);
        self.display_interface
            .send_command(ST7789Instructions::DISPON); // turn on display
        FreeRtos::delay_ms(10);
    }

    fn set_orientation(&mut self, orientation: Orientation) {
        self.display_interface
            .send_command(ST7789Instructions::MADCTL);
        self.display_interface.send_data_u8(&[orientation as u8]);
        self.orientation = orientation;
    }

    fn set_backlight(&mut self, state: Level) {
        self.bl.set_level(state).unwrap();
    }

    fn hard_rst(&mut self) {
        self.rst.set_high().unwrap();
        FreeRtos::delay_ms(1);
        self.rst.set_low().unwrap();
        FreeRtos::delay_ms(1);
        self.rst.set_high().unwrap();
        FreeRtos::delay_ms(1);
    }

    pub fn set_pixels<T>(&mut self, start: (u16, u16), end: (u16, u16), colours: T)
    where
        T: IntoIterator<Item = u16>,
    {
        self.set_address_window(start.0, start.1, end.0, end.1);
        self.display_interface
            .send_command(ST7789Instructions::RAMWR);

        self.display_interface
            .send_data_u16iter(&mut colours.into_iter());
    }

    pub fn set_pixel(&mut self, position: (u16, u16), colour: u16) {
        self.set_address_window(position.0, position.1, position.0, position.1);
        self.display_interface
            .send_command(ST7789Instructions::RAMWR);

        self.display_interface
            .send_data_u8(&colour.to_le().to_be_bytes());
    }

    fn set_address_window(&mut self, start_x: u16, start_y: u16, end_x: u16, end_y: u16) {
        self.display_interface
            .send_command(ST7789Instructions::CASET);
        self.display_interface
            .send_data_u8(&(start_x + DISPLAY_OFFSET_X).to_be_bytes());
        self.display_interface
            .send_data_u8(&(end_x + DISPLAY_OFFSET_X).to_be_bytes());
        self.display_interface
            .send_command(ST7789Instructions::RASET);
        self.display_interface
            .send_data_u8(&(start_y + DISPLAY_OFFSET_Y).to_be_bytes());
        self.display_interface
            .send_data_u8(&(end_y + DISPLAY_OFFSET_Y).to_be_bytes());
    }

    pub fn set_tearing_effect(&mut self, tearing_effect: TearingEffect) {
        match tearing_effect {
            TearingEffect::Off => self.display_interface.send_command(ST7789Instructions::TEOFF),
            TearingEffect::Vertical => {
                self.display_interface.send_command(ST7789Instructions::TEON);
                self.display_interface.send_data_u8(&[0]);
            }
            TearingEffect::HorizontalAndVertical => {
                self.display_interface.send_command(ST7789Instructions::TEON);
                self.display_interface.send_data_u8(&[1]);
            }
        }
    }
}

#[derive(Copy, Clone)]
pub enum TearingEffect {
    /// Disable output.
    Off,
    /// Output vertical blanking information.
    Vertical,
    /// Output horizontal and vertical blanking information.
    HorizontalAndVertical,
}

struct DisplaySpiInterface {
    spi: SpiDeviceDriver<'static, SpiDriver<'static>>,
    dc: PinDriver<'static, AnyOutputPin, Output>,
}

impl DisplaySpiInterface {
    fn new(
        spi: SpiDeviceDriver<'static, SpiDriver<'static>>,
        dc: PinDriver<'static, AnyOutputPin, Output>,
    ) -> Self {
        Self { spi, dc }
    }

    fn send_command(&mut self, cmd: ST7789Instructions) {
        self.dc.set_low().unwrap();
        self.spi.write(&[cmd as u8]).unwrap();
    }

    fn send_data_u8(&mut self, data: &[u8]) {
        self.dc.set_high().unwrap();
        self.spi.write(data).unwrap();
    }

    fn send_data_u16iter<'a>(&mut self, iter: &'a mut dyn Iterator<Item = u16>) {
        self.dc.set_high().unwrap();

        let mut buf = [0; 64];
        let mut i = 0;
        let len = buf.len();

        for v in iter.map(u16::to_be) {
            buf[i] = v;
            i += 1;

            if i == len {
                self.spi.write(buf.as_byte_slice()).unwrap();
                i = 0;
            }
        }

        if i > 0 {
            self.spi.write(buf[..i].as_byte_slice()).unwrap();
        }
    }
}

impl Dimensions for ST7789 {
    fn bounding_box(&self) -> Rectangle {
        Rectangle::new(
            Point::new(0, 0),
            Size::new(self.size_x as u32, self.size_y as u32),
        )
    }
}

impl DrawTarget for ST7789 {
    type Color = Rgb565;
    type Error = ();

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for pixel in pixels {
            let colour = RawU16::from(pixel.1).into_inner();
            self.set_pixel((pixel.0.x as u16, pixel.0.y as u16), colour);
        }

        Ok(())
    }

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        if let Some(bottom_right) = area.bottom_right() {
            let mut count = 0u32;
            let max = area.size.width * area.size.height;

            let colours = colors
                .into_iter()
                .take_while(|_| {
                    count += 1;
                    count <= max
                })
                .map(|colour| RawU16::from(colour).into_inner());

            let start_x = area.top_left.x as u16;
            let start_y = area.top_left.y as u16;
            let end_x = bottom_right.x as u16;
            let end_y = bottom_right.y as u16;
            self.set_pixels((start_x, start_y), (end_x, end_y), &mut colours.into_iter());
        };

        Ok(())
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        let area = area.intersection(&self.bounding_box());

        if let Some(bottom_right) = area.bottom_right() {
            let mut count = 0u32;
            let max = area.size.width * area.size.height;

            let mut colors = core::iter::repeat(color.into_storage()).take_while(|_| {
                count += 1;
                count <= max
            });

            let start_x = area.top_left.x as u16;
            let start_y = area.top_left.y as u16;
            let end_x = bottom_right.x as u16;
            let end_y = bottom_right.y as u16;
            self.set_pixels((start_x, start_y), (end_x, end_y), &mut colors);
        };

        Ok(())
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        let mut count = 0u32;
        let max = self.size_x as u32 * self.size_y as u32;

        let mut colors = core::iter::repeat(color.into_storage()).take_while(|_| {
            count += 1;
            count <= max
        });

        let start_x = 0u16;
        let start_y = 0u16;
        let end_x = self.size_x;
        let end_y = self.size_y;
        self.set_pixels((start_x, start_y), (end_x, end_y), &mut colors);

        Ok(())
    }
}

fn tft_task<APP: App>(mut lcd: ST7789, mut app: Box<APP>) -> ! {
    loop {
        app.update(&mut lcd);
        FreeRtos::delay_ms(1);
    }
}

pub fn tft_init<SPI, APP>(
    spi: impl Peripheral<P = SPI> + 'static,
    clk: AnyOutputPin,
    sdo: AnyOutputPin,
    cs: AnyOutputPin,
    bl: AnyOutputPin,
    dc: AnyOutputPin,
    rst: AnyOutputPin,
    app: Box<APP>,
) where
    SPI: SpiAnyPins,
    APP: 'static + App + Send,
{
    let spi_drv = SpiDriver::new(spi, clk, sdo, None::<AnyIOPin>, &SpiDriverConfig::new()).unwrap();

    let config = Config::new().baudrate(27.MHz().into());

    let spi = SpiDeviceDriver::new(spi_drv, Some(cs), &config).unwrap();

    let dc = PinDriver::output(dc).unwrap();
    let display_interface = DisplaySpiInterface::new(spi, dc);

    let rst = PinDriver::output(rst).unwrap();
    let bl = PinDriver::output(bl).unwrap();
    let lcd = ST7789::init(display_interface, rst, bl, 320, 170, Orientation::Landscape);

    std::thread::Builder::new()
        .name("tft_task".into())
        .stack_size(32 * 300)
        .spawn(move || tft_task(lcd, app))
        .unwrap();
}

pub trait App {
    fn update(&mut self, display: &mut ST7789);
}
