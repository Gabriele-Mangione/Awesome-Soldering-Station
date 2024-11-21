use core::time;
use std::{borrow::BorrowMut, ops::Deref, thread, time::SystemTime};

use embedded_graphics::{
    mono_font::{ascii::FONT_6X12, iso_8859_10::FONT_10X20, MonoTextStyle},
    pixelcolor::{self, Rgb565},
    prelude::{PixelColor, Point, RgbColor, Size, WebColors},
    primitives::{
        triangle::StyledPixelsIterator, CornerRadii, Primitive, PrimitiveStyle, Rectangle,
        RoundedRectangle, Triangle,
    },
    text::{renderer::CharacterStyle, Text},
    Drawable,
};
use esp_idf_hal::{
    gpio::{ADCPin, Pin, PinDriver},
    peripheral::Peripheral,
    units::MilliSeconds,
};
use esp_idf_svc::{
    hal::{
        i2c::{self},
        peripherals::Peripherals,
    },
    sys::EspError,
    systime::EspSystemTime,
    timer,
};

use ass::touchbreakout;
use ass::{fusb302, touchbreakout::TouchBreakout};

fn main() -> Result<(), EspError> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let mut peripherals = Peripherals::take()?;
    {
        let reset_pin = unsafe { peripherals.pins.gpio0.clone_unchecked() };
        unsafe {
            let g41 = peripherals.pins.gpio41.clone_unchecked();
            let g42 = peripherals.pins.gpio42.clone_unchecked();
            let g40 = peripherals.pins.gpio40.clone_unchecked();
            let g39 = peripherals.pins.gpio39.clone_unchecked();
            let g38 = peripherals.pins.gpio38.clone_unchecked();
            let g37 = peripherals.pins.gpio37.clone_unchecked();
            let g36 = peripherals.pins.gpio36.clone_unchecked();
            let g35 = peripherals.pins.gpio35.clone_unchecked();
            let g44 = peripherals.pins.gpio44.clone_unchecked();
            let g43 = peripherals.pins.gpio43.clone_unchecked();
            let g48 = peripherals.pins.gpio48.clone_unchecked();
            let g47 = peripherals.pins.gpio47.clone_unchecked();
            let mut pd = PinDriver::input_output(reset_pin)?;
            PinDriver::input_output(g41);
            PinDriver::input_output(g42);
            PinDriver::input_output(g40);
            PinDriver::input_output(g39);
            PinDriver::input_output(g38);
            PinDriver::input_output(g37);
            PinDriver::input_output(g36);
            PinDriver::input_output(g35);
            PinDriver::input_output(g44);
            PinDriver::input_output(g43);
            PinDriver::input_output(g48);
            PinDriver::input_output(g47);
            pd.set_high();
        }
        //drop(pd);
    }

    log::info!("init screen!");
    let mut screen = ass::ili9341::ILI9341::new(/*peripherals.pins*/);

    let timeno = SystemTime::now();

    //draw black screen
    Rectangle::new(Point::new(0, 0), Size::new(320, 240))
        .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
        .draw(&mut screen);

    let timepassed = SystemTime::now().duration_since(timeno).unwrap();
    log::info!("{}", timepassed.as_millis());

    /*
    RoundedRectangle::new(
        Rectangle::new(Point::new(260, 20), Size::new(40, 30)),
        CornerRadii::new(Size::new_equal(5)),
    )
    .into_styled(PrimitiveStyle::with_fill(Rgb565::GREEN))
    .draw(&mut screen);
    */

    /*
    log::info!("init touch!");
    let yp = peripherals.pins.gpio1;
    let xm = peripherals.pins.gpio2;
    let ym = peripherals.pins.gpio14;
    let xp = peripherals.pins.gpio13;

    let xpa = xp.adc_channel();
    let ypa = yp.adc_channel();

    let mut ts = TouchBreakout::new(xp.into(), yp.into(), xm.into(), ym.into(), xpa, ypa);

    */
    let mut pdo_vec: Vec<fusb302::PDO> = vec![];
    {
        let sda = unsafe { peripherals.pins.gpio5.clone_unchecked() };
        let scl = unsafe { peripherals.pins.gpio4.clone_unchecked() };

        let i2c_config = i2c::I2cConfig::new().baudrate(1000000.into()); //0 - 1MHz
        let i2c_driver = i2c::I2cDriver::new(peripherals.i2c0, sda, scl, &i2c_config)?;

        log::info!("init fusb!");

        let mut fusb = fusb302::Fusb::new(i2c_driver, 0x22);
        log::info!("scan pds!");
        pdo_vec = fusb.scan_pds()?;
        log::info!("request pdo!");

        if pdo_vec.len() > 0 {
            let found_pdo = pdo_vec.iter().find(|&&x| x.voltage == 9000);
            if found_pdo.is_some() {
                fusb.request_pdo(*found_pdo.unwrap(), 3000, 3000)?;
            }
        }
        log::info!("done");
    }

    let mut style = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);
    style.set_background_color(Some(Rgb565::BLACK));

    let st = format!("pdo amount: {}", pdo_vec.len());
    Text::new(&st, Point::new(50, 75), style).draw(&mut screen);
    for pdo in pdo_vec {
        log::info!("print pdo");
        let s = format!("V: {}, I: {}, id: {}", pdo.voltage, pdo.current, pdo.id);
        Text::new(&s, Point::new(50, 110 + pdo.id as i32 * 25), style).draw(&mut screen);
    }
    Text::new("This is a text", Point::new(50, 50), style).draw(&mut screen);

    loop {
        let time_stamp = SystemTime::now().duration_since(timeno).unwrap();
        let mut str = "This is a text ".to_owned() + &time_stamp.as_millis().to_string();
        Text::new(&str, Point::new(50, 50), style).draw(&mut screen);
        //let p = ts.get_y()?;
        thread::sleep_ms(100);
        //log::info!("touch!{}", p);
    }
}
