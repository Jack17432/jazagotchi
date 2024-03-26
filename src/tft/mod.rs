use std::marker::PhantomData;

#[allow(dead_code)]
pub mod commands {
    pub type CmdID = u8;

    pub struct TFTCmd {
        cmd: CmdID,
        data: [u8; 14],
        len: u8,
    }

    /// List of all commands for the ST7789
    /// (<https://www.waveshare.com/w/upload/a/ae/ST7789_Datasheet.pdf> Page 156 - 161)
    #[allow(non_snake_case)]
    pub mod ST7789 {
        use crate::tft::commands::CmdID;

        /// No operation
        pub const NOP: CmdID = 0x00;
        /// Software reset
        pub const SWRESET: CmdID = 0x01;
        /// Read display ID
        pub const RDDID: CmdID = 0x04;
        /// Read display status
        pub const RDDST: CmdID = 0x09;
        /// Read display power
        pub const RDDPM: CmdID = 0x0A;
        /// Read display
        pub const RDDMADCTL: CmdID = 0x0B;
        /// Read display pixel
        pub const RDDCOLMOD: CmdID = 0x0C;
        /// Read display image
        pub const RDDIM: CmdID = 0x0D;
        /// Read display signal
        pub const RDDSM: CmdID = 0x0E;
        /// Read display self-diagnostic result
        pub const RDDSDR: CmdID = 0x0F;
        /// Sleep in
        pub const SLPIN: CmdID = 0x10;
        /// Sleep out
        pub const SLPOUT: CmdID = 0x11;
        /// Partial mode on
        pub const PTLON: CmdID = 0x12;
        /// Partial off (Normal)
        pub const NORON: CmdID = 0x13;
        /// Display inversion off
        pub const INVOFF: CmdID = 0x20;
        /// Display inversion on
        pub const INVON: CmdID = 0x21;
        /// Gamma set
        pub const GAMSET: CmdID = 0x26;
        /// Display off
        pub const DISPOFF: CmdID = 0x28;
        /// Display on
        pub const DISPON: CmdID = 0x29;
        /// Column address set
        pub const CASET: CmdID = 0x2A;
        /// Row address set
        pub const RASET: CmdID = 0x2B;
        /// Memory write
        pub const RAMWR: CmdID = 0x2C;
        /// Memory read
        pub const RAMRD: CmdID = 0x2E;
        /// Partial start/end address set
        pub const PTLAR: CmdID = 0x30;
        /// Vertical scrolling definition
        pub const VSCRDEF: CmdID = 0x33;
        /// Tearing effect line off
        pub const TEOFF: CmdID = 0x34;
        /// Tearing effect line on
        pub const TEON: CmdID = 0x35;
        /// Memory data access control
        pub const MADCTL: CmdID = 0x36;
        /// Vertical scrolling start address
        pub const VSCRSADD: CmdID = 0x37;
        /// Idle mode off
        pub const IDMOFF: CmdID = 0x38;
        /// Idle mode on
        pub const IDMON: CmdID = 0x39;
        /// Interface pixel format
        pub const COLMOD: CmdID = 0x3A;
        /// Memory write continue
        pub const RAMWRC: CmdID = 0x3C;
        /// Memory read continue
        pub const RAMRDC: CmdID = 0x3E;
        /// Set tear scanline
        pub const TESCAN: CmdID = 0x44;
        /// Get scanline
        pub const RDTESCAN: CmdID = 0x45;
        /// Write display brightness
        pub const WRDISBV: CmdID = 0x51;
        /// Read display brightness value
        pub const RDDISBV: CmdID = 0x52;
        /// Write CTRL display
        pub const WRCTRLD: CmdID = 0x53;
        /// Read CTRL value display
        pub const RDCTRLD: CmdID = 0x54;
        /// Write content adaptive brightness control and Color enhancement
        pub const WRCACE: CmdID = 0x55;
        /// Read content adaptive brightness control
        pub const RDCABC: CmdID = 0x56;
        /// Write CABC minimum brightness
        pub const WRCABCMB: CmdID = 0x5E;
        /// Read CABC minimum brightness
        pub const RDCABCMB: CmdID = 0x5F;
        /// Read Automatic Brightness Control Self-Diagnostic Result
        pub const RDABCSDR: CmdID = 0x68;
        /// Read ID1
        pub const RDID1: CmdID = 0xDA;
        /// Read ID2
        pub const RDID2: CmdID = 0xDB;
        /// Read ID3
        pub const RDID3: CmdID = 0xDC;
    }
}

#[allow(dead_code)]
struct TFT<S> {
    _state: PhantomData<S>,
}
