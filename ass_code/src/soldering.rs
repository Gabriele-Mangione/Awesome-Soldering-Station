use core::cell::RefCell;
use core::fmt::Write;

use critical_section::Mutex;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::{Channel, Sender};
use embassy_time::Timer;
use embedded_io::{Read, ReadReady};
use embedded_storage::{ReadStorage, Storage};
use esp_hal::gpio::GpioPin;
use esp_hal::interrupt::InterruptConfigurable;
use esp_hal::time::RateExtU32;
use esp_hal::uart::{self, AnyUart, AtCmdConfig, Uart, UartInterrupt};
use esp_hal::{
    analog::adc::{self, Adc, AdcConfig},
    gpio::AnyPin,
    ledc::{
        channel::{self, ChannelHW, ChannelIFace},
        timer::{self, Number, TimerIFace},
        LSGlobalClkSource, Ledc, LowSpeed,
    },
    peripheral::Peripheral,
    peripherals::{ADC1, LEDC},
};
use esp_storage::FlashStorage;
use log::{debug, info, trace, warn};
use ringbuffer::{AllocRingBuffer, ConstGenericRingBuffer, RingBuffer};

pub struct Soldering<ADCI>
where
    ADCI: adc::RegisterAccess + Peripheral,
{
    to_uc_n: AnyPin,
    to_uc_p: AnyPin,
    solder_pin: AnyPin,
    tmp_pin: GpioPin<8>,
    adc_peripheral: ADCI,
    ledc_peripheral: LEDC,
    uart_peripheral: AnyUart,

    flash: FlashStorage,

    sender: Sender<'static, NoopRawMutex, TempData, 3>,
}

#[derive(Clone)]
pub struct TempData {
    pub set: f32,
    pub temp: f32,
    pub temp_p: f32,
    pub temp_i: f32,
    pub temp_d: f32,
}

impl<ADCI> Soldering<ADCI>
where
    //ADCI: Peripheral,
    ADCI: adc::RegisterAccess + Peripheral<P = ADCI>,
{
    pub fn new(
        to_uc_n: AnyPin,
        to_uc_p: AnyPin,
        solder_pin: AnyPin,
        tmp_pin: GpioPin<8>,
        uart_peripheral: AnyUart,
        adc_peripheral: ADCI,
        ledc_peripheral: LEDC,
        flash: FlashStorage,
        sender: Sender<'static, NoopRawMutex, TempData, 3>,
    ) -> Self {
        Self {
            to_uc_n,
            to_uc_p,
            solder_pin,
            tmp_pin,
            uart_peripheral,
            adc_peripheral,
            ledc_peripheral,
            flash,
            sender,
        }
    }
}

impl Soldering<ADC1> {
    pub fn task(self) -> embassy_executor::SpawnToken<impl Sized> {
        solder_task(self)
    }
}


#[embassy_executor::task]
async fn solder_task(s: Soldering<ADC1>) {
    let mut ledc = Ledc::new(s.ledc_peripheral);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);
    let mut ledc_timer = ledc.timer::<LowSpeed>(Number::Timer0);
    ledc_timer
        .configure(timer::config::Config {
            duty: timer::config::Duty::Duty14Bit, //0- 16384
            clock_source: timer::LSClockSource::APBClk,
            frequency: RateExtU32::Hz(500),
        })
        .unwrap();

    let mut solder_pin = ledc.channel(channel::Number::Channel0, s.solder_pin);
    solder_pin
        .configure(channel::config::Config {
            timer: &ledc_timer,
            duty_pct: 0,
            pin_config: channel::config::PinConfig::PushPull,
        })
        .unwrap();

    let mut adc_config = AdcConfig::new();
    let mut tmp_pin = adc_config.enable_pin(s.tmp_pin, esp_hal::analog::adc::Attenuation::_11dB);
    let mut adc = Adc::new(s.adc_peripheral, adc_config);

    adc.read_blocking(&mut tmp_pin);


    //try to communicate with handle
    //read response
    //set cable_connected flag

    let mut cable_connected: bool = comm::pulse_check();
    /*
        if buf[0] == b'y' {
            //confirmed connection
            cable_connected = true;
        }
        */

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
    let mut bytes = [0u8; 4];
    let mut flash = FlashStorage::new();

    flash.read(0x9000, &mut bytes).unwrap();

    //if flash has never been set, ig new device
    if bytes[1] == 0 && bytes[0] == 0 {
        // == 0xff?
        //todo change these values to standard ones
        flash.write(0x9000, &[0x02, 0xC2, 0x0B, 0x08]).unwrap(); //706 at 100° & 2824 at 400°
        flash.read(0x9000, &mut bytes).unwrap();
    }

    //todo convert 2bytes to u16...
    let p_at_100c: u16 = ((bytes[0] as u16) << 8) | bytes[1] as u16;
    let p_at_400c: u16 = ((bytes[2] as u16) << 8) | bytes[3] as u16;
    info!("p @ 100°C: {:5}, p @ 400°C: {:5}", p_at_100c, p_at_400c);

    let mut adc_ring = ConstGenericRingBuffer::<u16, 64>::new();
    //let mut act_temp:f32 = 0.;

    loop {
        if cable_connected == false {
            u.write_char('?').expect("uart write fail");
            //info!("sending pulse check");

            if u.read_ready().unwrap() {
                //info!("response acquired");
                u.read_bytes(&mut buf); //is this blocking???
            }
            if buf[0] != b'y' {
                //info!("response was not 'y'");
                Timer::after_millis(1000).await;
                continue;
            }
            //confirmed connection
            cable_connected = true;
            //info!("response was 'y'");

            //write gyro settings
            let mut gyro_settings = [0u8; 2];
            flash.read(0x9010, &mut gyro_settings).unwrap();
            u.write_bytes(&gyro_settings)
                .expect("uart write gyro settings fail");
        }
        Timer::after_millis(1).await;
        //turn off voltage for measurement
        solder_pin.set_duty_hw(0);
        //wait for voltage stabilisation
        Timer::after_micros(100).await;

        //read adc temperature pin with rb
        for _ in 0..10 {
            adc_ring.enqueue(adc.read_blocking(&mut tmp_pin));
        }
        let avg_adc_val: u32 = adc_ring.iter().map(|&x| x as u32).sum();
        let avg_adc_val: u16 = (avg_adc_val >> 6) as u16; //shifting instead of dividing to
                                                          //optimise speed

        //todo: check if soldering handle is connected f.i. via reading data lines

        if avg_adc_val > 4000 {
            //no soldering tip is connected
            int_diff = 0.;
            old_diff = 0.;
            continue;
        }

        //convert adc value to celcius
        let act_temp: f32 =
            300. / (p_at_400c - p_at_100c) as f32 * (avg_adc_val - p_at_100c) as f32 + 100.;

        //(PID)
        let diff: f32 = set_temp as f32 - act_temp;
        //proportional
        let pro_diff = diff * kp;
        //integral
        int_diff += diff * ki;
        //derivative
        let der_diff = (diff - old_diff) * kd;
        old_diff = diff;

        //adjust output duty cycle
        let duty_cycle: u16 = ((pro_diff + int_diff + der_diff) as u16).clamp(0, 16384);
        //solder_pin.set_duty_hw(duty_cycle as u32);

        /*
        info!(
            "act_temp: {}, set_temp: {}, duty_cycle: {}",
            act_temp, set_temp, duty_cycle
        );
        info!("pro: {}, int: {}, der: {}", pro_diff, int_diff, der_diff);
        */

        //simulation
        //act_temp += 0.02 * duty_cycle as f32;
        //act_temp -= act_temp / 50.;

        //using try_send, so that the thread isn't blocked when buffer gets full
        if s.sender
            .try_send(TempData {
                set: set_temp as _,
                temp: act_temp,
                temp_p: pro_diff,
                temp_i: int_diff,
                temp_d: der_diff,
            })
            .is_err()
        {
            //warn!("sent temperatures error: channel buffer is full");
        }
    }
}

/*
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
*/
