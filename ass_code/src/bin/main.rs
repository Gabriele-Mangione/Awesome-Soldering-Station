#![no_std]
#![no_main]

extern crate alloc;
use core::cell::RefCell;
use core::ptr::addr_of_mut;

use alloc::borrow::ToOwned;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::{format, vec};
use ass_code::soldering::Soldering;
//use ass_code::soldering::Soldering;
use critical_section::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
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
use esp_hal::i2c::master::AnyI2c;
use esp_hal::interrupt::InterruptConfigurable;
use esp_hal::ledc::channel::{self, Channel, ChannelHW, ChannelIFace};
use esp_hal::ledc::timer::{self, TimerIFace};
use esp_hal::ledc::{self, LSGlobalClkSource, Ledc, LowSpeed};
use esp_hal::peripheral::Peripheral;
use esp_hal::peripherals::{ADC1, I2C1, IO_MUX, LEDC};
use esp_hal::time::RateExtU32;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::{handler, main, peripherals, ram};
use esp_storage::FlashStorage;
use log::info;

use embedded_storage::{ReadStorage, Storage};
//use esp_storage::FlashStorage;
use embassy_executor::Spawner;
use esp_hal_embassy::Executor;
use static_cell::StaticCell;

use ass_code::fusb302;
use ass_code::ili9341;
use ass_code::touchbreakout_cap::{self, Touch, TouchBreakoutCap, TouchEventFlag};
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

    Text::new("This is a text", Point::new(50, 50), style)
        .draw(&mut screen)
        .unwrap();

    let mut time = 0;


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
    let mut flash = FlashStorage::new();

    /*
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
        */

    let mut soldering = Soldering::new(
        peripherals.GPIO9.into(),
        peripherals.GPIO8,
        peripherals.ADC1,
        peripherals.LEDC,
        flash,
    );

    spawner.spawn(soldering.task());

    let ts_sda = peripherals.GPIO14;
    let ts_scl = peripherals.GPIO13;
    let ts_irq = peripherals.GPIO1;
    spawner.spawn(handle_touch_events(
        ts_sda.into(),
        ts_scl.into(),
        ts_irq.into(),
        peripherals.IO_MUX,
        peripherals.I2C1.into(),
    ));

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

    loop {
        time += 1;
        let t = TOUCH_POINT.wait().await;
        let p1 = t.0.x as i32;
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

static IRQ: Mutex<RefCell<Option<Input>>> = Mutex::new(RefCell::new(None));
static TS_INT_CTRL: Signal<CriticalSectionRawMutex,bool> = Signal::new();
static TOUCH_POINT: Signal<CriticalSectionRawMutex,(Touch,Option<Touch>)> = Signal::new();

#[embassy_executor::task]
async fn handle_touch_events(
    ts_sda: AnyPin,
    ts_scl: AnyPin,
    ts_irq: AnyPin,
    io_mux: IO_MUX,
    i2c: AnyI2c,
) {
    let mut ts = TouchBreakoutCap::new(ts_sda.into(), ts_scl.into(), i2c);

    //create interrupt for ts
    let mut io = Io::new(io_mux);
    io.set_interrupt_handler(ts_handler);

    let mut irq_pin = Input::new(ts_irq, esp_hal::gpio::Pull::Up);

    critical_section::with(|cs| {
        irq_pin.listen(esp_hal::gpio::Event::FallingEdge);
        IRQ.borrow_ref_mut(cs).replace(irq_pin);
    });


    //use await to be awaken by interrupt?
    //distinguish type of touch and check if clicked a button in window

    loop {
        //wait for interrupt trigger
        TS_INT_CTRL.wait().await;
        let t = ts.read_points().unwrap();

        let s = match t.0.event_flag {
            TouchEventFlag::Contact => "Contact",
            TouchEventFlag::LiftUp => "LiftUp",
            TouchEventFlag::PressDown => "PressDown",
            TouchEventFlag::NoEvent => "NoEvent",
        };
        esp_println::println!("P1: \tx: {:4}\ty: {:4}\te: {}", t.0.x, t.0.y, s);
        TOUCH_POINT.signal(t);
        Timer::after_millis(5).await;
    }
}

#[handler]
#[ram]
fn ts_handler() {
    critical_section::with(|cs| {
        let mut button = IRQ.borrow_ref_mut(cs);
        let Some(button) = button.as_mut() else {
            // Some other interrupt has occurred
            // before the button was set up.
            return;
        };
        if button.is_interrupt_set() {
            TS_INT_CTRL.signal(true);
            //button.unlisten();
        }
    });
}
