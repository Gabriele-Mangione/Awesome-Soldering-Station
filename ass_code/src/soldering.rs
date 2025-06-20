use esp_hal::{analog::adc::{self, Adc, AdcChannel, AdcConfig, AdcPin}, gpio::AnalogPin, ledc::{channel::{Channel, ChannelHW}, LowSpeed}, peripheral::Peripheral};
use esp_storage::FlashStorage;
use embedded_storage::{ReadStorage, Storage};
use log::info;

struct Soldering<'a,PIN, ADCI>
where
    ADCI: adc::RegisterAccess + Peripheral,
    PIN: AdcChannel + AnalogPin,
{
    tmp_pin: AdcPin<PIN, ADCI>,
    adc: Adc<'a,ADCI>,

    flash: FlashStorage
}

impl<PIN, ADCI> Soldering<'_,PIN, ADCI>
where
    //ADCI: Peripheral,
    ADCI: adc::RegisterAccess + Peripheral<P=ADCI>,
    PIN: AdcChannel + AnalogPin,
{
    fn new(tmp_pin: PIN, adc: ADCI, flash: FlashStorage) -> Self {
        let mut adc_config = AdcConfig::new();
        let mut tmp_pin = adc_config.enable_pin(tmp_pin, esp_hal::analog::adc::Attenuation::_11dB);
        let mut adc = Adc::new(adc, adc_config);
        Self { tmp_pin, adc, flash }
    }
}

impl<PIN, ADCI> Soldering<'_,PIN, ADCI>
where
    ADCI: adc::RegisterAccess + Peripheral,
    PIN: AdcChannel + AnalogPin,
{
    fn read_temp(&mut self) -> u16 {
        self.adc.read_blocking(&mut self.tmp_pin)
    }

    pub fn task(&mut self) -> embassy_executor::SpawnToken<impl Sized> {
        solder_task(self)
    }
}


#[embassy_executor::task]
async fn solder_task<'a,PIN, ADCI>( s:&mut Soldering<'a,PIN,ADCI>)
where
    //ADCI: Peripheral,
    ADCI: adc::RegisterAccess + Peripheral<P=ADCI>,
    PIN: AdcChannel + AnalogPin,
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
        s.solder_pin.set_duty_hw(0);
        //read adc temperature pin
        let mut out: u32 = 0;
        for _ in 0..10 {
            out += s.adc.read_blocking(&mut s.temp_pin) as u32;
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

            s.solder_pin.set_duty_hw(duty_cycle as u32);

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
