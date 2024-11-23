use std::borrow::BorrowMut;
use std::f32::consts::PI;
use std::ops::Shr;
use std::ptr::write_volatile;
use std::thread;
use std::time::SystemTime;
use std::{fmt::Debug, time::Duration};

use embedded_graphics::mock_display::MockDisplay;
use embedded_graphics::pixelcolor::raw::ToBytes;
use embedded_graphics::pixelcolor::{self, BinaryColor, Rgb565};
use embedded_graphics::prelude::{Dimensions, DrawTarget, IntoStorage, Point, RgbColor, Size};
use embedded_graphics::primitives::{Primitive, PrimitiveStyle, Rectangle, Triangle};
use embedded_graphics::{Drawable, Pixel};
use esp32s3::gpio::out1_w1tc;
use esp_idf_hal::gpio::Pin;
use esp_idf_hal::gpio::Pins;
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::sys::*;
use std::ffi::*;

pub const WIDTH: usize = 320;
pub const HEIGHT: usize = 240;

// https://cdn-shop.adafruit.com/datasheets/ILI9341.pdf
const NOP: u8 = 0x00;
const SOFTWARE_RESET: u8 = 0x01;
const READ_DISPLAY_IDENTIFICATION_INFORMATION: u8 = 0x04;
const READ_DISPLAY_STATUS: u8 = 0x09;
const READ_DISPLAY_POWER_MODE: u8 = 0x0A;
const READ_DISPLAY_MADCTL: u8 = 0x0B;
const READ_DISPLAY_PIXEL_FORMAT: u8 = 0x0C;
const READ_DISPLAY_IMAGE_FORMAT: u8 = 0x0D;
const READ_DISPLAY_SIGNAL_MODE: u8 = 0x0E;
const READ_DISPLAY_SELF_DIAGNOSTIC_RESULT: u8 = 0x0F;

const ENTER_SLEEP_MODE: u8 = 0x10;
const SLEEP_OUT: u8 = 0x11;
const PARTIAL_MODE_ON: u8 = 0x12;
const NORMAL_DISPLAY_MODE_ON: u8 = 0x13;
const DISPLAY_INVERSION_OFF: u8 = 0x20;
const DISPLAY_INVERSION_ON: u8 = 0x21;
const GAMMA_SET: u8 = 0x26;
const DISPLAY_OFF: u8 = 0x28;
const DISPLAY_ON: u8 = 0x29;

const COLUMN_ADDRESS_SET: u8 = 0x2A;
const PAGE_ADDRESS_SET: u8 = 0x2B;
const MEMORY_WRITE: u8 = 0x2C;
const COLOR_SET: u8 = 0x2D;
const MEMORY_READ: u8 = 0x2E;
const PARTIAL_AREA: u8 = 0x30;

const VERTICAL_SCROLLING_DEFINITION: u8 = 0x33;
const TEARING_EFFECT_LINE_OFF: u8 = 0x34;
const TEARING_EFFECT_LINE_ON: u8 = 0x35;
const MEMORY_ACCESS_CONTROL: u8 = 0x36;
const VERTICAL_SCROLLING_START_ADDRESS: u8 = 0x37;

const IDLE_MODE_OFF: u8 = 0x38;
const IDLE_MODE_ON: u8 = 0x39;
const COLMOD_PIXEL_FORMAT_SET: u8 = 0x3A;
const WRITE_MEMORY_CONTINUE: u8 = 0x3C;
const READ_MEMORY_CONTINUE: u8 = 0x3E;
const SET_TEAR_SCANLINE: u8 = 0x44;
const GET_SCANLINE: u8 = 0x45;
const WRITE_DISPLAY_BRIGHTNESS: u8 = 0x51;
const READ_DISPLAY_BRIGHTNESS: u8 = 0x52;
const WRITE_CTRL_DISPLAY: u8 = 0x53;
const READ_CTRL_DISPLAY: u8 = 0x54;
const WRITE_CONTENT_ADAPTIVE_BRIGHTNESS_CONTROL: u8 = 0x55;
const READ_CONTENT_ADAPTIVE_BRIGHTNESS_CONTROL: u8 = 0x56;
const WRITE_CABC_MINIMUM_BRIGHTNESS: u8 = 0x5E;
const READ_CABC_MINIMUM_BRIGHTNESS: u8 = 0x5F;

const READ_ID1: u8 = 0xDA;
const READ_ID2: u8 = 0xDB;
const READ_ID3: u8 = 0xDC;

// LEVEL 2
const RGB_INTERFACE_SIGNAL_CONTROL: u8 = 0xB0;
const FRAME_RATE_CONTROL_NORMAL_MODE_FULL_COLORS: u8 = 0xB1;
const FRAME_RATE_CONTROL_IDLE_MODE_8_COLORS: u8 = 0xB2;
const FRAME_RATE_CONTROL_PARTIALMODE_FULL_COLORS: u8 = 0xB2;
const DISPLAY_INVERSION_CONTROL: u8 = 0xB4;
const BLANKING_PORCH_CONTROL: u8 = 0xB5;
const DISPLAY_FUNCTION_CONTROL: u8 = 0xB6;
const ENTRY_MODE_SET: u8 = 0xB7;
const BACKLIGHT_CONTROL_1: u8 = 0xB8;
const BACKLIGHT_CONTROL_2: u8 = 0xB9;
const BACKLIGHT_CONTROL_3: u8 = 0xBA;
const BACKLIGHT_CONTROL_4: u8 = 0xBB;
const BACKLIGHT_CONTROL_5: u8 = 0xBC;
const BACKLIGHT_CONTROL_7: u8 = 0xBE;
const BACKLIGHT_CONTROL_8: u8 = 0xBF;
const POWER_CONTROL_1: u8 = 0xC0;
const POWER_CONTROL_2: u8 = 0xC1;
const VCOM_CONTROL_1: u8 = 0xC5;
const VCOM_CONTROL_2: u8 = 0xC7;
const NV_MEMORY_WRITE: u8 = 0xD0;
const NV_MEMORY_PROTECTION_KEY: u8 = 0xD1;
const NV_MEMORY_STATUS_READ: u8 = 0xD2;
const READ_ID4: u8 = 0xD3;
const POSITIVE_GAMMA_CORRECTION: u8 = 0xE0;
const NEGATIVE_GAMMA_CORRECTION: u8 = 0xE1;
const DIGITAL_GAMMA_CONTROL_1: u8 = 0xE2;
const DIGITAL_GAMMA_CONTROL_2: u8 = 0xE3;
const INTERFACE_CONTROL: u8 = 0xF6;

// Extend Register command
const POWER_CONTROL_A: u8 = 0xCB;
const POWER_CONTROL_B: u8 = 0xCF;
const DRIVER_TIMING_CONTROL_A: u8 = 0xE8;
const DRIVER_TIMING_CONTROL_A_2: u8 = 0xE9;
const DRIVER_TIMING_CONTROL_B: u8 = 0xEA;
const POWER_ON_SEQUENCE_CONTROL: u8 = 0xED;
const ENABLE_3G: u8 = 0xF2;
const PUMP_RATIO_CONTROL: u8 = 0xF7;

macro_rules! take_pin {
    ($pin:expr, $mode:ident) => {
        esp_idf_hal::gpio::PinDriver::$mode(Into::<esp_idf_hal::gpio::AnyIOPin>::into($pin))
            .unwrap()
    };
    ($pin:expr) => {
        take_pin!($pin, input_output)
    };
}

use esp_idf_hal::gpio::{self, AnyIOPin, InputOutput};
use esp_idf_hal::prelude::Peripherals;

pub struct Pinacolada(gpio::PinDriver<'static, AnyIOPin, InputOutput>);
//pub struct Pinacolada( AnyIOPin);

pub struct ILI9341 {
    dc_state: bool,
    last_data: u8, 
    gpio: esp32s3::GPIO,
                   /*
                   d: [Pinacolada; 8],
                   rd: Pinacolada,
                   wr: Pinacolada,
                   cd: Pinacolada,
                   cs: Pinacolada,
                   reset: Pinacolada,
                   */
}

impl ILI9341 {
    pub fn new(/*pins: Pins*/) -> ILI9341 {
        let mut res = Self {
            dc_state: true,

            last_data: 0, 
            gpio: unsafe { esp32s3::Peripherals::steal() }.GPIO,
                          /*
                          d: [
                              Pinacolada(take_pin!(pins.gpio42)),
                              Pinacolada(take_pin!(pins.gpio41)),
                              Pinacolada(take_pin!(pins.gpio40)),
                              Pinacolada(take_pin!(pins.gpio39)),
                              Pinacolada(take_pin!(pins.gpio38)),
                              Pinacolada(take_pin!(pins.gpio37)),
                              Pinacolada(take_pin!(pins.gpio36)),
                              Pinacolada(take_pin!(pins.gpio35)),
                          ],
                          rd: Pinacolada(take_pin!(pins.gpio44)),
                          wr: Pinacolada(take_pin!(pins.gpio43)),

                          reset: Pinacolada(take_pin!(pins.gpio0)),

                          cd: Pinacolada(take_pin!(pins.gpio48)),
                          cs: Pinacolada(take_pin!(pins.gpio47)),
                          */
        };
        /*
        let mut what: i32;

        unsafe {
            what = gpio_set_direction(42, gpio_mode_t_GPIO_MODE_INPUT_OUTPUT);
            what |= gpio_set_direction(41, gpio_mode_t_GPIO_MODE_INPUT_OUTPUT);
            what |= gpio_set_direction(40, gpio_mode_t_GPIO_MODE_INPUT_OUTPUT);
            what |= gpio_set_direction(39, gpio_mode_t_GPIO_MODE_INPUT_OUTPUT);
            what |= gpio_set_direction(38, gpio_mode_t_GPIO_MODE_INPUT_OUTPUT);
            what |= gpio_set_direction(37, gpio_mode_t_GPIO_MODE_INPUT_OUTPUT);
            what |= gpio_set_direction(36, gpio_mode_t_GPIO_MODE_INPUT_OUTPUT);
            what |= gpio_set_direction(35, gpio_mode_t_GPIO_MODE_INPUT_OUTPUT);
            what |= gpio_set_direction(44, gpio_mode_t_GPIO_MODE_OUTPUT);
            what |= gpio_set_direction(43, gpio_mode_t_GPIO_MODE_OUTPUT);
            //gpio_set_direction(0, gpio_mode_t_GPIO_MODE_INPUT_OUTPUT);
            what |= gpio_set_direction(48, gpio_mode_t_GPIO_MODE_OUTPUT);
            what |= gpio_set_direction(47, gpio_mode_t_GPIO_MODE_OUTPUT);
        };
        unsafe {
            let c_string = CStr::from_ptr(esp_err_to_name(what));
            let c_st = c_string.to_string_lossy();
            log::info!("res: {}", c_st.to_string());
        }
        */

        let gpio = unsafe { esp32s3::Peripherals::steal() }.GPIO;

        //enable pin registers
        gpio.enable1_w1ts().write(|w| unsafe { w.bits(0x19FF8) });
        //gpio.enable_w1ts().write(|w| unsafe { w.bits(0x1) });

        //set all pins high
        gpio.out1_w1ts().write(|w| unsafe { w.bits(0x19800) });
        //gpio.out_w1ts().write(|w| unsafe { w.bits(1) });

        //std::thread::sleep(Duration::from_secs(1));

        /*
        res.rd.0.set_high().unwrap();
        res.wr.0.set_high().unwrap();
        res.reset.0.set_high().unwrap();
        res.cs.0.set_high().unwrap();
        res.cd.0.set_high().unwrap();
        */

        //set cs low
        gpio.out1_w1tc().write(|w| unsafe { w.bits(1 << 15) });

        res.software_reset();

        //std::thread::sleep(Duration::from_secs(1));

        res.power_control_a()
            .power_control_b()
            .driver_timing_control_a()
            .driver_timing_control_b()
            .power_on_sequence_control()
            .pump_ratio_control()
            .power_control_1()
            .power_control_2()
            .vcom_control_1()
            .vcom_control_2()
            .memory_access_control()
            .colmod_pixel_format_set()
            .frame_rate_control_normal_mode_full_colors()
            .display_function_control()
            .enable_3g()
            .gamma_set()
            .positive_gamma_correction()
            .sleep_out();
        std::thread::sleep(Duration::from_millis(160));

        res.display_on();

        res
    }

    #[inline]
    pub fn display_inversion_on(&mut self) -> &mut Self {
        self.write_command(DISPLAY_INVERSION_ON)
    }

    #[inline]
    pub fn enter_sleep_mode(&mut self) -> &mut Self {
        self.write_command(ENTER_SLEEP_MODE)
    }

    #[inline]
    pub fn sleep_out(&mut self) -> &mut Self {
        self.write_command(SLEEP_OUT)
    }

    #[inline]
    pub fn display_inversion_off(&mut self) -> &mut Self {
        self.write_command(DISPLAY_INVERSION_OFF)
    }

    #[inline]
    pub fn display_on(&mut self) -> &mut Self {
        self.write_command(DISPLAY_ON)
    }
    #[inline]
    pub fn display_off(&mut self) -> &mut Self {
        self.write_command(DISPLAY_OFF)
    }
    #[inline]
    pub fn software_reset(&mut self) -> &mut Self {
        self.write_command(SOFTWARE_RESET)
    }

    #[inline]
    pub fn power_control_b(&mut self) -> &mut Self {
        // Page 196, the parameters are exact those
        self.write_command(POWER_CONTROL_A)
            .write_data(0x39)
            .write_data(0x2C)
            .write_data(0x00)
            .write_data(0x34)
            .write_data(0x02)
    }

    #[inline]
    pub fn power_control_a(&mut self) -> &mut Self {
        // Page 196, the parameters are exact those
        self.write_command(POWER_CONTROL_B)
            .write_data(0x00)
            .write_data(0xC1) // TODO: the docs says 81
            .write_data(0x30)
    }

    #[inline]
    pub fn driver_timing_control_a(&mut self) -> &mut Self {
        // Page 197, the parameters are exact those
        self.write_command(DRIVER_TIMING_CONTROL_A_2)
            .write_data(0x84) // 84: non overlap; 85: +1 unit
            .write_data(0x00) // 00 | 10 | 01 | 11 -> x0: EQ, 0x: CR
            .write_data(0x78)
    }

    #[inline]
    pub fn driver_timing_control_b(&mut self) -> &mut Self {
        // Page 197, the parameters are exact those
        self.write_command(DRIVER_TIMING_CONTROL_B)
            .write_data(0x00)
            .write_data(0x00)
    }

    #[inline]
    pub fn power_on_sequence_control(&mut self) -> &mut Self {
        self.write_command(POWER_ON_SEQUENCE_CONTROL)
            .write_data(0x64)
            .write_data(0x03)
            .write_data(0x12)
            .write_data(0x81)
    }

    #[inline]
    pub fn pump_ratio_control(&mut self) -> &mut Self {
        self.write_command(PUMP_RATIO_CONTROL).write_data(0x20)
    }

    #[inline]
    pub fn power_control_1(&mut self) -> &mut Self {
        self.write_command(POWER_CONTROL_1).write_data(0x23)
    }

    #[inline]
    pub fn power_control_2(&mut self) -> &mut Self {
        self.write_command(POWER_CONTROL_2).write_data(0x10)
    }

    #[inline]
    pub fn vcom_control_1(&mut self) -> &mut Self {
        self.write_command(VCOM_CONTROL_1)
            .write_data(0x3E)
            .write_data(0x28)
    }

    // Define other functions for remaining commands in a similar way...

    #[inline]
    pub fn positive_gamma_correction(&mut self) -> &mut Self {
        self.write_command(POSITIVE_GAMMA_CORRECTION)
            .write_data(0x0F)
            .write_data(0x31)
            .write_data(0x2B)
            .write_data(0x0C)
            .write_data(0x0E)
            .write_data(0x08)
            .write_data(0x4E)
            .write_data(0xF1)
            .write_data(0x37)
            .write_data(0x07)
            .write_data(0x10)
            .write_data(0x03)
            .write_data(0x0E)
            .write_data(0x09)
            .write_data(0x00)
    }

    #[inline]
    pub fn negative_gamma_correction(&mut self) -> &mut Self {
        self.write_command(NEGATIVE_GAMMA_CORRECTION)
            .write_data(0x00)
            .write_data(0x0E)
            .write_data(0x14)
            .write_data(0x03)
            .write_data(0x11)
            .write_data(0x07)
            .write_data(0x31)
            .write_data(0xC1)
            .write_data(0x48)
            .write_data(0x08)
            .write_data(0x0F)
            .write_data(0x0C)
            .write_data(0x31)
            .write_data(0x36)
            .write_data(0x0F)
    }
    #[inline]
    pub fn vcom_control_2(&mut self) -> &mut Self {
        self.write_command(VCOM_CONTROL_2).write_data(0x86)
    }

    #[inline]
    pub fn memory_access_control(&mut self) -> &mut Self {
        self.write_command(MEMORY_ACCESS_CONTROL).write_data(0x38) //0x48
    }

    #[inline]
    pub fn colmod_pixel_format_set(&mut self) -> &mut Self {
        self.write_command(COLMOD_PIXEL_FORMAT_SET).write_data(0x55)
    }

    #[inline]
    pub fn frame_rate_control(&mut self) -> &mut Self {
        self.write_command(FRAME_RATE_CONTROL_NORMAL_MODE_FULL_COLORS)
            .write_data(0x00)
            .write_data(0x18)
    }

    #[inline]
    pub fn display_function_control(&mut self) -> &mut Self {
        self.write_command(DISPLAY_FUNCTION_CONTROL)
            .write_data(0x08)
            .write_data(0x82)
            .write_data(0x27)
    }

    #[inline]
    pub fn enable_3g(&mut self) -> &mut Self {
        self.write_command(ENABLE_3G).write_data(0x00)
    }

    #[inline]
    pub fn gamma_set(&mut self) -> &mut Self {
        self.write_command(GAMMA_SET).write_data(0x01)
    }

    #[inline]
    pub fn frame_rate_control_normal_mode_full_colors(&mut self) -> &mut Self {
        self.write_command(FRAME_RATE_CONTROL_NORMAL_MODE_FULL_COLORS)
            .write_data(0x00)
            .write_data(0x18)
    }
}

impl ILI9341 {
    fn write8(&mut self, data: u8) -> &mut Self {


        unsafe {
            let out1_w1ts_addr: *mut u32 = 0x60004014 as *mut u32;
            let out1_w1tc_addr: *mut u32 = 0x60004018 as *mut u32;

            let mut out1_w1ts_= *out1_w1ts_addr;
            let mut out1_w1tc_= *out1_w1tc_addr;


        if data != self.last_data {
            let result = ((data.reverse_bits() as u32) << 3);
            //let mask = 0b111111111000;
            //p.GPIO.out1().modify(|r, w| unsafe { w.bits((r.bits() & !mask) | (mask & result)) });

            let inv_result = (((!data.reverse_bits()) as u32) << 3) + 0b1_0000_0000_000;
            //set and clear data and clear wr pin
            //
            /*
            write_volatile(out1_w1tc_addr, inv_result);
            write_volatile(out1_w1ts_addr, result);
            */
            self.gpio.out1_w1tc().write(|w| unsafe { w.bits(inv_result) });
            self.gpio.out1_w1ts().write(|w| unsafe { w.bits(result) });

            //self.gpio.out1_w1ts().write(|w| unsafe { w.bits(0) });
            //self.last_data = data;
        } else {
            self.gpio.out1_w1tc().write(|w| unsafe { w.bits(0b1_0000_0000_000) });
            //write_volatile(out1_w1tc_addr, 0b1_0000_0000_000);
        }

        //set wr pin
        self.gpio.out1_w1ts().write(|w| unsafe { w.bits(0b1_0000_0000_000) });
            //write_volatile(out1_w1ts_addr, 0b1_0000_0000_000);
        }

        self
    }

    fn write_command(&mut self, cmd: u8) -> &mut Self {
        //set dc to send command
        if self.dc_state == true {
            self.dc_state = false;
            //self.cd.0.set_low().unwrap();
            let gpio = unsafe { esp32s3::Peripherals::steal() }.GPIO;
            gpio.out1_w1tc().write(|w| unsafe { w.bits(1 << 16) });
        }

        self.write8(cmd)
    }
    fn write_data(&mut self, data: u8) -> &mut Self {
        //clear dc to send data
        if self.dc_state == false {
            self.dc_state = true;
            //self.cd.0.set_high().unwrap();
            //let p = unsafe { esp32s3::Peripherals::steal() };
            let gpio = unsafe { esp32s3::Peripherals::steal() }.GPIO;
            gpio.out1_w1ts().write(|w| unsafe { w.bits(1 << 16) });
        }

        self.write8(data)
    }

    fn read_data(&mut self) -> u8 {
        //let p = unsafe { esp32s3::Peripherals::steal() };
        //deactivate output
        let gpio = unsafe { esp32s3::Peripherals::steal() }.GPIO;
        gpio.enable1_w1tc().write(|w| unsafe { w.bits(0xFF << 3) });

        let data = (gpio.in1().read().bits() >> 3) as u8;

        //reactivate output
        gpio.enable1_w1ts().write(|w| unsafe { w.bits(0xFF << 3) });

        data
    }
}

impl Dimensions for ILI9341 {
    fn bounding_box(&self) -> Rectangle {
        Rectangle::new(Point::new(0, 0), Size::new(WIDTH as _, HEIGHT as _))
    }
}

impl DrawTarget for ILI9341 {
    type Color = Rgb565;
    type Error = ();

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let mut x: i32 = 0;
        let mut y: i32 = 0;
        for Pixel(point, color) in pixels {
            //write new positions only if changed
            if x != point.x {
                x = point.x;
                self.write_command(COLUMN_ADDRESS_SET)
                    .write_data((x >> 8) as u8)
                    .write_data(x as u8)
                    .write_data((x >> 8) as u8)
                    .write_data(x as u8);
            }
            if y != point.y {
                y = point.y;
                self.write_command(PAGE_ADDRESS_SET)
                    .write_data((y >> 8) as u8)
                    .write_data(y as u8)
                    .write_data((y >> 8) as u8)
                    .write_data(y as u8);
            }

            //write 16 bit color to memory
            //self.write_command(MEMORY_WRITE).write_data(color.into_storage().checked_shr(8).unwrap() as u8).write_data((color.into_storage() & 0xFF) as u8);

            self.write_command(MEMORY_WRITE)
                .write_data(((color.r() << 3) | (color.g() >> 3)) as u8)
                .write_data(((color.g() << 5) | (color.b())) as u8);
        }

        Ok(())
    }

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        let x1 = area.top_left.x;
        let x2 = area.bottom_right().unwrap_or(area.top_left).x;
        let y1 = area.top_left.y;
        let y2 = area.bottom_right().unwrap_or(area.top_left).y;
        //let huh = area.bottom_right().unwrap();
        //log::info!("{}, {}, {}, {}", x1, x2, y1, y2);

        self.write_command(COLUMN_ADDRESS_SET)
            .write_data((x1 >> 8) as u8)
            .write_data(x1 as u8)
            .write_data((x2 >> 8) as u8)
            .write_data(x2 as u8);
        self.write_command(PAGE_ADDRESS_SET)
            .write_data((y1 >> 8) as u8)
            .write_data(y1 as u8)
            .write_data((y2 >> 8) as u8)
            .write_data(y2 as u8);

        self.write_command(MEMORY_WRITE);

        //if fill solid is used color iter will be repeated:
        /*
        fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
            self.fill_contiguous(area, core::iter::repeat(color))
        }
        */
        let mut color_iter = colors.into_iter();
        for pix in 0..(area.size.width * area.size.height) {
            let color = color_iter.next().unwrap();
            // self.wr.0.set_low().unwrap();
            self.write_data(color.to_ne_bytes()[1]);

            /*
            let p = unsafe { esp32s3::Peripherals::steal() };
            p.GPIO.out1_w1ts().write(|w| unsafe { w.bits(0b1_0000_0000_000) });
            */
            //self.wr.0.set_high().unwrap();
            // self.wr.0.set_low().unwrap();
            self.write_data(color.to_ne_bytes()[0]);
            //p.GPIO.out1_w1ts().write(|w| unsafe { w.bits(0b1_0000_0000_000) });
            //self.wr.0.set_high().unwrap();
        }
        /*
        for color in colors {
            self.write_data(((color.r() << 3) | (color.g() >> 3)) as u8)
                .write_data(((color.g() << 5) | (color.b() >> 3)) as u8);
            log::info!("for loop inside");
            thread::sleep_ms(10);

        }
        */
        //log::info!("for loop over");
        Ok(())
    }
}

impl Debug for ILI9341 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            /*
                        format!(
                            "d = [
                |7 6 5 4 3 2 1 0|
                |{} {} {} {} {} {} {} {}|
              ],
              rd: {},
              wr: {},
              cd: {},
              cs: {},
              reset: {}
            ",
                            self.d[7].0.is_set_high() as i8,
                            self.d[6].0.is_set_high() as i8,
                            self.d[5].0.is_set_high() as i8,
                            self.d[4].0.is_set_high() as i8,
                            self.d[3].0.is_set_high() as i8,
                            self.d[2].0.is_set_high() as i8,
                            self.d[1].0.is_set_high() as i8,
                            self.d[0].0.is_set_high() as i8,
                            self.rd.0.is_set_high(),
                            self.wr.0.is_set_high(),
                            self.cd.0.is_set_high(),
                            self.cs.0.is_set_high(),
                            self.reset.0.is_set_high(),
                        )
                        .as_str(),
                        */
            "oh hello",
        )
    }
}
