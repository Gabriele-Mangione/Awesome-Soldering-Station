use core::time;
use std::{borrow::BorrowMut, marker::PhantomData, ops::Deref, thread, time::SystemTime};

use embedded_graphics::{
    mono_font::{ascii::FONT_6X12, iso_8859_10::FONT_10X20, MonoTextStyle},
    pixelcolor::{self, Rgb565},
    prelude::{PixelColor, Point, RgbColor, Size, WebColors},
    primitives::{
        triangle::StyledPixelsIterator, Circle, CornerRadii, Primitive, PrimitiveStyle, Rectangle,
        RoundedRectangle, Triangle,
    },
    text::{renderer::CharacterStyle, Text},
    Drawable,
};
use esp_idf_hal::{
    gpio::{ADCPin, Gpio8, Pin, PinDriver},
    peripheral::Peripheral,
    sys::gpio_set_level,
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
    log::info!("init screen!");
    let mut screen = ass::ili9341::ILI9341::new();

    let timeno = SystemTime::now();

    //draw black screen
    Rectangle::new(Point::new(0, 0), Size::new(320, 240))
        .into_styled(PrimitiveStyle::with_fill(ass::MyColor(0, 0)))
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

    let mut pdo_vec: Vec<fusb302::PDO> = vec![];
    let mut style = MonoTextStyle::new(&FONT_10X20, ass::MyColor(255, 255));
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
                Text::new("A PDO has been requested", Point::new(50, 170), style).draw(&mut screen);
            }
        }
        log::info!("done");
    }

    style.set_background_color(Some(ass::MyColor(0, 0)));

    let st = format!("pdo amount: {}", pdo_vec.len());
    Text::new(&st, Point::new(50, 75), style).draw(&mut screen);
    for pdo in pdo_vec {
        log::info!("print pdo");
        let s = format!("V: {}, I: {}, id: {}", pdo.voltage, pdo.current, pdo.id);
        Text::new(&s, Point::new(50, 110 + pdo.id as i32 * 25), style).draw(&mut screen);
    }
    Text::new("This is a text", Point::new(50, 50), style).draw(&mut screen);

    log::info!("init touch!");
    let yp = peripherals.pins.gpio1;
    let xm = peripherals.pins.gpio2;
    let ym = peripherals.pins.gpio14;
    let xp = peripherals.pins.gpio13;

    let xpa = xp.adc_channel();
    let ypa = yp.adc_channel();
    let xma = xm.adc_channel();

    let mut ts = TouchBreakout::new(
        xp.into(),
        yp.into(),
        xm.into(),
        ym.into(),
        xpa,
        ypa,
        xma,
        320,
        240,
    );

    let mut toggle = 0;

    let mut balls = vec![];

    let mut styles = vec![];
    styles.push(PrimitiveStyle::with_fill(ass::MyColor(0xF8, 0)));
    styles.push(PrimitiveStyle::with_fill(ass::MyColor(0xE0, 0)));
    styles.push(PrimitiveStyle::with_fill(ass::MyColor(0xD0, 0)));
    styles.push(PrimitiveStyle::with_fill(ass::MyColor(0xC0, 0)));
    styles.push(PrimitiveStyle::with_fill(ass::MyColor(0xB0, 0)));
    styles.push(PrimitiveStyle::with_fill(ass::MyColor(0x90, 0)));
    styles.push(PrimitiveStyle::with_fill(ass::MyColor(0x70, 0)));
    styles.push(PrimitiveStyle::with_fill(ass::MyColor(0x50, 0)));
    styles.push(PrimitiveStyle::with_fill(ass::MyColor(0x30, 0)));
    styles.push(PrimitiveStyle::with_fill(ass::MyColor(0x10, 0)));

    loop {
        let time_stamp = SystemTime::now().duration_since(timeno).unwrap();
        let mut str = "This is a text ".to_owned() + &time_stamp.as_millis().to_string();
        Text::new(&str, Point::new(50, 50), style).draw(&mut screen);
        let p1 = ts.get_x()?;
        let p2 = ts.get_y()?;
        let pressure = ts.touch_detection()?;
        let bool_pressure = ts.touch_detection_bool()?;
        let strong = format!("{:5}  {}", pressure, bool_pressure);
        Text::new(&strong, Point::new(50, 100), style).draw(&mut screen);
        let ball = Circle::new(Point::new(p2 - 5, p1 - 5), 10);

        balls.insert(0, ball);
        if balls.len() > 10 {
            balls.pop();
        }
        let mut i: usize = balls.len() - 1;
        let mut rev_balls = balls.clone();
        rev_balls.reverse();
        for b in rev_balls {
            b.into_styled(styles[i]).draw(&mut screen);
            i -= 1;
        }

        thread::sleep_ms(10);
        let str1 = format!("x: {:4}, y{:4}", p1, p2);
        log::info!("{str1}");
    }
}

struct Ball {
    p: Point,
    id: u8,
}

fn solder_task() {
    //read adc temperature pin
    //
    let adc_heat = unsafe { Gpio8::new().adc_channel() };
    let mut adc_handle: adc_oneshot_unit_handle_t = std::ptr::null_mut();
    unsafe {
        let init_config: adc_oneshot_unit_init_cfg_t = adc_oneshot_unit_init_cfg_t {
            unit_id: adc_unit_t_ADC_UNIT_2,
            clk_src: 0,
            ulp_mode: 0,
        };

        let mut config: adc_oneshot_chan_cfg_t = adc_oneshot_chan_cfg_t {
            atten: adc_atten_t_ADC_ATTEN_DB_11,
            bitwidth: adc_bitwidth_t_ADC_BITWIDTH_DEFAULT,
        };
        adc_oneshot_new_unit(&init_config, &mut adc_handle);
        adc_oneshot_config_channel(adc_handle, adc_heat, &mut config);
    }
    let mut out: i32 = 0;
    adc_oneshot_read(adc_handle, adc_heat, std::ptr::from_mut(&mut out));
    //read saved temperature calibration values
    

    //convert to right temperature
    //(PID)
    //adjust output duty cycle

    //todo implement PID
}
