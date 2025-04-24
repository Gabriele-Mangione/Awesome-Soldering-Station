extern crate alloc;
use alloc::boxed::Box;
use embedded_graphics::prelude::Point;

use esp_hal::{
    analog::adc::{Adc, AdcChannel, AdcConfig, AdcPin, RegisterAccess}, gpio::{AnalogPin, AnyPin, GpioPin, Input, InputPin, Output, OutputPin}, i2c::master::{Config, Error, I2c}, peripheral::Peripheral, peripherals::{ADC1, ADC2}, Blocking
};

use core::{borrow::BorrowMut, char::from_digit, ptr::write_volatile};

//use crate::adc_monitor_link::*;TouchBreakoutCap
//TODO:
//https://www.buydisplay.com/download/ic/FT6206.pdf

pub struct TouchBreakoutCap<'a> {
    i2c: I2c<'a, Blocking>,
    i2c_address: u8,
}
impl<'a> TouchBreakoutCap<'a> {
    pub fn new(
        sda: AnyPin,
        scl: AnyPin,
        i2c: impl Peripheral<P = impl esp_hal::i2c::master::Instance> + 'a,
        i2c_address: u8,
    ) -> TouchBreakoutCap<'a> {
        let i2c = I2c::new(i2c, Config::default())
            .unwrap()
            .with_sda(sda)
            .with_scl(scl);
        Self { i2c, i2c_address }
    }
}

impl TouchBreakoutCap<'_> {
}

impl TouchBreakoutCap<'_> {
    fn write_reg(&mut self, reg: u8, val: u8) -> Result<(), Error> {
        self.i2c.write(self.i2c_address, &[reg, val])
    }
    fn read_reg_to_buf(&mut self, reg: u8, buf: &mut u8) -> Result<(), Error> {
        self.i2c.write_read(self.i2c_address, &[reg], &mut [*buf])
    }

    #[inline]
    fn read_reg(&mut self, reg: u8) -> Result<u8, Error> {
        let mut buf: [u8; 1] = [0];
        self.i2c.write_read(self.i2c_address, &[reg], &mut buf)?;
        Ok(buf[0])
    }
}
