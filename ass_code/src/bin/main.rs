#![no_std]
#![no_main]

extern crate alloc;
use core::ptr::addr_of_mut;

use alloc::borrow::ToOwned;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::{format, vec};
use embassy_time::Timer;
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::prelude::{Point, Size};
use embedded_graphics::primitives::PrimitiveStyle;
use embedded_graphics::primitives::{Circle, Primitive, Rectangle};
use embedded_graphics::text::renderer::CharacterStyle;
use embedded_graphics::text::Text;
use esp_backtrace as _;
use esp_hal::analog::adc::{self, Adc, AdcChannel, AdcConfig, AdcPin};
use esp_hal::clock::CpuClock;
use esp_hal::cpu_control::{CpuControl, Stack};
use esp_hal::delay::Delay;
use esp_hal::gpio::interconnect::PeripheralOutput;
use esp_hal::gpio::{AnalogPin, AnyPin, GpioPin, Input, Io, Level, Output};
use esp_hal::ledc::channel::{self, Channel, ChannelHW, ChannelIFace};
use esp_hal::ledc::timer::{self, TimerIFace};
use esp_hal::ledc::{self, LSGlobalClkSource, Ledc, LowSpeed};
use esp_hal::peripheral::Peripheral;
use esp_hal::peripherals::{ADC1, LEDC};
use esp_hal::time::RateExtU32;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::{main, peripherals};
use esp_storage::FlashStorage;
use log::info;

use embedded_storage::{ReadStorage, Storage};
//use esp_storage::FlashStorage;
use embassy_executor::Spawner;
use esp_hal_embassy::Executor;
use static_cell::StaticCell;

use ass_code::fusb302;
use ass_code::ili9341;
use ass_code::touchbreakout_cap::{self, TouchBreakoutCap, TouchEventFlag};
use embedded_graphics::{self, Drawable};

static mut APP_CORE_STACK: Stack<8192> = Stack::new();

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // generator version: 0.2.2
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    esp_println::logger::init_logger_from_env();

    esp_alloc::heap_allocator!(72 * 1024);

    // init fusb, select 9 volt
    let pdo_vec: Vec<fusb302::PDO>;

    let mut pdo_requested = false;
    {
        log::info!("init fusb!");
        let mut fusb = fusb302::Fusb::new(
            peripherals.GPIO5.into(),
            peripherals.GPIO4.into(),
            peripherals.I2C0,
            0x22,
        );
        log::info!("scan pds!");
        pdo_vec = fusb.scan_pds().unwrap();
        log::info!("request pdo!");
        if pdo_vec.len() > 0 {
            let found_pdo = pdo_vec.iter().find(|&&x| x.voltage == 9000);
            if found_pdo.is_some() {
                fusb.request_pdo(*found_pdo.unwrap(), 3000, 3000).unwrap();
                pdo_requested = true;
            }
        }
        log::info!("done");
    }

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);
    //let delay = Delay::new();

    //for some reason necessary when using reg write for pins. DON'T DELETE
    Output::new(peripherals.GPIO35, Level::Low);
    Output::new(peripherals.GPIO36, Level::Low);
    Output::new(peripherals.GPIO37, Level::Low);
    Output::new(peripherals.GPIO38, Level::Low);
    Output::new(peripherals.GPIO39, Level::Low);
    Output::new(peripherals.GPIO40, Level::Low);
    Output::new(peripherals.GPIO41, Level::Low);
    Output::new(peripherals.GPIO42, Level::Low);
    Output::new(peripherals.GPIO43, Level::Low);
    Output::new(peripherals.GPIO44, Level::Low);
    Output::new(peripherals.GPIO47, Level::Low);
    Output::new(peripherals.GPIO48, Level::Low);

    //init screen
    let mut screen = ass_code::ili9341::ILI9341::new();

    let mut style = MonoTextStyle::new(&FONT_10X20, ass_code::MyColor(255, 255));
    style.set_background_color(Some(ass_code::MyColor(0, 0)));

    //draw black screen
    Rectangle::new(Point::new(0, 0), Size::new(320, 240))
        .into_styled(PrimitiveStyle::with_fill(ass_code::MyColor(0, 0)))
        .draw(&mut screen)
        .unwrap();

    //print pdo info to screen
    if pdo_requested == true {
        Text::new("A PDO has been requested", Point::new(50, 170), style)
            .draw(&mut screen)
            .unwrap();
    }
    let pdo_info = format!("pdo amount: {}", pdo_vec.len());
    Text::new(&pdo_info, Point::new(50, 75), style)
        .draw(&mut screen)
        .unwrap();
    for pdo in pdo_vec {
        log::info!("print pdo");
        let s = format!("V: {}, I: {}, id: {}", pdo.voltage, pdo.current, pdo.id);
        Text::new(&s, Point::new(50, 110 + pdo.id as i32 * 25), style)
            .draw(&mut screen)
            .unwrap();
    }

    log::info!("init touch!");
    let ts_sda = peripherals.GPIO14;
    let ts_scl = peripherals.GPIO13;
    let ts_irq = peripherals.GPIO1;
    let mut ts = TouchBreakoutCap::new(ts_sda.into(), ts_scl.into(), peripherals.I2C1);

    //touch circles
    let mut balls: Vec<Circle> = vec![];
    let mut styles = vec![];
    styles.push(PrimitiveStyle::with_fill(ass_code::MyColor(0xF8, 0)));
    styles.push(PrimitiveStyle::with_fill(ass_code::MyColor(0xE0, 0)));
    styles.push(PrimitiveStyle::with_fill(ass_code::MyColor(0xD0, 0)));
    styles.push(PrimitiveStyle::with_fill(ass_code::MyColor(0xC0, 0)));
    styles.push(PrimitiveStyle::with_fill(ass_code::MyColor(0xB0, 0)));
    styles.push(PrimitiveStyle::with_fill(ass_code::MyColor(0x90, 0)));
    styles.push(PrimitiveStyle::with_fill(ass_code::MyColor(0x70, 0)));
    styles.push(PrimitiveStyle::with_fill(ass_code::MyColor(0x50, 0)));
    styles.push(PrimitiveStyle::with_fill(ass_code::MyColor(0x30, 0)));
    styles.push(PrimitiveStyle::with_fill(ass_code::MyColor(0x10, 0)));

    Text::new("This is a text", Point::new(50, 50), style)
        .draw(&mut screen)
        .unwrap();

    let mut time = 0;

    let mut adc_config = AdcConfig::new();
    let temp_pin =
        adc_config.enable_pin(peripherals.GPIO8, esp_hal::analog::adc::Attenuation::_11dB);
    let adc1 = Adc::new(peripherals.ADC1, adc_config);

    let mut ledc = Ledc::new(peripherals.LEDC);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

    let mut ledc_timer = ledc.timer::<LowSpeed>(timer::Number::Timer0);
    ledc_timer
        .configure(timer::config::Config {
            duty: timer::config::Duty::Duty14Bit, //0- 16384
            clock_source: timer::LSClockSource::APBClk,
            frequency: RateExtU32::Hz(500),
        })
        .unwrap();

    let mut solder_pin = ledc.channel(channel::Number::Channel0, peripherals.GPIO9);
    solder_pin
        .configure(channel::config::Config {
            timer: &ledc_timer,
            duty_pct: 0,
            pin_config: channel::config::PinConfig::PushPull,
        })
        .unwrap();

    //developing handle code
    let rxGyro = Input::new(peripherals.GPIO12, esp_hal::gpio::Pull::Down);
    let mut red_style = MonoTextStyle::new(&FONT_10X20, ass_code::MyColor(255, 0));
    red_style.set_background_color(Some(ass_code::MyColor(0, 0)));

    /*
    loop {
        if rxGyro.is_high() {
            time = 500;
        } else if time >= 0 {
            time -= 1;
        }

        if time > 0 {
            let st = format!("Movement {}", time);
            Text::new(&st, Point::new(50, 180), style)
                .draw(&mut screen)
                .unwrap();
        } else if time == 0 {
            Text::new("no Movement      ", Point::new(50, 180), red_style)
                .draw(&mut screen)
                .unwrap();
        }
    }
    */

    //solder_task::<ADC1, GpioPin<8>>(temp_pin, adc1, solder_pin);
    //

    //spawner.spawn(handle_touch_events(ts)).unwrap();

    loop {
        time += 1;
        let p1 = t.0.x as i32; //crash
        let p2 = 320i32 - t.0.y as i32;
        if p1 != 0 {
            let ball = Circle::new(Point::new(p2 - 5, p1 - 5), 5u32);

            balls.insert(0, ball);
            if balls.len() > 10 {
                balls.pop();
            }
            let mut i: usize = balls.len() - 1;
            let mut rev_balls = balls.clone();
            rev_balls.reverse();
            for b in rev_balls {
                b.into_styled(styles[i]).draw(&mut screen).unwrap();
                i -= 1;
            }
        }

        let str = "This is a text ".to_owned() + &time.to_string();
        Text::new(&str, Point::new(50, 50), style)
            .draw(&mut screen)
            .unwrap();

        Timer::after_millis(5).await;
    }
}

#[embassy_executor::task]
async fn handle_touch_events() {
    //create interrupt for ts

    //use await to be awaken by interrupt?

    //distinguish type of touch and check if clicked a button in window
    /*
    loop {
        let t = ts.read_points().unwrap();

        let s = match t.0.event_flag {
            TouchEventFlag::Contact => "Contact",
            TouchEventFlag::LiftUp => "LiftUp",
            TouchEventFlag::PressDown => "PressDown",
            TouchEventFlag::NoEvent => "NoEvent",
        };
        esp_println::println!("P1: \tx: {:4}\ty: {:4}\te: {}", t.0.x, t.0.y, s);
        Timer::after_millis(5).await;
    }
    */
}

fn calib_temp_points<ADCI, Pinno>(
    mut temp_pin: AdcPin<Pinno, ADCI>,
    mut adc: Adc<ADCI>,
    solder_pin: Channel<LowSpeed>,
) where
    ADCI: adc::RegisterAccess,
    Pinno: AnalogPin + AdcChannel,
{
    //activate output and show buttons

    //user must augment output until the externally measured temperature reaches STABLE 100°C

    //the measured value shall be saved
    solder_pin.set_duty_hw(0);
    let mut out: u32 = 0;
    for _ in 0..10 {
        out += adc.read_blocking(&mut temp_pin) as u32;
    }
    out /= 10;
    let p_at_100c: u16 = out as u16;

    //repeat for 400°C

    //save button
    solder_pin.set_duty_hw(0);
    out = 0;
    for _ in 0..10 {
        out += adc.read_blocking(&mut temp_pin) as u32;
    }
    out /= 10;
    let p_at_400c: u16 = out as u16;

    let mut bytes = [0u8; 4];
    bytes[0] = (p_at_100c >> 8) as u8;
    bytes[1] = (p_at_100c) as u8;
    bytes[2] = (p_at_400c >> 8) as u8;
    bytes[3] = (p_at_400c) as u8;
    //write new calibration values to flash
    let mut flash = FlashStorage::new();
    flash.write(0x9000, &[0x1, 0x2, 0x3, 0x4]).unwrap();
}

//#[embassy_executor::task]
fn solder_task<ADCI, Pinno>(
    mut temp_pin: AdcPin<Pinno, ADCI>,
    mut adc: Adc<ADCI>,
    solder_pin: Channel<LowSpeed>,
) where
    ADCI: adc::RegisterAccess,
    Pinno: AnalogPin + AdcChannel,
{
    let set_temp: u16 = 380;
    //to calibrate
    //1. ki = 0, kd = 0, adjust kp so that the first peak is near the set temp
    //2. drive ki up to the point where the temp is stable at the set temp. (there will be an
    //   overshoot where the first peak was
    //3. adjust kd to flatten the overshoot
    let kp: f32 = 0.8;
    let ki: f32 = 0.005;
    let kd: f32 = 0.5;
    let mut old_diff: f32 = 0.;
    let mut int_diff: f32 = 0.;

    //read saved temperature calibration values
    //
    let mut bytes = [0u8; 4];
    let mut flash = FlashStorage::new();

    flash.write(0x9000, &[0x1, 0x2, 0x3, 0x4]).unwrap();

    flash.read(0x9000, &mut bytes).unwrap();

    //todo convert 2bytes to u16...
    let p_at_100c: u16 = ((bytes[0] as u16) << 8) | bytes[1] as u16;
    let p_at_400c: u16 = ((bytes[2] as u16) << 8) | bytes[3] as u16;
    info!("p @ 100°C: {:5}, p @ 400°C: {:5}", p_at_100c, p_at_400c);

    let mut act_temp: f32 = 0.;
    let mut old_duty: u16 = 0;
    loop {
        //turn off voltage for measurement
        solder_pin.set_duty_hw(0);
        //read adc temperature pin
        let mut out: u32 = 0;
        for _ in 0..10 {
            out += adc.read_blocking(&mut temp_pin) as u32;
        }
        out /= 10;

        if out > 4000 {
            //no soldering tip is connected
            int_diff = 0.;
            old_diff = 0.;
        } else {
            //convert to right temperature
            //let act_temp: f32 =p_at_100c as f32 * 300. * (out as f32 - 1.) / (p_at_400c as f32 - 1.) + 100.;

            //(PID)

            //proportional
            let diff: f32 = set_temp as f32 - act_temp;

            let pro_diff = diff * kp;
            //integral
            int_diff += diff * ki;
            //derivative
            let der_diff = (diff - old_diff) * kd;
            old_diff = diff;

            //adjust output duty cycle
            let duty_cycle: u16 = ((pro_diff + int_diff + der_diff) as u16).clamp(0, 16384);

            solder_pin.set_duty_hw(duty_cycle as u32);

            info!(
                "act_temp: {}, set_temp: {}, duty_cycle: {}",
                act_temp, set_temp, duty_cycle
            );
            info!("pro: {}, int: {}, der: {}", pro_diff, int_diff, der_diff);

            //simulation
            act_temp += 0.5 * old_duty as f32 - 5.;
            old_duty = duty_cycle;

            //would be really cool to have a visualisation of the PID stuff on the screen.
        }

        //let delay = Delay::new();
        //delay.delay_millis(50);
    }
}
