#![allow()]


use embedded_graphics::{prelude::{PixelColor, RawData}, pixelcolor::raw::RawU16};

pub mod fusb302;
pub mod ili9341;
pub mod touchbreakout;
//pub mod touch_button;

//pub mod fancy_pin;

#[derive(Copy, PartialEq, Clone)]
pub struct MyColor(pub u8, pub u8);

impl PixelColor for MyColor {
    type Raw = RawU16;
    
    
}
/*
impl RawData for MyColor {
    type Storage = u16;
    const BITS_PER_PIXEL: usize = 16;
    
}
*/


impl From<u16> for MyColor {
    fn from(value: u16) -> Self {
        Self((value >> 8) as u8, value as u8)
    }
    
}

