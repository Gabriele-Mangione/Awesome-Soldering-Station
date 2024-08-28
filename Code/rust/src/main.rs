use core::time;
use std::{thread, time::SystemTime};

use embedded_graphics::{
    mono_font::{ascii::FONT_6X12, MonoTextStyle},
    pixelcolor::{self, Rgb565},
    prelude::{PixelColor, Point, RgbColor, Size, WebColors},
    primitives::{
        triangle::StyledPixelsIterator, CornerRadii, Primitive, PrimitiveStyle, Rectangle,
        RoundedRectangle, Triangle,
    },
    text::Text,
    Drawable,
};
use esp_idf_hal::{gpio::ADCPin, units::MilliSeconds};
use esp_idf_svc::{
    hal::{
        i2c::{self},
        peripherals::Peripherals,
    },
    sys::EspError,
    systime::EspSystemTime,
    timer,
};

use ass::fusb302;

fn main() -> Result<(), EspError> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;


    /*
    let sda = peripherals.pins.gpio1;
    let scl = peripherals.pins.gpio2;

    let i2c_config = i2c::I2cConfig::new().baudrate(10000.into());
    let i2c_driver = i2c::I2cDriver::new(peripherals.i2c0, sda, scl, &i2c_config)?;

    let _fusb = fusb302::Fusb::new(i2c_driver, 0x68);

    log::info!("Hello, world!");
    loop {
        log::info!("Hello, world!");
        FreeRtos::delay_ms(1000);
    }
    */

    let mut screen = ass::ili9341::ILI9341::new(peripherals.pins);

    let timeno = SystemTime::now();
    Rectangle::new(Point::new(0, 0), Size::new(320, 240))
        .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
        .draw(&mut screen);

    RoundedRectangle::new(
        Rectangle::new(Point::new(260, 20), Size::new(40, 30)),
        CornerRadii::new(Size::new_equal(5)),
    )
    .into_styled(PrimitiveStyle::with_fill(Rgb565::GREEN))
    .draw(&mut screen);

    let timepassed = SystemTime::now().duration_since(timeno);
    log::info!("{}", timepassed.unwrap().as_millis());

    Triangle::new(
        Point::new(10, 10),
        Point::new(200, 10),
        Point::new(105, 200),
    )
    .into_styled(PrimitiveStyle::with_stroke(Rgb565::CSS_BLUE, 3))
    .draw(&mut screen);

    Triangle::new(Point::new(20, 20), Point::new(40, 10), Point::new(60, 100))
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::CSS_RED, 3))
        .draw(&mut screen);

    let style = MonoTextStyle::new(&FONT_6X12, Rgb565::WHITE);

    Text::new("This is a text", Point::new(50, 50), style).draw(&mut screen);

    loop {
        thread::sleep_ms(100);
    }

    Ok(())
}
