
use esp_idf_hal::sys::*;

pub enum PinMode {
    INPUT,
    OUTPUT
}

pub struct FancyPin {
    pin_num: u8,
    adc_channel: u8
}

impl FancyPin {
    
    pub fn new(pin_num: u8) -> FancyPin {
        Self{pin_num, }
    }

    pub fn set_mode(&self,mode: PinMode){

    }

    pub fn set_high(&self) {
        unsafe { gpio_set_level(self.pin_num as i32, 1)};
    }

    pub fn set_low(&self) {
        unsafe { gpio_set_level(self.pin_num as i32, 0)};

    }

    pub fn analog_read(&self) -> u16{
        unsafe{
            let mut handle: adc_continuous_handle_t = core::ptr::null_mut();
            adc_continuous_new_handle(
                    &adc_continuous_handle_cfg_t {
                        max_store_buf_size: SOC_ADC_DIGI_DATA_BYTES_PER_CONV
                            * (config.frame_measurements as u32)
                            * (config.frames_count as u32),
                        conv_frame_size: SOC_ADC_DIGI_DATA_BYTES_PER_CONV
                            * (config.frame_measurements as u32),
                        ..Default::default()
                    },
                    &mut handle,
                );

        }

    }
}
    pin!(Gpio0:0, IO, RTC:0, NOADC:0, NODAC:0, NOTOUCH:0);
    pin!(Gpio1:1, IO, RTC:1, ADC1:0, NODAC:0, TOUCH:1);
    pin!(Gpio2:2, IO, RTC:2, ADC1:1, NODAC:0, TOUCH:2);
    pin!(Gpio3:3, IO, RTC:3, ADC1:2, NODAC:0, TOUCH:3);
    pin!(Gpio4:4, IO, RTC:4, ADC1:3, NODAC:0, TOUCH:4);
    pin!(Gpio5:5, IO, RTC:5, ADC1:4, NODAC:0, TOUCH:5);
    pin!(Gpio6:6, IO, RTC:6, ADC1:5, NODAC:0, TOUCH:6);
    pin!(Gpio7:7, IO, RTC:7, ADC1:6, NODAC:0, TOUCH:7);
    pin!(Gpio8:8, IO, RTC:8, ADC1:7, NODAC:0, TOUCH:8);
    pin!(Gpio9:9, IO, RTC:9, ADC1:8, NODAC:0, TOUCH:9);
    pin!(Gpio10:10, IO, RTC:10, ADC1:9, NODAC:0, TOUCH:10);
    pin!(Gpio11:11, IO, RTC:11, ADC2:0, NODAC:0, TOUCH:11);
    pin!(Gpio12:12, IO, RTC:12, ADC2:1, NODAC:0, TOUCH:12);
    pin!(Gpio13:13, IO, RTC:13, ADC2:2, NODAC:0, TOUCH:13);
    pin!(Gpio14:14, IO, RTC:14, ADC2:3, NODAC:0, TOUCH:14);
    pin!(Gpio15:15, IO, RTC:15, ADC2:4, NODAC:0, NOTOUCH:0);
    pin!(Gpio16:16, IO, RTC:16, ADC2:5, NODAC:0, NOTOUCH:0);
    #[cfg(esp32s2)]
    pin!(Gpio17:17, IO, RTC:17, ADC2:6, DAC:1, NOTOUCH:0);
    #[cfg(esp32s3)]
    pin!(Gpio17:17, IO, RTC:17, ADC2:6, NODAC:0, NOTOUCH:0);
    #[cfg(esp32s2)]
    pin!(Gpio18:18, IO, RTC:18, ADC2:7, DAC:2, NOTOUCH:0);
    #[cfg(esp32s3)]
    pin!(Gpio18:18, IO, RTC:18, ADC2:7, NODAC:0, NOTOUCH:0);
    pin!(Gpio19:19, IO, RTC:19, ADC2:8, NODAC:0, NOTOUCH:0);
    pin!(Gpio20:20, IO, RTC:20, ADC2:9, NODAC:0, NOTOUCH:0);
