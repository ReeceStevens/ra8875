//! # RA8875
//! A driver for the RA8875 display chip. Adapted from Adafruit's open-source
//! driver for their RA8875 line of TFT displays.
#![allow(dead_code)]
#![no_std]

#[macro_use]
extern crate nb;
extern crate embedded_graphics;
extern crate embedded_hal as hal;

use core::fmt;
use core::fmt::Write;

use embedded_graphics::{
    prelude::*,
    drawable,
    pixelcolor::{IntoStorage, Rgb565},
    DrawTarget,
    style::{PrimitiveStyle, Styled},
    primitives,
};

use hal::digital::v2::InputPin;
use hal::spi::FullDuplex;

type SpiError<SPI> = <SPI as FullDuplex<u8>>::Error;

#[derive(Copy, Clone)]
enum Color {
    Black = 0x0000,
    Blue = 0x001F,
    Red = 0xF800,
    Green = 0x07E0,
    Cyan = 0x07FF,
    Magenta = 0xF81F,
    Yellow = 0xFFE0,
    White = 0xFFFF,
}

#[derive(Copy, Clone)]
enum Command {
    DataWrite = 0x00,
    DataRead = 0x40,
    CmdWrite = 0x80,
    CmdRead = 0xC0,
}

#[derive(Copy, Clone)]
#[allow(non_camel_case_types)]
enum Register {
    Pwrr = 0x01,
    Mrwc = 0x02,
    PllC1 = 0x88,
    PllC2 = 0x89,
    Sysr = 0x10,
    Pcsr = 0x04,
    Hdwr = 0x14,
    Hndftr = 0x15,
    Hndr = 0x16,
    Hstr = 0x17,
    Hpwr = 0x18,
    Vdhr0 = 0x19,
    Vdhr1 = 0x1A,
    Vndr0 = 0x1B,
    Vndr1 = 0x1C,
    Vstr0 = 0x1D,
    Vstr1 = 0x1E,
    Vpwr = 0x1F,
    Hsaw0 = 0x30,
    Hsaw1 = 0x31,
    Vsaw0 = 0x32,
    Vsaw1 = 0x33,
    Heaw0 = 0x34,
    Heaw1 = 0x35,
    Veaw0 = 0x36,
    Veaw1 = 0x37,
    Mclr = 0x8E,
    Dcr = 0x90,
    DrawEllipseCR = 0xa0,
    Mwcr0 = 0x40,
    CurH0 = 0x46,
    CurH1 = 0x47,
    CurV0 = 0x48,
    CurV1 = 0x49,
    P1cr = 0x8A,
    P1dcr = 0x8B,
    P2cr = 0x8C,
    P2dcr = 0x8D,
    Tpcr0 = 0x70,
    Tpcr1 = 0x71,
    Tpxh = 0x72,
    Tpyh = 0x73,
    Tpxyl = 0x74,
    Intc1 = 0xF0,
    Intc2 = 0xF1,
    Becr0 = 0x50,
    Becr1 = 0x51,
    Hsbe0 = 0x54,
    Hsbe1 = 0x55,
    Vsbe0 = 0x56,
    Vsbe1 = 0x57,
    Hdbe0 = 0x58,
    Hdbe1 = 0x59,
    Vdbe0 = 0x5A,
    Vdbe1 = 0x5B,
    Bewr0 = 0x5C,
    Bewr1 = 0x5D,
    Behr0 = 0x5E,
    Behr1 = 0x5F,
    TextX0 = 0x2A,
    TextX1 = 0x2B,
    TextY0 = 0x2C,
    TextY1 = 0x2D,
    TextBg0 = 0x60,
    TextBg1 = 0x61,
    TextBg2 = 0x62,
    Color0 = 0x63,
    Color1 = 0x64,
    Color2 = 0x65,
    FontOptions = 0x22,
    ShapeStartX0 = 0x91,
    ShapeStartX1 = 0x92,
    ShapeStartY0 = 0x93,
    ShapeStartY1 = 0x94,
    ShapeEndX0 = 0x95,
    ShapeEndX1 = 0x96,
    ShapeEndY0 = 0x97,
    ShapeEndY1 = 0x98,
    CircleX0 = 0x99,
    CircleX1 = 0x9a,
    CircleY0 = 0x9b,
    CircleY1 = 0x9c,
    CircleR = 0x9d,
    TriangleP2X0 = 0xa9,
    TriangleP2X1 = 0xaa,
    TriangleP2Y0 = 0xab,
    TriangleP2Y1 = 0xac,
    EllipseLongA0 = 0xa1,
    EllipseLongA1 = 0xa2,
    EllipseShortB0 = 0xa3,
    EllipseShortB1 = 0xa4,
    EllipseCenterX0 = 0xa5,
    EllipseCenterX1 = 0xa6,
    EllipseCenterY0 = 0xa7,
    EllipseCenterY1 = 0xa8,
}

#[allow(non_camel_case_types)]
mod cmds {
    pub enum Pwrr {
        DispOn = 0x80,
        // DispOff = 0x00,
        Sleep = 0x02,
        Normal = 0x00,
        SoftReset = 0x01,
    }
    pub enum PllC1 {
        Div2 = 0x80,
        Div1 = 0x00,
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
        Pdatr = 0x00,
        Pdatl = 0x80,
        Clk_2 = 0x01,
        Clk_4 = 0x02,
        Clk_8 = 0x03,
    }
    pub enum Hndftr {
        High = 0x00,
        Low = 0x80,
    }
    pub enum Hpwr {
        High = 0x80,
        Low = 0x00,
    }
    pub enum Vpwr {
        High = 0x80,
        Low = 0x00,
    }
    pub enum Mclr {
        Start = 0x80,
        Stop = 0x00,
        // TODO: Come back to the use cases of these cmds here
        // ReadStatus = 0x80,
        // Full = 0x00,
        Active = 0x40,
    }
    pub enum Dcr {
        LINESQUTRI_START = 0x80,
        // LINESQUTRI_STOP   = 0x00,
        // LINESQUTRI_STATUS = 0x80,
        CIRCLE_START = 0x40,
        // CIRCLE_STOP       = 0x00,
        // CIRCLE_STATUS     = 0x40,
        FILL = 0x20,
        // NOFILL            = 0x00,
        DRAWLINE = 0x00,
        DRAWTRIANGLE = 0x01,
        DRAWSQUARE = 0x10,
    }
    pub enum DrawEllipseCR {
        DRAWSTART = 0x80,
        FILL = 0x40,
        ELLIPSE_CIRCSQ_SEL = 0x20,
        ELLIPSE_CURVE_SEL = 0x10,
        EllipseCurvePart = 0x03,
    }
    pub enum Mwcr0 {
        GfxMode = 0x00,
        TxtMode = 0x80,
    }
    pub enum P1cr {
        Enable = 0x80,
        // Disable = 0x00,
        ClkOut = 0x10,
        PwmOut = 0x00,
    }
    pub enum P2cr {
        Enable = 0x80,
        // Disable = 0x00,
        ClkOut = 0x10,
        PwmOut = 0x00,
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
        ENABLE = 0x80,
        // DISABLE         =  0x00,
        WAIT_512CLK = 0x00,
        WAIT_1024CLK = 0x10,
        WAIT_2048CLK = 0x20,
        WAIT_4096CLK = 0x30,
        WAIT_8192CLK = 0x40,
        WAIT_16384CLK = 0x50,
        WAIT_32768CLK = 0x60,
        WAIT_65536CLK = 0x70,
        WAKEENABLE = 0x08,
        // WAKEDISABLE     =  0x00,
        // ADCCLK_DIV1     =  0x00,
        ADCCLK_DIV2 = 0x01,
        ADCCLK_DIV4 = 0x02,
        ADCCLK_DIV8 = 0x03,
        ADCCLK_DIV16 = 0x04,
        ADCCLK_DIV32 = 0x05,
        ADCCLK_DIV64 = 0x06,
        ADCCLK_DIV128 = 0x07,
    }
    pub enum Tprc1 {
        AUTO = 0x00,
        MANUAL = 0x40,
        // VREFINT    =  0x00,
        VREFEXT = 0x20,
        DEBOUNCE = 0x04,
        // NODEBOUNCE =  0x00,
        // IDLE       =  0x00,
        WAIT = 0x01,
        LATCHX = 0x02,
        LATCHY = 0x03,
    }
    pub enum Intc1 {
        KEY = 0x10,
        DMA = 0x08,
        TP = 0x04,
        BTE = 0x02,
    }
    pub enum Intc2 {
        KEY = 0x10,
        DMA = 0x08,
        TP = 0x04,
        BTE = 0x02,
    }
}

type Coord = (i16, i16);

struct TextModeSettings {
    cursor: Coord,
    fg_color: u16,
    bg_color: Option<u16>,
    text_scale: u8,
    transparency: bool,
}

struct GraphicsModeSettings {
    cursor: Coord,
    color: u16,
}

#[derive(Copy, Clone)]
enum Mode {
    Text,
    Graphics,
}

struct RA8875<SPI: FullDuplex<u8>, P: InputPin> {
    spi: SPI,
    dims: (u16, u16),
    text_settings: TextModeSettings,
    gfx_settings: GraphicsModeSettings,
    mode: Mode,
    ready: P,
}

impl<SPI: FullDuplex<u8>, P: InputPin> RA8875<SPI, P> {
    fn write_data(&mut self, data: u8) -> nb::Result<(), SpiError<SPI>> {
        if self.ready.is_low().ok().unwrap() {
            Err(nb::Error::WouldBlock)
        } else {
            self.spi.send(Command::DataWrite as u8)?;
            self.spi.send(data)?;
            Ok(())
        }
    }

    fn read_data(&mut self) -> nb::Result<u8, SpiError<SPI>> {
        if self.ready.is_low().ok().unwrap() {
            Err(nb::Error::WouldBlock)
        } else {
            self.spi.send(Command::DataRead as u8)?;
            let result = self.spi.read()?;
            Ok(result)
        }
    }

    fn write_command(&mut self, command: u8) -> nb::Result<(), SpiError<SPI>> {
        if self.ready.is_low().ok().unwrap() {
            Err(nb::Error::WouldBlock)
        } else {
            self.spi.send(Command::CmdWrite as u8)?;
            self.spi.send(command)?;
            Ok(())
        }
    }

    fn read_status(&mut self) -> nb::Result<u8, SpiError<SPI>> {
        if self.ready.is_low().ok().unwrap() {
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
        let (width, height) = self.dims;
        self.write_register(Register::PllC1, cmds::PllC1::Div1 as u8 + 10)?;
        self.write_register(Register::PllC2, cmds::PllC2::Div4 as u8)?;
        self.write_register(Register::Sysr, cmds::Sysr::BBP_16 as u8)?;
        let t = match self.dims {
            (480, 272) => Timing {
                pixclk: cmds::Pcsr::Pdatl as u8 | cmds::Pcsr::Clk_4 as u8,
                hsync_nondisp: 10,
                hsync_start: 8,
                hsync_pw: 48,
                hsync_finetune: 0,
                vsync_nondisp: 3,
                vsync_start: 8,
                vsync_pw: 10,
            },
            (800, 480) => Timing {
                pixclk: cmds::Pcsr::Pdatl as u8 | cmds::Pcsr::Clk_2 as u8,
                hsync_nondisp: 26,
                hsync_start: 32,
                hsync_pw: 96,
                hsync_finetune: 0,
                vsync_nondisp: 32,
                vsync_start: 23,
                vsync_pw: 2,
            },
            _ => {
                panic!("Unsupported display dimensions.");
            }
        };
        self.write_register(Register::Pcsr, t.pixclk)?;

        self.write_register(Register::Hdwr, ((width / 8) - 1) as u8)?;
        self.write_register(
            Register::Hndftr,
            cmds::Hndftr::High as u8 + t.hsync_finetune,
        )?;
        self.write_register(Register::Hndr, (t.hsync_nondisp - t.hsync_finetune - 2) / 8)?;
        self.write_register(Register::Hstr, t.hsync_start / 8 - 1)?;
        self.write_register(Register::Hpwr, cmds::Hpwr::Low as u8 + t.hsync_pw / 8 - 1)?;

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

        // Clear screen
        self.write_register(Register::Mclr, cmds::Mclr::Start as u8)?;

        self.text_mode()?;

        Ok(())
    }

    /// Enables text mode
    ///
    /// This currently forces the user to select the internal ROM font.
    pub fn text_mode(&mut self) -> Result<(), SpiError<SPI>> {
        match self.mode {
            Mode::Text => Ok(()),
            Mode::Graphics => {
                let tmp = self.read_register(Register::Mwcr0)?;
                block!(self.write_data(tmp | cmds::Mwcr0::TxtMode as u8))?;

                // Sets the internal ROM font.
                // TODO: Get the register names + values for this so it isn't so cryptic.
                block!(self.write_command(0x21))?;
                let tmp = block!(self.read_data())?;
                block!(self.write_data(tmp & ((1 << 7) | (1 << 5))))?;

                // Clear serial font ROM settings
                block!(self.write_command(0x2F))?;
                block!(self.write_data(0x00))?;

                self.mode = Mode::Text;

                Ok(())
            }
        }
    }

    pub fn set_text_scale(&mut self, scale: u8) -> Result<(), SpiError<SPI>> {
        let bit_pattern = match scale {
            0 => 0b0000,
            1 => 0b0101,
            2 => 0b1010,
            3 => 0b1111,
            _ => 0b1111,
        };
        let mut tmp = self.read_register(Register::FontOptions)?;
        tmp &= !(0xF);
        block!(self.write_data(tmp | bit_pattern))?;

        self.text_settings.text_scale = scale;

        Ok(())
    }

    /// Enables graphics mode
    pub fn graphics_mode(&mut self) -> Result<(), SpiError<SPI>> {
        match self.mode {
            Mode::Graphics => Ok(()),
            Mode::Text => {
                let tmp = self.read_register(Register::Mwcr0)?;
                block!(self.write_data(tmp & !(cmds::Mwcr0::TxtMode as u8)))?;
                self.mode = Mode::Graphics;
                Ok(())
            }
        }
    }

    /// Low-level function to push a raw chunk of pixel data.
    fn push_pixels(&mut self, num_pixels: u32, color: u16) -> Result<(), SpiError<SPI>> {
        block!(self.spi.send(Command::DataWrite as u8))?;
        for _ in 0..num_pixels {
            block!(self.spi.send((color >> 8) as u8))?;
            block!(self.spi.send(color as u8))?;
        }
        Ok(())
    }

    /// Sets the cursor position for the current display mode.
    fn set_cursor(&mut self, new_position: Coord) -> Result<(), SpiError<SPI>> {
        let (x, y) = new_position;
        match self.mode {
            Mode::Graphics => {
                self.write_register(Register::CurH0, x as u8)?;
                self.write_register(Register::CurH1, (x >> 8) as u8)?;
                self.write_register(Register::CurV0, y as u8)?;
                self.write_register(Register::CurV1, (y >> 8) as u8)?;
                self.gfx_settings.cursor = new_position;
                Ok(())
            }
            Mode::Text => {
                self.write_register(Register::TextX0, x as u8)?;
                self.write_register(Register::TextX1, (x >> 8) as u8)?;
                self.write_register(Register::TextY0, y as u8)?;
                self.write_register(Register::TextY1, (y >> 8) as u8)?;
                self.text_settings.cursor = new_position;
                Ok(())
            }
        }
    }

    /// Sets the colors for the current display mode. If `bg_color` is `None`, then a transparent
    /// background will be used.
    fn set_colors(&mut self, fg_color: u16, bg_color: Option<u16>) -> Result<(), SpiError<SPI>> {
        match self.mode {
            Mode::Graphics => {
                // TODO
                Ok(())
            }
            Mode::Text => {
                self.write_register(Register::Color0, ((fg_color & 0xf800) >> 11) as u8)?;
                self.write_register(Register::Color1, ((fg_color & 0x07e0) >> 5) as u8)?;
                self.write_register(Register::Color2, (fg_color & 0x001f) as u8)?;

                match bg_color {
                    Some(color) => {
                        self.write_register(Register::TextBg0, ((color & 0xf800) >> 11) as u8)?;
                        self.write_register(Register::TextBg1, ((color & 0x07e0) >> 5) as u8)?;
                        self.write_register(Register::TextBg2, (color & 0x001f) as u8)?;
                        // Clear transparency flag
                        let tmp = self.read_register(Register::FontOptions)?;
                        block!(self.write_data(tmp & !(1 << 6)))?;
                    }
                    None => {
                        // Set transparency flag
                        let tmp = self.read_register(Register::FontOptions)?;
                        block!(self.write_data(tmp | (1 << 6)))?;
                    }
                }

                self.text_settings.fg_color = fg_color;
                self.text_settings.bg_color = bg_color;

                Ok(())
            }
        }
    }

    fn fill_rect(&mut self) -> Result<(), SpiError<SPI>> {
        block!(self.write_command(Register::Dcr as u8))?;
        block!(self.write_data(cmds::Dcr::DRAWSQUARE as u8))?;
        block!(self.write_data(
            cmds::Dcr::LINESQUTRI_START as u8 | cmds::Dcr::FILL as u8 | cmds::Dcr::DRAWSQUARE as u8
        ))?;
        Ok(())
    }

    /// Draw a single `color` colored point at coordinate `coord`.
    pub fn draw_point(&mut self, coord: Coord, color: u16) -> Result<(), SpiError<SPI>> {
        self.set_cursor(coord)?;
        block!(self.write_command(Register::Mrwc as u8))?;
        block!(self.spi.send(Command::DataWrite as u8))?;
        block!(self.spi.send((color >> 8) as u8))?;
        block!(self.spi.send(color as u8))?;
        Ok(())
    }

    pub fn draw_line(&mut self, start: Coord, end: Coord, color: u16) -> Result<(), SpiError<SPI>> {
        let (x0, y0) = start;
        self.write_register(Register::ShapeStartX0, x0 as u8)?;
        self.write_register(Register::ShapeStartX1, (x0 >> 8) as u8)?;
        self.write_register(Register::ShapeStartY0, y0 as u8)?;
        self.write_register(Register::ShapeStartY1, (y0 >> 8) as u8)?;
        let (x1, y1) = end;
        self.write_register(Register::ShapeEndX0, x1 as u8)?;
        self.write_register(Register::ShapeEndX1, (x1 >> 8) as u8)?;
        self.write_register(Register::ShapeEndY0, y1 as u8)?;
        self.write_register(Register::ShapeEndY1, (y1 >> 8) as u8)?;
        self.set_colors(color, None)?;
        self.write_register(Register::Dcr, 0x80)?;
        // Wait for command to finish
        while (self.read_register(Register::Dcr)? & 0x80) != 0x00 {}
        Ok(())
    }

    pub fn draw_vline(
        &mut self,
        start: Coord,
        height: i16,
        color: u16,
    ) -> Result<(), SpiError<SPI>> {
        self.draw_line(start, (start.0, start.1 + height), color)
    }

    pub fn draw_hline(
        &mut self,
        start: Coord,
        width: i16,
        color: u16,
    ) -> Result<(), SpiError<SPI>> {
        self.draw_line(start, (start.0 + width, start.1), color)
    }

    pub fn draw_rect(
        &mut self,
        corner: Coord,
        width: i16,
        height: i16,
        color: u16,
        fill: bool,
    ) -> Result<(), SpiError<SPI>> {
        let (x0, y0) = corner;
        self.write_register(Register::ShapeStartX0, x0 as u8)?;
        self.write_register(Register::ShapeStartX1, (x0 >> 8) as u8)?;
        self.write_register(Register::ShapeStartY0, y0 as u8)?;
        self.write_register(Register::ShapeStartY1, (y0 >> 8) as u8)?;
        self.write_register(Register::ShapeEndX0, width as u8)?;
        self.write_register(Register::ShapeEndX1, (width >> 8) as u8)?;
        self.write_register(Register::ShapeEndY0, height as u8)?;
        self.write_register(Register::ShapeEndY1, (height >> 8) as u8)?;
        self.set_colors(color, None)?;
        if fill {
            self.write_register(Register::Dcr, 0xB0)?;
        } else {
            self.write_register(Register::Dcr, 0x90)?;
        }
        // Wait for command to finish
        while (self.read_register(Register::Dcr)? & 0x80) != 0x00 {}
        Ok(())
    }

    pub fn fill_screen(&mut self, color: u16) -> Result<(), SpiError<SPI>> {
        let (width, height) = self.dims;
        self.draw_rect((0, 0), width as i16, height as i16, color, true)
    }

    pub fn draw_circle(
        &mut self,
        center: Coord,
        radius: i16,
        color: u16,
        fill: bool,
    ) -> Result<(), SpiError<SPI>> {
        let (x0, y0) = center;
        self.write_register(Register::CircleX0, x0 as u8)?;
        self.write_register(Register::CircleX1, (x0 >> 8) as u8)?;
        self.write_register(Register::CircleY0, y0 as u8)?;
        self.write_register(Register::CircleY1, (y0 >> 8) as u8)?;
        self.write_register(Register::CircleR, radius as u8)?;
        self.set_colors(color, None)?;
        if fill {
            self.write_register(
                Register::Dcr,
                cmds::Dcr::CIRCLE_START as u8 | cmds::Dcr::FILL as u8,
            )?;
        } else {
            self.write_register(Register::Dcr, cmds::Dcr::CIRCLE_START as u8)?;
        }
        // Wait for command to finish
        while (self.read_register(Register::Dcr)? & cmds::Dcr::CIRCLE_START as u8) != 0x00 {}
        Ok(())
    }

    pub fn draw_triangle(
        &mut self,
        (x0, y0): Coord,
        (x1, y1): Coord,
        (x2, y2): Coord,
        color: u16,
        fill: bool,
    ) -> Result<(), SpiError<SPI>> {
        // Point 0
        self.write_register(Register::ShapeStartX0, x0 as u8)?;
        self.write_register(Register::ShapeStartX1, (x0 >> 8) as u8)?;
        self.write_register(Register::ShapeStartY0, y0 as u8)?;
        self.write_register(Register::ShapeStartY1, (y0 >> 8) as u8)?;

        // Point 1
        self.write_register(Register::ShapeEndX0, x1 as u8)?;
        self.write_register(Register::ShapeEndX1, (x1 >> 8) as u8)?;
        self.write_register(Register::ShapeEndY0, y1 as u8)?;
        self.write_register(Register::ShapeEndY1, (y1 >> 8) as u8)?;

        // Point 2
        self.write_register(Register::TriangleP2X0, x2 as u8)?;
        self.write_register(Register::TriangleP2X1, (x2 >> 8) as u8)?;
        self.write_register(Register::TriangleP2Y0, y2 as u8)?;
        self.write_register(Register::TriangleP2Y1, (y2 >> 8) as u8)?;

        self.set_colors(color, None)?;
        if fill {
            self.write_register(
                Register::Dcr,
                cmds::Dcr::LINESQUTRI_START as u8 | cmds::Dcr::FILL as u8,
            )?;
        } else {
            self.write_register(Register::Dcr, cmds::Dcr::LINESQUTRI_START as u8)?;
        }
        // Wait for command to finish
        while (self.read_register(Register::Dcr)? & cmds::Dcr::LINESQUTRI_START as u8) != 0x00 {}
        Ok(())
    }

    pub fn draw_ellipse(
        &mut self,
        (x, y): Coord,
        long_axis: u16,
        short_axis: u16,
        color: u16,
        fill: bool,
    ) -> Result<(), SpiError<SPI>> {
        // Center
        self.write_register(Register::EllipseCenterX0, x as u8)?;
        self.write_register(Register::EllipseCenterX1, (x >> 8) as u8)?;
        self.write_register(Register::EllipseCenterY0, y as u8)?;
        self.write_register(Register::EllipseCenterY1, (y >> 8) as u8)?;

        // Long Axis
        self.write_register(Register::EllipseLongA0, long_axis as u8)?;
        self.write_register(Register::EllipseLongA1, (long_axis >> 8) as u8)?;

        // Short Axis
        self.write_register(Register::EllipseShortB0, short_axis as u8)?;
        self.write_register(Register::EllipseShortB1, (short_axis >> 8) as u8)?;

        self.set_colors(color, None)?;

        if fill {
            self.write_register(
                Register::DrawEllipseCR,
                cmds::DrawEllipseCR::DRAWSTART as u8 | cmds::DrawEllipseCR::FILL as u8,
            )?;
        } else {
            self.write_register(
                Register::DrawEllipseCR,
                cmds::DrawEllipseCR::DRAWSTART as u8,
            )?;
        }
        while (self.read_register(Register::DrawEllipseCR)? & cmds::DrawEllipseCR::DRAWSTART as u8)
            != 0x00
        {}

        Ok(())
    }

    pub fn draw_curve(
        &mut self,
        (x, y): Coord,
        long_axis: u16,
        short_axis: u16,
        curve_part: u8,
        color: u16,
        fill: bool,
    ) -> Result<(), SpiError<SPI>> {
        // Center
        self.write_register(Register::EllipseCenterX0, x as u8)?;
        self.write_register(Register::EllipseCenterX1, (x >> 8) as u8)?;
        self.write_register(Register::EllipseCenterY0, y as u8)?;
        self.write_register(Register::EllipseCenterY1, (y >> 8) as u8)?;

        // Long Axis
        self.write_register(Register::EllipseLongA0, long_axis as u8)?;
        self.write_register(Register::EllipseLongA1, (long_axis >> 8) as u8)?;

        // Short Axis
        self.write_register(Register::EllipseShortB0, short_axis as u8)?;
        self.write_register(Register::EllipseShortB1, (short_axis >> 8) as u8)?;

        self.set_colors(color, None)?;

        if fill {
            self.write_register(
                Register::DrawEllipseCR,
                cmds::DrawEllipseCR::DRAWSTART as u8
                    | cmds::DrawEllipseCR::FILL as u8
                    | cmds::DrawEllipseCR::ELLIPSE_CURVE_SEL as u8
                    | (curve_part & cmds::DrawEllipseCR::EllipseCurvePart as u8),
            )?;
        } else {
            self.write_register(
                Register::DrawEllipseCR,
                cmds::DrawEllipseCR::DRAWSTART as u8
                    | cmds::DrawEllipseCR::ELLIPSE_CURVE_SEL as u8
                    | (curve_part & cmds::DrawEllipseCR::EllipseCurvePart as u8),
            )?;
        }
        while (self.read_register(Register::DrawEllipseCR)? & cmds::DrawEllipseCR::DRAWSTART as u8)
            != 0x00
        {}

        Ok(())
    }

    /// Enable the touch panel, establish auto mode, and enable touch interrupts.
    fn enable_touch(&mut self) -> Result<(), SpiError<SPI>> {
        self.write_register(
            Register::Tpcr0,
            cmds::Tpcr0::ENABLE as u8
                | cmds::Tpcr0::WAIT_16384CLK as u8
                | cmds::Tpcr0::ADCCLK_DIV32 as u8,
        )?;
        self.write_register(
            Register::Tpcr1,
            cmds::Tprc1::AUTO as u8 | cmds::Tprc1::DEBOUNCE as u8,
        )?;
        let tmp = self.read_register(Register::Intc1)?;
        self.write_register(Register::Intc1, tmp | cmds::Intc1::TP as u8)?;
        Ok(())
    }

    /// Check if touch event interrupt occurred
    pub fn touched(&mut self) -> Result<bool, SpiError<SPI>> {
        Ok(self.read_register(Register::Intc2)? & cmds::Intc2::TP as u8 != 0x00)
    }

    pub fn get_touch(&mut self) -> Result<Coord, SpiError<SPI>> {
        // unimplemented!()
        let tx_high = self.read_register(Register::Tpxh)? as u16;
        let ty_high = self.read_register(Register::Tpyh)? as u16;
        let t_xy_lower_bits = self.read_register(Register::Tpxyl)? as u16;
        let tx = (tx_high << 2) | (t_xy_lower_bits & 0x03);
        let ty = (ty_high << 2) | ((t_xy_lower_bits >> 2) & 0x03);

        // Clear the touch interrupt
        self.write_register(Register::Intc2, cmds::Intc2::TP as u8)?;

        Ok((tx as i16, ty as i16))
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
    vsync_start: u16,
}

impl<SPI: FullDuplex<u8>, P: InputPin> Write for RA8875<SPI, P> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        match self.mode {
            Mode::Text => {
                block!(self.write_command(Register::Mrwc as u8)).ok();
                for c in s.as_bytes() {
                    block!(self.write_data(*c)).ok();
                }
                Ok(())
            }
            Mode::Graphics => Err(fmt::Error),
        }
    }
}

fn to_coord(p: Point) -> Coord {
    (p.x as i16, p.y as i16)
}

impl<SPI: FullDuplex<u8>, P: InputPin> DrawTarget<Rgb565> for RA8875<SPI, P> {
    type Error = SpiError<SPI>;

    fn draw_pixel(&mut self, item: drawable::Pixel<Rgb565>) -> Result<(), Self::Error> {
        self.draw_point((item.0.x as i16, item.0.y as i16), item.1.into_storage())
    }

    fn size(&self) -> Size {
        Size::new(self.dims.0 as u32, self.dims.1 as u32)
    }

    fn clear(&mut self, color: Rgb565) -> Result<(), Self::Error>
    where
        Self: Sized,
    {
        self.fill_screen(color.into_storage())
    }

    fn draw_line(
        &mut self,
        item: &Styled<primitives::Line, PrimitiveStyle<Rgb565>>,
    ) -> Result<(), Self::Error> {
        self.draw_line(to_coord(item.primitive.start), to_coord(item.primitive.end), item.style.stroke_color.unwrap().into_storage())
    }

    fn draw_triangle(
        &mut self,
        item: &Styled<primitives::Triangle, PrimitiveStyle<Rgb565>>,
    ) -> Result<(), Self::Error> {
        // TODO: Currently don't allow fill colors different from stroke color
        self.draw_triangle(to_coord(item.primitive.p1), to_coord(item.primitive.p2), to_coord(item.primitive.p3), item.style.stroke_color.unwrap().into_storage(), item.style.fill_color.is_some())
    }

    fn draw_rectangle(
        &mut self,
        item: &Styled<primitives::Rectangle, PrimitiveStyle<Rgb565>>,
    ) -> Result<(), Self::Error> {
        // TODO: Currently don't allow fill colors different from stroke color
        let dimensions = item.bottom_right() - item.top_left();
        let (width, height) = (dimensions.x, dimensions.y);
        self.draw_rect(
            to_coord(item.top_left()),
            width as i16,
            height as i16,
            item.style.stroke_color.unwrap().into_storage(),
            item.style.fill_color.is_some()
        )
    }

    fn draw_circle(
        &mut self,
        item: &Styled<primitives::Circle, PrimitiveStyle<Rgb565>>,
    ) -> Result<(), Self::Error> {
        self.draw_circle(
            to_coord(item.primitive.center),
            item.primitive.radius as i16,
            item.style.stroke_color.unwrap().into_storage(),
            item.style.fill_color.is_some()
        )
    }

}
