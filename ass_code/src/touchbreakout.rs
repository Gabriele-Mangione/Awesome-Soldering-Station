extern crate alloc;
use alloc::boxed::Box;
use embedded_graphics::prelude::Point;

use esp_hal::{
    analog::adc::{Adc, AdcChannel, AdcConfig, AdcPin, RegisterAccess},
    gpio::{AnalogPin, AnyPin, GpioPin, Input, InputPin, Output, OutputPin},
    peripheral::Peripheral,
    peripherals::{ADC1, ADC2},
};

use core::{borrow::BorrowMut, char::from_digit, ptr::write_volatile};

//use crate::adc_monitor_link::*;

//todo type
const OUT_W1TS_ADDR: *mut u32 = 0x60004008 as *mut u32;
const OUT_W1TC_ADDR: *mut u32 = 0x6000400C as *mut u32;

pub struct abaabab<B>
where
    B: Peripheral<P: InputPin>,
{
    p: B,
}

impl<B> abaabab<B>
where
    B: Peripheral<P: InputPin>,
{
    pub fn new(pin: B) -> abaabab<B> {
        Self { p: pin }
    }

    pub fn foo(&mut self) {
        Input::new(self.p.borrow_mut(), esp_hal::gpio::Pull::None);
    }
}

pub trait Pinny: AnalogPin
        + Peripheral<P: InputPin>
        + Peripheral<P: OutputPin>
        + AdcChannel
        + InputPin
        + OutputPin
        + Into<AnyPin>{}


pub struct TouchBreakout<'a, A,B,C,D>
where
    A: AnalogPin
        + Peripheral<P: InputPin>
        + Peripheral<P: OutputPin>
        + AdcChannel
        + InputPin
        + OutputPin
        + Into<AnyPin>,
    B: AnalogPin
        + Peripheral<P: InputPin>
        + Peripheral<P: OutputPin>
        + AdcChannel
        + InputPin
        + OutputPin
        + Into<AnyPin>,
    C: AnalogPin
        + Peripheral<P: InputPin>
        + Peripheral<P: OutputPin>
        + AdcChannel
        + InputPin
        + OutputPin
        + Into<AnyPin>,
    D: AnalogPin
        + Peripheral<P: InputPin>
        + Peripheral<P: OutputPin>
        + AdcChannel
        + InputPin
        + OutputPin
        + Into<AnyPin>,
{
    xp_pin: A,
    yp_pin: B,
    xm_pin: C,
    ym_pin: D,
    x_max: i32,
    y_max: i32,
    adc1: &'a mut ADC1,
    adc2: &'a mut ADC2,
}


impl<'a, A,B,C,D> TouchBreakout<'a,A,B,C,D>
where
    A: AnalogPin
        + Peripheral<P: InputPin>
        + Peripheral<P: OutputPin>
        + AdcChannel
        + InputPin
        + OutputPin
        + Into<AnyPin>,
    B: AnalogPin
        + Peripheral<P: InputPin>
        + Peripheral<P: OutputPin>
        + AdcChannel
        + InputPin
        + OutputPin
        + Into<AnyPin>,
    C: AnalogPin
        + Peripheral<P: InputPin>
        + Peripheral<P: OutputPin>
        + AdcChannel
        + InputPin
        + OutputPin
        + Into<AnyPin>,
    D: AnalogPin
        + Peripheral<P: InputPin>
        + Peripheral<P: OutputPin>
        + AdcChannel
        + InputPin
        + OutputPin
        + Into<AnyPin>,
{
    pub fn new(
        xp_pin: A,
        yp_pin: B,
        xm_pin: C,
        ym_pin: D,
        x_max: i32,
        y_max: i32,
        adc1: &'a mut ADC1,
        adc2: &'a mut ADC2,
    ) -> TouchBreakout<'a, A,B,C,D> {
        TouchBreakout {
            xp_pin,
            yp_pin,
            xm_pin,
            ym_pin,
            x_max,
            y_max,
            adc1,
            adc2,
        }
    }

    /*
    pub fn get_point(&mut self) -> Point {
        let x = self.get_x();
        let y = self.get_y();

        Point::new(x, y)
    }
    pub fn get_point_and_pressure(&mut self) -> (Point, i32) {
        let x = self.get_x();
        let y = self.get_y();

        (Point::new(x, y), self.get_z())
    }
    */

    pub fn get_x(&mut self) -> i32 where <A as Peripheral>::P: AnalogPin + AdcChannel{
        Input::new(self.xp_pin.borrow_mut(), esp_hal::gpio::Pull::None);
        Output::new(self.yp_pin.borrow_mut(), esp_hal::gpio::Level::High);
        Output::new(self.ym_pin.borrow_mut(), esp_hal::gpio::Level::Low);

        let mut adc_config = AdcConfig::new();
        let mut xp_adc = adc_config.enable_pin(
            unsafe { 
                self.
                xp_pin.clone_unchecked() },
            esp_hal::analog::adc::Attenuation::_11dB,
        );
        let mut adc = Adc::new(&mut self.adc2, adc_config);

        let mut adc_val: i32 = 0;
        for i in 0..10 {
            let out = adc.read_oneshot(&mut xp_adc).unwrap();
            adc_val += out as i32;
        }
        adc_val * self.y_max / 40960

        /*
        uint16_t TouchPoint::getX() {
          pinMode(ypPin, INPUT);
          pinMode(ymPin, INPUT);
          pinMode(xpPin, OUTPUT);
          pinMode(xmPin, OUTPUT);
          digitalWrite(xpPin, HIGH);
          digitalWrite(xmPin, LOW);
          delayMicroseconds(50);
          uint16_t Xmeasured[NUMBER_OF_MEASUREMENTS] = { 0 };
          for (uint8_t i = 0; i < NUMBER_OF_MEASUREMENTS; i++) {
            Xmeasured[i] = analogRead(ypPin);
            delayMicroseconds(5);
          }


          if (max(Xmeasured) - min(Xmeasured) < MAX_SWING_ACCURACY_X) {
            return average(Xmeasured);
          }
          return 0;
        }
        */
    }

    pub fn get_y(&mut self) -> i32  where <B as Peripheral>::P: AnalogPin + AdcChannel{
        Input::new(self.yp_pin.borrow_mut(), esp_hal::gpio::Pull::None);
        Output::new(self.xp_pin.borrow_mut(), esp_hal::gpio::Level::High);
        Output::new(self.xm_pin.borrow_mut(), esp_hal::gpio::Level::Low);

    let mut adc_config = AdcConfig::new();
        let mut yp_adc = adc_config.enable_pin(
            unsafe { 
                self.
                yp_pin.clone_unchecked() },
            esp_hal::analog::adc::Attenuation::_11dB,
        );
    let mut adc = Adc::new(&mut self.adc1, adc_config);

        let mut adc_val: i32 = 0;
        for i in 0..10 {
            let out = adc.read_oneshot(&mut yp_adc).unwrap();
            adc_val += out as i32;
        }
        self.x_max - (adc_val * self.x_max / 40960)
    }

    pub fn get_z(&mut self) -> i32 where <B as Peripheral>::P: AnalogPin + AdcChannel{
        Input::new(self.yp_pin.borrow_mut(), esp_hal::gpio::Pull::None);
        Output::new(self.xp_pin.borrow_mut(), esp_hal::gpio::Level::High);
        Output::new(self.ym_pin.borrow_mut(), esp_hal::gpio::Level::Low);

    let mut adc_config = AdcConfig::new();
        let mut yp_adc = adc_config.enable_pin(
            unsafe { 
                self.
                yp_pin.clone_unchecked() },
            esp_hal::analog::adc::Attenuation::_11dB,
        );
    let mut adc = Adc::new(&mut self.adc1, adc_config);

        let mut adc_val: i32 = 0;
        for i in 0..10 {
            let out = adc.read_oneshot(&mut yp_adc).unwrap();
            adc_val += out as i32;
        }
        adc_val / 10
        //if Z1-Z2 is very high (whole voltage over Z, Pressure resistor) no touch is detected
        //Z1-Z2
        //
        //todo!("Pressure detection is yet to develop");
        //Ok((z_adc_val * self.y_max / 40960) as i32)
    }

    /*
    pub fn setup_touch_interrupt(&mut self) -> bool {
        true
        */
    /*
    unsafe {
        //gpio_reset_pin(self.ym_pin.pin());
        //gpio_reset_pin(self.yp_pin.pin());
        gpio_reset_pin(self.yp_pin.pin());
        gpio_reset_pin(self.xm_pin.pin());

        gpio_set_direction(self.xp_pin.pin(), gpio_mode_t_GPIO_MODE_OUTPUT);
        gpio_set_direction(self.ym_pin.pin(), gpio_mode_t_GPIO_MODE_OUTPUT);
        //gpio_set_direction(self.xp_pin.pin(), gpio_mode_t_GPIO_MODE_INPUT);
        //gpio_set_direction(self.xm_pin.pin(), gpio_mode_t_GPIO_MODE_INPUT);
        //
        //13, 1, 2, 14
        //xp, yp, xm, ym

        gpio_set_level(self.xp_pin.pin(), 1);
        gpio_set_level(self.ym_pin.pin(), 0);
    }

    let mut continuous_handle: adc_continuous_handle_t = std::ptr::null_mut();
    let adc_continuous_config: adc_continuous_handle_cfg_t = adc_continuous_handle_cfg_t {
        max_store_buf_size: 1024,
        conv_frame_size: 256,
    };

    unsafe {
        adc_continuous_new_handle(&adc_continuous_config, &mut continuous_handle);
    }

    /*
    let dig_cfg: adc_continuous_config_t = adc_continuous_config_t {
        sample_freq_hz: 20000,
        conv_mode: adc_digi_convert_mode_t_ADC_CONV_SINGLE_UNIT_1,
        format: adc_digi_output_format_t_ADC_DIGI_OUTPUT_FORMAT_TYPE1,
    }
    */

    let mut monitor_handle: adc_monitor_handle_t = std::ptr::null_mut();
    let monitor_config: adc_monitor_config_t = adc_monitor_config_t {
        adc_unit: adc_unit_t_ADC_UNIT_1,
        channel: self.yp_adc,
        h_threshold: 1000,
        l_threshold: -1,
    };
    let cb: adc_monitor_evt_cb_t = callback;

    let cbs_monitor: adc_monitor_evt_cbs_t = adc_monitor_evt_cbs_t {
        on_over_high_thresh: cb,
        on_below_low_thresh: cb,
    };

    unsafe {
        let mut res =
            adc_new_continuous_monitor(continuous_handle, &monitor_config, &monitor_handle);

        res |= adc_continuous_monitor_register_event_callbacks(
            monitor_handle,
            &cbs_monitor,
            std::ptr::null_mut(),
        );

        res |= adc_continuous_monitor_enable(monitor_handle);
    }
    */

    //todo!("Pressure detection is yet to develop");

    /*
        let init2_config: adc_oneshot_unit_init_cfg_t = adc_oneshot_unit_init_cfg_t {
            unit_id: adc_unit_t_ADC_UNIT_2,
            clk_src: 0,
            ulp_mode: 0,
        };
        let mut config: adc_oneshot_chan_cfg_t = adc_oneshot_chan_cfg_t {
            atten: adc_atten_t_ADC_ATTEN_DB_11,
            bitwidth: adc_bitwidth_t_ADC_BITWIDTH_DEFAULT,
        };
        let mut xm_adc_handle: adc_oneshot_unit_handle_t = std::ptr::null_mut();
        unsafe {
            adc_oneshot_new_unit(&init1_config, &mut xm_adc_handle);
            adc_oneshot_config_channel(yp_adc_handle, yp_adc, &mut config);
            gpio_reset_pin(ym_pin.pin());
        }

        *//*
    }
        */

    /*
    pub fn touch_detection_bool(&mut self) -> i32 {
        Input::new(self.yp_pin.borrow_mut(), esp_hal::gpio::Pull::None);
        Output::new(self.xp_pin.borrow_mut(), esp_hal::gpio::Level::High);
        Output::new(self.ym_pin.borrow_mut(), esp_hal::gpio::Level::Low);

    let mut adc_config = AdcConfig::new();
    let mut yp_adc = adc_config.enable_pin(self.yp_pin.into(), esp_hal::analog::adc::Attenuation::_11dB);
    let mut adc = Adc::new(&mut self.adc1, adc_config);

        adc.read_oneshot(&mut yp_adc).unwrap() as i32
        //return Ok(gpio_get_level(self.yp_pin.pin()));
    }
    pub fn touch_detection(&mut self) -> i32 {
        Input::new(self.yp_pin.borrow_mut(), esp_hal::gpio::Pull::None);
        Output::new(self.xp_pin.borrow_mut(), esp_hal::gpio::Level::High);
        Output::new(self.ym_pin.borrow_mut(), esp_hal::gpio::Level::Low);

    let mut adc_config = AdcConfig::new();
    let mut yp_adc = adc_config.enable_pin(self.yp_pin, esp_hal::analog::adc::Attenuation::_11dB);
    let mut adc = Adc::new(&mut self.adc1, adc_config);


        let mut y_adc_val: i32 = 0;
        for i in 0..10 {
            //adc_val += adc1_get_raw(self.y_adc) as u32;
            let out = adc.read_oneshot(&mut yp_adc).unwrap();
            y_adc_val += out as i32;
        }
        y_adc_val / 10
    }
    */
}
/*
impl Future for TouchBreakout<'a> {
    type Output = ();

    fn poll(self: std::pin::Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.get_point().unwrap().is_some() {
            return Poll::Ready(());
        }
        Poll::Pending
    }
}
*/
