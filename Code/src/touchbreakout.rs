use std::{
    future::Future,
    task::{Context, Poll},
};

use embedded_graphics::prelude::Point;
use esp_idf_hal::{
    adc::AdcChannelDriver,
    gpio::{
        self, ADCPin, AnyIOPin, InputOutput, InputPin, OutputPin, Pin, PinDriver, Pins,
        RtcInputOutput,
    },
    peripheral::{Peripheral, PeripheralRef},
    sys::EspError,
    sys::*,
};

use core::ptr::write_volatile;

//todo type
const OUT_W1TS_ADDR: *mut u32 = 0x60004008 as *mut u32;
const OUT_W1TC_ADDR: *mut u32 = 0x6000400C as *mut u32;

pub struct TouchBreakout {
    xp_pin: AnyIOPin,
    yp_pin: AnyIOPin,
    xm_pin: AnyIOPin,
    ym_pin: AnyIOPin,
    x_adc: adc_channel_t,
    y_adc: adc_channel_t,
    x_max: u32,
    y_max: u32,
    adc_handle: adc_oneshot_unit_handle_t
}

impl TouchBreakout {
    pub fn new(
        xp_pin: AnyIOPin,
        yp_pin: AnyIOPin,
        xm_pin: AnyIOPin,
        ym_pin: AnyIOPin,
        x_adc: adc_channel_t,
        y_adc: adc_channel_t,
        x_max: u32,
        y_max: u32,
    ) -> TouchBreakout {
        /*

        let mut adc_handle_cfg: adc_continuous_handle_cfg_t;
        adc_handle_cfg.max_store_buf_size = 10;
        let mut adc_handle: adc_continuous_handle_t;
        */
        let init_config: adc_oneshot_unit_init_cfg_t = adc_oneshot_unit_init_cfg_t {
            unit_id: adc_unit_t_ADC_UNIT_1,
            clk_src: 0,
            ulp_mode: 0,
        };

        let mut config: adc_oneshot_chan_cfg_t = adc_oneshot_chan_cfg_t {
            atten: adc_atten_t_ADC_ATTEN_DB_11,
            bitwidth: adc_bitwidth_t_ADC_BITWIDTH_DEFAULT,
        };

        let mut adc_handle: adc_oneshot_unit_handle_t = std::ptr::null_mut();

        unsafe {
            adc_oneshot_new_unit(&init_config, &mut adc_handle);
            adc_oneshot_config_channel(adc_handle, x_adc, &mut config);
            adc_oneshot_config_channel(adc_handle, y_adc, &mut config);
            //adc1_config_width(12);
            gpio_reset_pin(xp_pin.pin());
            gpio_reset_pin(yp_pin.pin());
            gpio_reset_pin(xm_pin.pin());
            gpio_reset_pin(ym_pin.pin());
        }

        TouchBreakout {
            xp_pin,
            yp_pin,
            xm_pin,
            ym_pin,
            x_adc,
            y_adc,
            x_max,
            y_max,
            adc_handle
        }
    }

    pub fn get_point(&mut self) -> Result<Option<Point>, EspError> {
        let x = self.get_x()?;
        let y = self.get_y()?;

        Ok(Some(Point::new(x, y)))
    }

    pub fn get_x(&mut self) -> Result<i32, EspError> {
        unsafe {
            /*
            gpio_reset_pin(self.ym_pin.pin());
            gpio_reset_pin(self.yp_pin.pin());
            */
            gpio_reset_pin(self.xp_pin.pin());
            gpio_reset_pin(self.xm_pin.pin());

            gpio_set_direction(self.yp_pin.pin(), gpio_mode_t_GPIO_MODE_OUTPUT);
            gpio_set_direction(self.ym_pin.pin(), gpio_mode_t_GPIO_MODE_OUTPUT);
            //gpio_set_direction(self.xp_pin.pin(), gpio_mode_t_GPIO_MODE_INPUT);
            //gpio_set_direction(self.xm_pin.pin(), gpio_mode_t_GPIO_MODE_INPUT);
            //
            //13, 1, 2, 14 
            //xp, yp, xm, ym

            /*
            gpio_set_level(self.yp_pin.pin(), 1);
            gpio_set_level(self.ym_pin.pin(), 0);
            */

            gpio_set_level(self.xp_pin.pin(), 0);

            //adc1_config_channel_atten(self.y_adc, adc_atten_t_ADC_ATTEN_DB_11);

            let mut adc_val: u32 = 0;
            for i in 0..10 {
                //adc_val += adc1_get_raw(self.y_adc) as u32;
                let mut out: i32 = 0;
                unsafe {
                    adc_oneshot_read(self.adc_handle, self.x_adc, std::ptr::from_mut(&mut out));
                }
                adc_val += out as u32;
            }
            Ok((adc_val / (10 /* * self.x_max*/)) as i32)
        }

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

    pub fn get_y(&mut self) -> Result<i32, EspError> {
        unsafe {
            /*
            gpio_reset_pin(self.xp_pin.pin());
            gpio_reset_pin(self.xm_pin.pin());
            */
            gpio_reset_pin(self.yp_pin.pin());
            gpio_reset_pin(self.ym_pin.pin());

            gpio_set_direction(self.xp_pin.pin(), gpio_mode_t_GPIO_MODE_OUTPUT);
            gpio_set_direction(self.xp_pin.pin(), gpio_mode_t_GPIO_MODE_OUTPUT);
            /*
            gpio_set_direction(self.yp_pin.pin(), gpio_mode_t_GPIO_MODE_INPUT);
            gpio_set_direction(self.ym_pin.pin(), gpio_mode_t_GPIO_MODE_INPUT);
            */

            /*
            gpio_set_level(self.xp_pin.pin(), 1);
            gpio_set_level(self.xm_pin.pin(), 0);
            */

            //adc1_config_channel_atten(self.x_adc, adc_atten_t_ADC_ATTEN_DB_11);

            let mut adc_val: u32 = 0;
            for i in 0..10 {
                //adc_val += adc1_get_raw(self.x_adc) as u32;
                let mut out: i32 = 0;
                unsafe {
                    adc_oneshot_read(self.adc_handle, self.y_adc, std::ptr::from_mut(&mut out));
                }
                adc_val += out as u32;
            }
            Ok((adc_val / (10 /* * self.y_max*/)) as i32)
        }
    }
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
