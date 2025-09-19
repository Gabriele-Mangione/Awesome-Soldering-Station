#![no_std]
#![allow(dead_code)]

use embedded_graphics::{prelude::PixelColor, pixelcolor::raw::RawU16};


pub mod soldering;
pub mod fusb302;
pub mod ili9341;
pub mod touchbreakout_cap;
#[derive(Copy, PartialEq, Clone)]
    //565
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

impl MyColor{
    pub fn from_rgb(r: u8, g:u8, b:u8) ->Self {
        Self((r << 2) | (g>> 3), (g << 5) | b)
    }
}

impl From<u16> for MyColor {
    fn from(value: u16) -> Self {
        Self((value >> 8) as u8, value as u8)
    }
    
}
