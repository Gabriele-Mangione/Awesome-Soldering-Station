use esp_hal::{
    gpio::AnyPin, i2c::master::{AnyI2c, Config, Error, I2c}, time, Blocking
};
use log::info;
use time::RateExtU32;


//TODO:
//https://www.buydisplay.com/download/ic/FT6206.pdf

const I2C_ADDR:u8 = 0x38;

const DEV_MODE_REG: u8 = 0x00;
const GEST_ID_REG: u8 = 0x01;
const TD_STATUS_REG: u8 = 0x02;
const P1_XH_REG: u8 = 0x03;
const P1_XL_REG: u8 = 0x04;
const P1_YH_REG: u8 = 0x05;
const P1_YL_REG: u8 = 0x06;
const P1_WEIGHT_REG: u8 = 0x07;
const P1_MISC_REG: u8 = 0x08;
const P2_XH_REG: u8 = 0x09;
const P2_XL_REG: u8 = 0x0A;
const P2_YH_REG: u8 = 0x0B;
const P2_YL_REG: u8 = 0x0C;
const P2_WEIGHT_REG: u8 = 0x0D;
const P2_MISC_REG: u8 = 0x0E;

const TH_GROUP_REG: u8 = 0x80;
const TH_DIFF_REG: u8 = 0x85;
const CTRL_REG: u8 = 0x86;
const TIME_ENTER_MONITOR_REG: u8 = 0x87;
const PERIOD_ACTIVE_REG: u8 = 0x88;
const PERIOD_MONITOR_REG: u8 = 0x89;
const RADIAN_VALUE_REG: u8 = 0x91;
const OFFSET_LEFT_RIGHT_REG: u8 = 0x92;
const OFFSET_UP_DOWN_REG: u8 = 0x93;
const DISTANCE_LEFT_RIGHT_REG: u8 = 0x94;
const DISTANCE_UP_DOWN_REG: u8 = 0x95;
const DISTANCE_ZOOM_REG: u8 = 0x96;

const LIB_VER_H_REG: u8 = 0xA1;
const LIB_VER_L_REG: u8 = 0xA2;
const CIPHER_REG: u8 = 0xA3;
const G_MODE_REG: u8 = 0xA4;
const PWR_MODE_REG: u8 = 0xA5;
const FIRMID_REG: u8 = 0xA6;
const FOCALTECH_ID_REG: u8 = 0xA8;
const RELEASE_CODE_ID_REG: u8 = 0xAF;
const STATE_REG: u8 = 0xBC;

pub enum TouchEventFlag {
    PressDown,
    LiftUp,
    Contact,
    NoEvent
}

impl TouchEventFlag {
    fn from(n: u8) -> Self{
        match n {
            0 => Self::PressDown,
            1 => Self::LiftUp,
            2 => Self::Contact,
            3.. => Self::NoEvent
        }
    }
}
pub struct Touch {
    pub x: u16,
    pub y: u16,
    //pub weight: u8,
    //pub area: u8,
    pub id: u8,
    pub event_flag: TouchEventFlag,
}

pub struct TouchBreakoutCap<'a> {
    i2c: I2c<'a, Blocking>,
}
impl<'a> TouchBreakoutCap<'a> {
    pub fn new(
        sda: AnyPin,
        scl: AnyPin,
        i2c: AnyI2c,
    ) -> TouchBreakoutCap<'a> {
        
        let i2c = I2c::new(i2c, Config::default().with_frequency(400.kHz()))
            .unwrap()
            .with_sda(sda)
            .with_scl(scl);
        Self { i2c }

        /*
        let mut a = Self { i2c};
        //these registers of the chip don't do anything...........
        a.write_reg(CTRL_REG, 0x00).unwrap(); //keep active mode when there is no touching
        a.write_reg(PERIOD_ACTIVE_REG, 0xFF).unwrap(); //set scan frequency to 10Hz
        a.write_reg(PERIOD_MONITOR_REG, 0xFF).unwrap(); //set scan frequency to 10Hz

        info!("CTRL_REG: {}, PAR: {}, PMR: {}", a.read_reg(CTRL_REG).unwrap(), a.read_reg(PERIOD_ACTIVE_REG).unwrap(), a.read_reg(PERIOD_MONITOR_REG).unwrap());
        a
        */

    }
}

impl TouchBreakoutCap<'_> {
    pub fn read_points(&mut self) -> Result<(Touch, Option<Touch>), Error> {
        let mut r: [u8; 6] = [0; 6];
        self.read_reg_to_buf(P1_XH_REG, &mut r)?;
        let t0: Touch = Touch {
            x: (((r[0] & 0x0F) as u16) << 8) | r[1] as u16,
            y: (((r[2] & 0x0F) as u16) << 8) | r[3] as u16,
            //weight: r[4],
            //area: r[5] >> 4,
            id: r[2] >> 4,
            event_flag: TouchEventFlag::from(r[0] >> 6),
        };
        if (self.read_reg(TD_STATUS_REG)?) == 2 {
            self.read_reg_to_buf(P2_XH_REG, &mut r)?;
            let t1: Touch = Touch {
                x: (((r[0] & 0x0F) as u16) << 8) | r[1] as u16,
                y: (((r[2] & 0x0F) as u16) << 8) | r[3] as u16,
                //weight: r[4],
                //area: r[5] >> 4,
                id: r[2] >> 4,
                event_flag: TouchEventFlag::from(r[0] >> 6),
            };

            return Ok((t0, Some(t1)));
        }
        Ok((t0, None))
    }
}

impl TouchBreakoutCap<'_> {
    fn write_reg(&mut self, reg: u8, val: u8) -> Result<(), Error> {
        self.i2c.write(I2C_ADDR, &[reg, val])
    }
    fn read_reg_to_buf(&mut self, reg: u8, buf: &mut [u8]) -> Result<(), Error> {
        self.i2c.write_read(I2C_ADDR, &[reg], buf)
    }

    #[inline]
    fn read_reg(&mut self, reg: u8) -> Result<u8, Error> {
        let mut buf: [u8; 1] = [0];
        self.i2c.write_read(I2C_ADDR, &[reg], &mut buf)?;
        Ok(buf[0])
    }
}
