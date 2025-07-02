use esp_hal::{analog::adc::{self, Adc, AdcChannel, AdcConfig, AdcPin}, gpio::{AnalogPin, AnyPin, OutputPin}, ledc::{channel::{self, Channel, ChannelHW, ChannelIFace}, timer::{self, Number, TimerIFace}, LSGlobalClkSource, Ledc, LowSpeed}, peripheral::Peripheral, peripherals::{ADC1, LEDC}};
use esp_storage::FlashStorage;
use embedded_storage::{ReadStorage, Storage};
use log::info;
use esp_hal::time::RateExtU32;

pub struct Soldering<ADCI>
where
    ADCI: adc::RegisterAccess + Peripheral,
{
    solder_pin: AnyPin,
    tmp_pin: AnyPin,
    adc_peripheral:ADCI,
    ledc_peripheral: LEDC,

    flash: FlashStorage
}

impl<ADCI> Soldering<ADCI>
where
    //ADCI: Peripheral,
    ADCI: adc::RegisterAccess + Peripheral<P=ADCI>,
{
    pub fn new(solder_pin:AnyPin, tmp_pin: AnyPin, adc_peripheral: ADCI, ledc_peripheral: LEDC, flash: FlashStorage) -> Self {
        Self { solder_pin, tmp_pin, adc_peripheral,ledc_peripheral, flash }
    }
}

impl Soldering<ADC1>
{

    pub fn task(self) -> embassy_executor::SpawnToken<impl Sized> {
        solder_task(self)
    }
}


#[embassy_executor::task]
async fn solder_task( s:Soldering<ADC1>)
{
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
            out += adc.read_blocking(&mut tmp_pin) as u32;
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
