use std::{
    ffi::c_void,
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
use esp_idf_svc;

use core::ptr::write_volatile;

use crate::adc_monitor_link::*;

//todo type
const OUT_W1TS_ADDR: *mut u32 = 0x60004008 as *mut u32;
const OUT_W1TC_ADDR: *mut u32 = 0x6000400C as *mut u32;

pub struct TouchBreakout {
    xp_pin: AnyIOPin,
    yp_pin: AnyIOPin,
    xm_pin: AnyIOPin,
    ym_pin: AnyIOPin,
    xp_adc: adc_channel_t,
    yp_adc: adc_channel_t,
    xm_adc: adc_channel_t,
    x_max: i32,
    y_max: i32,
    xp_adc_handle: adc_oneshot_unit_handle_t,
    yp_adc_handle: adc_oneshot_unit_handle_t,
    xm_adc_handle: adc_oneshot_unit_handle_t,
}

pub fn callback(
    a: adc_monitor_handle_t,
    b: *const adc_monitor_evt_data_t,
    c: *const c_void,
) -> bool {
    true
}

impl TouchBreakout {
    pub fn new(
        xp_pin: AnyIOPin,
        yp_pin: AnyIOPin,
        xm_pin: AnyIOPin,
        ym_pin: AnyIOPin,
        xp_adc: adc_channel_t,
        yp_adc: adc_channel_t,
        xm_adc: adc_channel_t,
        x_max: i32,
        y_max: i32,
    ) -> TouchBreakout {
        /*

        let mut adc_handle_cfg: adc_continuous_handle_cfg_t;
        adc_handle_cfg.max_store_buf_size = 10;
        let mut adc_handle: adc_continuous_handle_t;
        */
        let init1_config: adc_oneshot_unit_init_cfg_t = adc_oneshot_unit_init_cfg_t {
            unit_id: adc_unit_t_ADC_UNIT_1,
            clk_src: 0,
            ulp_mode: 0,
        };
        let init2_config: adc_oneshot_unit_init_cfg_t = adc_oneshot_unit_init_cfg_t {
            unit_id: adc_unit_t_ADC_UNIT_2,
            clk_src: 0,
            ulp_mode: 0,
        };

        let mut config: adc_oneshot_chan_cfg_t = adc_oneshot_chan_cfg_t {
            atten: adc_atten_t_ADC_ATTEN_DB_11,
            bitwidth: adc_bitwidth_t_ADC_BITWIDTH_DEFAULT,
        };

        let mut xp_adc_handle: adc_oneshot_unit_handle_t = std::ptr::null_mut();
        let mut yp_adc_handle: adc_oneshot_unit_handle_t = std::ptr::null_mut();
        let mut xm_adc_handle: adc_oneshot_unit_handle_t = std::ptr::null_mut();

        unsafe {
            adc_oneshot_new_unit(&init1_config, &mut yp_adc_handle);
            adc_oneshot_new_unit(&init2_config, &mut xp_adc_handle);
            adc_oneshot_new_unit(&init1_config, &mut xm_adc_handle);
            adc_oneshot_config_channel(xp_adc_handle, xp_adc, &mut config);
            adc_oneshot_config_channel(xm_adc_handle, xm_adc, &mut config);
            adc_oneshot_config_channel(yp_adc_handle, yp_adc, &mut config);
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
            xp_adc,
            xm_adc,
            yp_adc,
            x_max,
            y_max,
            xp_adc_handle,
            xm_adc_handle,
            yp_adc_handle,
        }
    }

    pub fn get_point(&mut self) -> Result<Point, EspError> {
        let x = self.get_x()?;
        let y = self.get_y()?;

        Ok(Point::new(x, y))
    }
    pub fn get_point_and_pressure(&mut self) -> Result<(Point, i32), EspError> {
        let x = self.get_x()?;
        let y = self.get_y()?;

        Ok((Point::new(x, y), self.get_z()?))
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

            gpio_set_level(self.yp_pin.pin(), 1);
            gpio_set_level(self.ym_pin.pin(), 0);

            //adc1_config_channel_atten(self.y_adc, adc_atten_t_ADC_ATTEN_DB_11);

            let mut adc_val: i32 = 0;
            for i in 0..10 {
                //adc_val += adc1_get_raw(self.y_adc) as u32;
                let mut out: i32 = 0;
                unsafe {
                    adc_oneshot_read(
                        self.xp_adc_handle,
                        self.xp_adc,
                        std::ptr::from_mut(&mut out),
                    );
                }
                adc_val += out;
            }
            gpio_set_level(self.ym_pin.pin(), 1);
            Ok(adc_val * self.y_max / 40960)
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
            gpio_set_direction(self.xm_pin.pin(), gpio_mode_t_GPIO_MODE_OUTPUT);
            /*
            gpio_set_direction(self.yp_pin.pin(), gpio_mode_t_GPIO_MODE_INPUT);
            gpio_set_direction(self.ym_pin.pin(), gpio_mode_t_GPIO_MODE_INPUT);
            */

            gpio_set_level(self.xp_pin.pin(), 1);
            gpio_set_level(self.xm_pin.pin(), 0);

            //adc1_config_channel_atten(self.x_adc, adc_atten_t_ADC_ATTEN_DB_11);

            let mut adc_val: i32 = 0;
            for i in 0..10 {
                //adc_val += adc1_get_raw(self.x_adc) as u32;
                let mut out: i32 = 0;
                unsafe {
                    adc_oneshot_read(
                        self.yp_adc_handle,
                        self.yp_adc,
                        std::ptr::from_mut(&mut out),
                    );
                }
                adc_val += out;
            }
            gpio_set_level(self.xm_pin.pin(), 1);
            Ok(self.x_max - (adc_val * self.x_max / 40960))
        }
    }

    pub fn get_z(&mut self) -> Result<i32, EspError> {
        unsafe {
            /*
            gpio_reset_pin(self.ym_pin.pin());
            gpio_reset_pin(self.yp_pin.pin());
            */
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

            //adc1_config_channel_atten(self.y_adc, adc_atten_t_ADC_ATTEN_DB_11);

            let mut z1_adc_val: i32 = 0;
            for i in 0..10 {
                //adc_val += adc1_get_raw(self.y_adc) as u32;
                let mut out: i32 = 0;
                unsafe {
                    adc_oneshot_read(
                        self.yp_adc_handle,
                        self.yp_adc,
                        std::ptr::from_mut(&mut out),
                    );
                }
                z1_adc_val += out;
            }
            z1_adc_val /= 40960;

            let mut z2_adc_val: i32 = 0;
            for i in 0..10 {
                //adc_val += adc1_get_raw(self.y_adc) as u32;
                let mut out: i32 = 0;
                unsafe {
                    adc_oneshot_read(
                        self.xm_adc_handle,
                        self.xm_adc,
                        std::ptr::from_mut(&mut out),
                    );
                }
                z2_adc_val += out;
            }
            z2_adc_val /= 40960;
            //if Z1-Z2 is very high (whole voltage over Z, Pressure resistor) no touch is detected
            //Z1-Z2
            //
            todo!("Pressure detection is yet to develop");
            //Ok((z_adc_val * self.y_max / 40960) as i32)
        }
    }

    pub fn setup_touch_interrupt(&mut self) -> Result<bool, EspError> {
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

        todo!("Pressure detection is yet to develop");

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

        */
    }

    pub fn touch_detection(&mut self) -> Result<bool, EspError> {
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
        //adc1_config_channel_atten(self.y_adc, adc_atten_t_ADC_ATTEN_DB_11);

        let mut y_adc_val: i32 = 0;
        for i in 0..10 {
            //adc_val += adc1_get_raw(self.y_adc) as u32;
            let mut out: i32 = 0;
            unsafe {
                adc_oneshot_read(
                    self.yp_adc_handle,
                    self.yp_adc,
                    std::ptr::from_mut(&mut out),
                );
            }
            y_adc_val += out;
        }
        Ok(true)
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
