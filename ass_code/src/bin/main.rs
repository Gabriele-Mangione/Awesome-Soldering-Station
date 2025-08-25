#![no_std]
#![no_main]

extern crate alloc;
use core::cell::RefCell;

use alloc::borrow::ToOwned;
use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::{format, vec};
use critical_section::Mutex;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex};
use embassy_sync::channel::Channel;
use embassy_sync::signal::Signal;
use embassy_time::Timer;
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::prelude::{Point, Size};
use embedded_graphics::primitives::{Line, PrimitiveStyle};
use embedded_graphics::primitives::{Circle, Primitive, Rectangle};
use embedded_graphics::text::renderer::CharacterStyle;
use embedded_graphics::text::Text;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{AnyPin, Input, Io, Level, Output};
use esp_hal::i2c::master::AnyI2c;
use esp_hal::interrupt::InterruptConfigurable;
use esp_hal::peripherals::IO_MUX;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::{handler, ram};
use esp_storage::FlashStorage;

use ass_code::{ili9341, MyColor};
use ass_code::soldering::{Soldering, TempData};
use ass_code::touchbreakout_cap::{Touch, TouchBreakoutCap, TouchEventFlag};
use ass_code::{fusb302, soldering};
use embedded_graphics::{self, Drawable, Pixel};
use ringbuffer::{AllocRingBuffer, ConstGenericRingBuffer, RingBuffer};

//static mut APP_CORE_STACK: Stack<8192> = Stack::new();

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // generator version: 0.2.2
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    esp_println::logger::init_logger_from_env();

    esp_alloc::heap_allocator!(72 * 1024);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    // init fusb, select 9 volt
    let pdo_vec: Vec<fusb302::PDO>;

    let mut pdo_requested = false;
    log::info!("init fusb!");
    let mut fusb = fusb302::Fusb::new(
        peripherals.GPIO5.into(),
        peripherals.GPIO4.into(),
        peripherals.I2C0,
        0x22,
    );
    log::info!("scan pds!");
    pdo_vec = fusb.scan_pds().await.unwrap();
    log::info!("request pdo!");
    if pdo_vec.len() > 0 {
        let found_pdo = pdo_vec.iter().find(|&&x| x.voltage == 9000);
        if found_pdo.is_some() {
            fusb.request_pdo(*found_pdo.unwrap(), 3000, 3000).unwrap();
            pdo_requested = true;
        }
    }
    log::info!("done");

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
    let mut screen = ili9341::ILI9341::new();

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
    /*
    let rx_gyro = Input::new(peripherals.GPIO12, esp_hal::gpio::Pull::Down);
    let mut red_style = MonoTextStyle::new(&FONT_10X20, ass_code::MyColor(255, 0));
    red_style.set_background_color(Some(ass_code::MyColor(0, 0)));

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

    static mut CHAN: Channel<NoopRawMutex, TempData, 3> =
        Channel::<NoopRawMutex, TempData, 3>::new();

    let flash = FlashStorage::new();
    let soldering = Soldering::new(
        peripherals.GPIO9.into(),
        peripherals.GPIO8,
        peripherals.ADC1,
        peripherals.LEDC,
        flash,
        unsafe { CHAN.sender() },
    );

    spawner.spawn(soldering.task()).unwrap();

    let ts_sda = peripherals.GPIO14;
    let ts_scl = peripherals.GPIO13;
    let ts_irq = peripherals.GPIO1;
    spawner
        .spawn(handle_touch_events(
            ts_sda.into(),
            ts_scl.into(),
            ts_irq.into(),
            peripherals.IO_MUX,
            peripherals.I2C1.into(),
        ))
        .unwrap();

    //touch circles
    //let mut balls: Vec<Circle> = vec![];
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

    let mut rb = ConstGenericRingBuffer::<Circle, 10>::new();
    let mut diagram_data = ConstGenericRingBuffer::<(u8,u8,u8,u8), 276>::new();

            Line::new(Point::new(20,20), Point::new(20,220)).into_styled(PrimitiveStyle::with_stroke(ass_code::MyColor(0xFF, 0xFF), 2)).draw(&mut screen).unwrap();
            Line::new(Point::new(20,220), Point::new(300,220)).into_styled(PrimitiveStyle::with_stroke(ass_code::MyColor(0xFF, 0xFF), 2)).draw(&mut screen).unwrap();
    loop {
        time += 1;
        if TOUCH_POINT.signaled() {
            let t = TOUCH_POINT.wait().await;
            let p1 = t.0.x as i32;
            let p2 = 320i32 - t.0.y as i32;
            if p1 != 0 {
                rb.enqueue(Circle::new(Point::new(p2 - 5, p1 - 5), 5u32));
                let mut rbi = rb.clone().into_iter();
                for i in (0..rb.len()).rev() {
                    rbi.nth(0)
                        .unwrap()
                        .into_styled(styles[i])
                        .draw(&mut screen)
                        .unwrap();
                }
                /*
                balls.insert(0, ball);
                if balls.len() > 10 {
                    balls.pop();
                }
                for i in (0..balls.len()).rev() {
                    balls.get(i).unwrap().into_styled(styles[i]).draw(&mut screen).unwrap();
                }
                    */
            }
        }

        /*
        let str = "This is a text ".to_owned() + &time.to_string();
        Text::new(&str, Point::new(50, 50), style)
            .draw(&mut screen)
            .unwrap();
        */

        let temp_data: TempData = unsafe { CHAN.receive().await };
        let temp_str = format!(
            "t: {:6.2}, p: {:6.2},\ni: {:6.2}, d: {:6.2}",
            temp_data.temp, temp_data.temp_p, temp_data.temp_i, temp_data.temp_d
        );

        diagram_data.enqueue((
                (temp_data.temp * 200./600.) as u8,
                (temp_data.temp_p * 200./600.) as u8,
                (temp_data.temp_i * 200./600.) as u8,
                (temp_data.temp_d * 200./600.) as u8));

    Rectangle::new(Point::new(22, 20), Size::new(diagram_data.len() as _, 200))
        .into_styled(PrimitiveStyle::with_fill(ass_code::MyColor(0, 0)))
        .draw(&mut screen)
        .unwrap();

        let mut data_clone = diagram_data.clone().into_iter();
        for i in 0..diagram_data.len() {
            let v = data_clone.nth(0).unwrap();
        //data_clone.iter().map(|v| {
            //draw black line at x for every y
            //Line::new(Point::new(i as i32 +21,0), Point::new(i as i32 +20,240)).into_styled(PrimitiveStyle::with_stroke(ass_code::MyColor(0, 0), 1)).draw(&mut screen).unwrap();
            //draw the 4 values respectively in a scale (ie 0° to 600°)
            Pixel(Point::new(i as i32 +22, (218. - temp_data.set * 200./600. ) as _), MyColor::from_rgb(0x0F, 0x1F, 0x0F)).draw(&mut screen).unwrap();
            Pixel(Point::new(i as i32 +22, (218 -v.0)as _), MyColor::from_rgb(0x1F, 0x3F, 0x1F)).draw(&mut screen).unwrap();
            Pixel(Point::new(i as i32 +22, (218 -v.1) as _), MyColor::from_rgb(0x1F, 0, 0)).draw(&mut screen).unwrap();
            Pixel(Point::new(i as i32 +22, (218 -v.2) as _), MyColor::from_rgb(0, 0x3F, 0)).draw(&mut screen).unwrap();
            Pixel(Point::new(i as i32 +22, (218 -v.3) as _), MyColor::from_rgb(0, 0, 0x1F)).draw(&mut screen).unwrap();

        };

        //draw diagram data at an x coordinate

        Timer::after_millis(5).await;
    }
}

static IRQ: Mutex<RefCell<Option<Input>>> = Mutex::new(RefCell::new(None));
static TS_INT_CTRL: Signal<CriticalSectionRawMutex, bool> = Signal::new();
static TOUCH_POINT: Signal<CriticalSectionRawMutex, (Touch, Option<Touch>)> = Signal::new();

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
    /*
    let mut io = Io::new(io_mux);
    io.set_interrupt_handler(ts_handler);
    */

    let mut irq_pin = Input::new(ts_irq, esp_hal::gpio::Pull::Up);

    /*
    critical_section::with(|cs| {
        irq_pin.listen(esp_hal::gpio::Event::FallingEdge);
        IRQ.borrow_ref_mut(cs).replace(irq_pin);
    });
    */

    //use await to be awaken by interrupt?
    //distinguish type of touch and check if clicked a button in window

    loop {
        //wait for interrupt trigger
        //TS_INT_CTRL.wait().await;
        irq_pin.wait_for_falling_edge().await;
        let t = ts.read_points().unwrap();

        let s = match t.0.event_flag {
            TouchEventFlag::Contact => "Contact",
            TouchEventFlag::LiftUp => "LiftUp",
            TouchEventFlag::PressDown => "PressDown",
            TouchEventFlag::NoEvent => "NoEvent",
        };
        esp_println::println!("P1: \tx: {:4}\ty: {:4}\te: {}", t.0.x, t.0.y, s);
        TOUCH_POINT.signal(t);
    }
}
/*

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
*/
