//! # RA8875
//! A driver for the RA8875 display chip. Adapted from Adafruit's open-source
//! driver for their RA8875 line of TFT displays.
#![allow(dead_code)]

#[macro_use]
extern crate nb;
extern crate embedded_hal as hal;

use hal::spi::FullDuplex;
use hal::digital::InputPin;

type SpiError<SPI> = <SPI as FullDuplex<u8>>::Error;

#[derive(Copy,Clone,Debug)]
enum DisplayType {
    small_480x272,
    large_800x480
}

#[derive(Copy,Clone)]
enum Color {
    Black   =   0x0000,
    Blue    =   0x001F,
    Red     =   0xF800,
    Green   =   0x07E0,
    Cyan    =   0x07FF,
    Magenta =   0xF81F,
    Yellow  =   0xFFE0,
    White   =   0xFFFF
}

#[derive(Copy,Clone)]
enum Command {
    DataWrite =      0x00,
    DataRead  =      0x40,
    CmdWrite  =      0x80,
    CmdRead   =      0xC0
}

#[derive(Copy,Clone)]
enum Register {
    Pwrr    = 0x01,
    Mrwc    = 0x02,
    PllC1   = 0x88,
    PllC2   = 0x89,
    Sysr    = 0x10,
    Pcsr    = 0x04,
    Hdwr    = 0x14,
    Hndftr  = 0x15,
    Hndr    = 0x16,
    Hstr    = 0x17,
    Hpwr    = 0x18,
    Vdhr0   = 0x19,
    Vdhr1   = 0x1A,
    Vndr0   = 0x1B,
    Vndr1   = 0x1C,
    Vstr0   = 0x1D,
    Vstr1   = 0x1E,
    Vpwr    = 0x1F,
    Hsaw0   = 0x30,
    Hsaw1   = 0x31,
    Vsaw0   = 0x32,
    Vsaw1   = 0x33,
    Heaw0   = 0x34,
    Heaw1   = 0x35,
    Veaw0   = 0x36,
    Veaw1   = 0x37,
    Mlcr    = 0x8E,
    Dcr     = 0x90,
    Mwcr0   = 0x40,
    CurH0   = 0x46,
    CurH1   = 0x47,
    CurV0   = 0x48,
    CurV1   = 0x49,
    P1cr    = 0x8A,
    P1dcr   = 0x8B,
    P2cr    = 0x8C,
    P2dcr   = 0x8D,
    Tpcr0   = 0x70,
    Tpcr1   = 0x71,
    Tpxh    = 0x72,
    Tpyh    = 0x73,
    Tpxyl   = 0x74,
    Intc1   = 0xF0,
    Intc2   = 0xF1,
    Becr0   = 0x50,
    Becr1   = 0x51,
    Hsbe0   = 0x54,
    Hsbe1   = 0x55,
    Vsbe0   = 0x56,
    Vsbe1   = 0x57,
    Hdbe0   = 0x58,
    Hdbe1   = 0x59,
    Vdbe0   = 0x5A,
    Vdbe1   = 0x5B,
    Bewr0   = 0x5C,
    Bewr1   = 0x5D,
    Behr0   = 0x5E,
    Behr1   = 0x5F
}

mod cmds {
    pub enum Pwrr {
        DispOn = 0x80,
        // DispOff = 0x00,
        Sleep   = 0x02,
        Normal  = 0x00,
        SoftReset   = 0x01
    }
    pub enum PllC1 {
        Div2 = 0x80,
        Div1 = 0x00
    }
    pub enum PllC2 {
        Div1 = 0x00,
        Div2 = 0x01,
        Div4 = 0x02,
        Div8 = 0x03,
        Div16 = 0x04,
        Div32 = 0x05,
        Div64 = 0x06,
        Div128 = 0x07,
    }
    pub enum Sysr {
        BBP_8 = 0x00,
        BBP_16 = 0x0C,
        MCU_16 = 0x03,
        // MCU_8  = 0x00,
    }
    pub enum Pcsr {
        Pdatr     = 0x00,
        Pdatl     = 0x80,
        Clk_2     = 0x01,
        Clk_4     = 0x02,
        Clk_8     = 0x03,
    }
    pub enum Hndftr {
        High = 0x00,
        Low = 0x80
    }
    pub enum Hpwr {
        High = 0x80,
        Low = 0x00
    }
    pub enum Vpwr {
        High = 0x80,
        Low = 0x00
    }
    pub enum Mclr {
        Start = 0x80,
        Stop = 0x00,
        // TODO: Come back to the use cases of these cmds here
        // ReadStatus = 0x80,
        // Full = 0x00,
        Active  = 0x40
    }
    pub enum Dcr {
        LINESQUTRI_START  = 0x80,
        // LINESQUTRI_STOP   = 0x00,
        // LINESQUTRI_STATUS = 0x80,
        CIRCLE_START      = 0x40,
        // CIRCLE_STOP       = 0x00,
        // CIRCLE_STATUS     = 0x40,
        FILL              = 0x20,
        // NOFILL            = 0x00,
        DRAWLINE          = 0x00,
        DRAWTRIANGLE      = 0x01,
        DRAWSQUARE        = 0x10,
    }
    pub enum Mwcr0 {
        GfxMode = 0x00,
        TxtMode = 0x80
    }
    pub enum P1cr {
        Enable = 0x80,
        // Disable = 0x00,
        ClkOut  = 0x10,
        PwmOut = 0x00
    }
    pub enum P2cr {
        Enable = 0x80,
        // Disable = 0x00,
        ClkOut  = 0x10,
        PwmOut = 0x00
    }
    pub enum PwmClk {
        Div1 = 0x00,
        Div2 = 0x01,
        Div4 = 0x02,
        Div8 = 0x03,
        Div16 = 0x04,
        Div32 = 0x05,
        Div64 = 0x06,
        Div128 = 0x07,
        Div256 = 0x08,
        Div512 = 0x09,
        Div1024 = 0x0A,
        Div2048 = 0x0B,
        Div4096 = 0x0C,
        Div8192 = 0x0D,
        Div16384 = 0x0E,
        Div32768 = 0x0F,
    }
    pub enum Tpcr0 {
        ENABLE          =  0x80,
        // DISABLE         =  0x00,
        WAIT_512CLK     =  0x00,
        WAIT_1024CLK    =  0x10,
        WAIT_2048CLK    =  0x20,
        WAIT_4096CLK    =  0x30,
        WAIT_8192CLK    =  0x40,
        WAIT_16384CLK   =  0x50,
        WAIT_32768CLK   =  0x60,
        WAIT_65536CLK   =  0x70,
        WAKEENABLE      =  0x08,
        // WAKEDISABLE     =  0x00,
        // ADCCLK_DIV1     =  0x00,
        ADCCLK_DIV2     =  0x01,
        ADCCLK_DIV4     =  0x02,
        ADCCLK_DIV8     =  0x03,
        ADCCLK_DIV16    =  0x04,
        ADCCLK_DIV32    =  0x05,
        ADCCLK_DIV64    =  0x06,
        ADCCLK_DIV128   =  0x07,
    }
    pub enum Tprc1 {
        AUTO       =  0x00,
        MANUAL     =  0x40,
        // VREFINT    =  0x00,
        VREFEXT    =  0x20,
        DEBOUNCE   =  0x04,
        // NODEBOUNCE =  0x00,
        // IDLE       =  0x00,
        WAIT       =  0x01,
        LATCHX     =  0x02,
        LATCHY     =  0x03,
    }
    pub enum Intc1 {
        KEY     =  0x10,
        DMA     =  0x08,
        TP      =  0x04,
        BTE     =  0x02
    }
    pub enum Intc2 {
        KEY     =  0x10,
        DMA     =  0x08,
        TP      =  0x04,
        BTE     =  0x02
    }
}


struct RA8875<SPI: FullDuplex<u8>> {
    spi: SPI,
    display_type: DisplayType,
    width: u16,
    height: u16,
    ready: InputPin
}

impl<SPI: FullDuplex<u8>> RA8875<SPI> {
    fn write_data(&mut self, data: u8) -> nb::Result<(), SpiError<SPI>> {
        if self.ready.is_low() {
            Err(nb::Error::WouldBlock)
        } else {
            self.spi.send(Command::DataWrite as u8)?;
            self.spi.send(data)?;
            Ok(())
        }
    }

    fn read_data(&mut self) -> nb::Result<u8, SpiError<SPI>> {
        if self.ready.is_low() {
            Err(nb::Error::WouldBlock)
        } else {
            self.spi.send(Command::DataRead as u8)?;
            let result = self.spi.read()?;
            Ok(result)
        }
    }

    fn write_command(&mut self, command: u8) -> nb::Result<(), SpiError<SPI>>{
        if self.ready.is_low() {
            Err(nb::Error::WouldBlock)
        } else {
            self.spi.send(Command::CmdWrite as u8)?;
            self.spi.send(command)?;
            Ok(())
        }
    }

    fn read_status(&mut self) -> nb::Result<u8, SpiError<SPI>> {
        if self.ready.is_low() {
            Err(nb::Error::WouldBlock)
        } else {
            self.spi.send(Command::CmdRead as u8)?;
            let result = self.spi.read()?;
            Ok(result)
        }
    }

    fn write_register(&mut self, register: Register, data: u8) -> Result<(), SpiError<SPI>> {
        block!(self.write_command(register as u8))?;
        block!(self.write_data(data))?;
        Ok(())
    }

    fn read_register(&mut self, register: Register) -> Result<u8, SpiError<SPI>> {
        block!(self.write_command(register as u8))?;
        block!(self.read_data())
    }

    fn init(&mut self) -> Result<(), SpiError<SPI>> {
        let width = self.width;
        let height = self.height;
        self.write_register(Register::PllC1, cmds::PllC1::Div1 as u8 + 10)?;
        self.write_register(Register::PllC2, cmds::PllC2::Div4 as u8)?;
        self.write_register(Register::Sysr, cmds::Sysr::BBP_16 as u8)?;
        let t = match self.display_type {
            DisplayType::small_480x272 => {
                Timing {
                    pixclk          : cmds::Pcsr::Pdatl as u8 | cmds::Pcsr::Clk_4 as u8,
                    hsync_nondisp   : 10,
                    hsync_start     : 8,
                    hsync_pw        : 48,
                    hsync_finetune  : 0,
                    vsync_nondisp   : 3,
                    vsync_start     : 8,
                    vsync_pw        : 10,

                }
            },
            DisplayType::large_800x480 => {
                Timing {
                    pixclk          : cmds::Pcsr::Pdatl as u8 | cmds::Pcsr::Clk_2 as u8,
                    hsync_nondisp   : 26,
                    hsync_start     : 32,
                    hsync_pw        : 96,
                    hsync_finetune  : 0,
                    vsync_nondisp   : 32,
                    vsync_start     : 23,
                    vsync_pw        : 2,
                }
            }
        };
        self.write_register(Register::Pcsr, t.pixclk)?;

        self.write_register(Register::Hdwr, ((width / 8) - 1) as u8)?;
        self.write_register(Register::Hndftr, cmds::Hndftr::High as u8 + t.hsync_finetune)?;
        self.write_register(Register::Hndr, (t.hsync_nondisp - t.hsync_finetune - 2)/8)?;
        self.write_register(Register::Hstr, t.hsync_start/8 - 1)?;
        self.write_register(Register::Hpwr, cmds::Hpwr::Low as u8 + t.hsync_pw/8 - 1)?;

        self.write_register(Register::Vdhr0, ((height - 1) & 0xFF) as u8)?;
        self.write_register(Register::Vdhr1, ((height - 1) >> 8) as u8)?;
        self.write_register(Register::Vndr0, (t.vsync_nondisp - 1) as u8)?;
        self.write_register(Register::Vndr1, (t.vsync_nondisp >> 8) as u8)?;
        self.write_register(Register::Vstr0, (t.vsync_start - 1) as u8)?;
        self.write_register(Register::Vstr1, (t.vsync_start >> 8) as u8)?;
        self.write_register(Register::Vpwr, cmds::Vpwr::Low as u8 + t.vsync_pw - 1)?;

        self.write_register(Register::Hsaw0, 0)?;
        self.write_register(Register::Hsaw1, 0)?;
        self.write_register(Register::Heaw0, ((width - 1) & 0xFF) as u8)?;
        self.write_register(Register::Heaw1, ((width - 1) >> 8) as u8)?;

        self.write_register(Register::Vsaw0, 0)?;
        self.write_register(Register::Vsaw1, 0)?;
        self.write_register(Register::Veaw0, ((height - 1) & 0xFF) as u8)?;
        self.write_register(Register::Veaw1, ((height - 1) >> 8) as u8)?;

        self.text_mode()?;

        Ok(())
    }

    /// Enables text mode
    ///
    /// This currently forces the user to select the internal ROM font.
    pub fn text_mode(&mut self) -> Result<(), SpiError<SPI>>{
        let tmp = self.read_register(Register::Mwcr0)?;
        block!(self.write_data(tmp | cmds::Mwcr0::TxtMode as u8))?;

        // Sets the internal ROM font.
        // TODO: Get the register names + values for this so it isn't so cryptic.
        block!(self.write_command(0x21))?;
        let tmp = block!(self.read_data())?;
        block!(self.write_data(tmp & ((1<<7) | (1<<5))))?;

        // Clear serial font ROM settings
        block!(self.write_command(0x2F))?;
        block!(self.write_data(0x00))?;

        Ok(())
    }
}

struct Timing {
    pixclk: u8,
    hsync_start: u8,
    hsync_pw: u8,
    hsync_finetune: u8,
    hsync_nondisp: u8,
    vsync_pw: u8,
    vsync_nondisp: u16,
    vsync_start: u16
}
