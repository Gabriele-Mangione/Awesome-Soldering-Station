#![no_std]
#![no_main]

use alloc::borrow::ToOwned;
use alloc::string::ToString;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::prelude::{Size, Point};
use embedded_graphics::primitives::{Rectangle, Primitive};
use embedded_graphics::primitives::PrimitiveStyle;
use embedded_graphics::text::Text;
use embedded_graphics::text::renderer::CharacterStyle;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Output, Io, Level};
use esp_hal::main;
use esp_hal::timer::timg::TimerGroup;
use log::info;

use ass_code::ili9341;
use embedded_graphics::{self, Drawable};

extern crate alloc;

#[main]
fn main() -> ! {
    // generator version: 0.2.2

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_println::logger::init_logger_from_env();

    esp_alloc::heap_allocator!(72 * 1024);
    /*

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let _init = esp_wifi::init(
        timg0.timer0,
        esp_hal::rng::Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
    )
    .unwrap();
    */

    let io = Io::new(peripherals.IO_MUX);
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


    let mut screen = ass_code::ili9341::ILI9341::new();
    let mut style = MonoTextStyle::new(&FONT_10X20, ass_code::MyColor(255, 255));
    style.set_background_color(Some(ass_code::MyColor(0, 0)));

    //draw black screen
    Rectangle::new(Point::new(0, 0), Size::new(320, 240))
        .into_styled(PrimitiveStyle::with_fill(ass_code::MyColor(0, 0)))
        .draw(&mut screen);

    //Rectangle::new(Point::new(0, 0), Size::new(320, 240));
    
    Text::new("This is a text", Point::new(50, 50), style).draw(&mut screen);

    let delay = Delay::new();
    let mut time = 0;
    loop {
        time += 1;
        
        let mut str = "This is a text ".to_owned() + &time.to_string();
        Text::new(&str, Point::new(50, 50), style).draw(&mut screen);
        info!("Hello world!");
        delay.delay_millis(500);
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/v0.23.1/examples/src/bin
}
