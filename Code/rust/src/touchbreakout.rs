use embedded_graphics::prelude::Point;
use esp_idf_hal::{adc::AdcChannelDriver, gpio::{self,AnyIOPin, InputOutput, PinDriver, Pins}};

pub struct Pin(gpio::PinDriver<'static, AnyIOPin, InputOutput>);

pub struct TouchBreakout {
    xp_pin: Pin,
    yp_pin: Pin,
    xm_pin: Pin,
    ym_pin: Pin,
}

impl TouchBreakout {
    pub fn new(xp_pin: Pin, yp_pin: Pin, xm_pin: Pin, ym_pin: Pin) -> TouchBreakout {
        TouchBreakout {
            xp_pin,
            yp_pin,
            xm_pin,
            ym_pin,
        }
    }

    pub fn get_point(&self) -> Point {
        let x = self.get_x();
        let y = self.get_y();

        Point::new(x,y)
    }

    fn get_x(&self) -> i32 {
        self.xp_pin

    }

    fn get_y(&self) -> i32 {
        self.yp_pin

    }
}
