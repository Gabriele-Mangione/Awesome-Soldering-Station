use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use embedded_graphics::prelude::Point;
use esp_idf_hal::{
    adc::AdcChannelDriver,
    gpio::{self, AnyIOPin, InputOutput, InputPin, OutputPin, PinDriver, Pins, ADCPin, RtcInputOutput},
    peripheral::{Peripheral, PeripheralRef},
    sys::EspError,
};

pub struct TouchBreakout {
    xp_pin: PinDriver<'static, AnyIOPin, RtcInputOutput>,
    yp_pin: PinDriver<'static, AnyIOPin, RtcInputOutput>,
    xm_pin: PinDriver<'static, AnyIOPin, RtcInputOutput>,
    ym_pin: PinDriver<'static, AnyIOPin, RtcInputOutput>,
}

impl TouchBreakout {
    pub fn new(
        xp_pin: PinDriver<'static, AnyIOPin, RtcInputOutput>,
        yp_pin: PinDriver<'static, AnyIOPin, RtcInputOutput>,
        xm_pin: PinDriver<'static, AnyIOPin, RtcInputOutput>,
        ym_pin: PinDriver<'static, AnyIOPin, RtcInputOutput>,
    ) -> TouchBreakout {
        TouchBreakout {
            xp_pin,
            yp_pin,
            xm_pin,
            ym_pin,
        }
    }

    pub fn get_point(&mut self) -> Result<Option<Point>, EspError> {
        let x = self.get_x()?;
        let y = self.get_y()?;

        Ok(Some(Point::new(x, y)))
    }

    fn get_x(&mut self) -> Result<i32, EspError> {
        self.yp_pin.into;
        self.ym_pin.get_level();
        self.xp_pin.into_output().unwrap().set_high();
        self.xm_pin.into_output().unwrap().set_low();
        todo!()
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

    fn get_y(&mut self) -> Result<i32, EspError> {
        //self.yp_pin;
        todo!()
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
