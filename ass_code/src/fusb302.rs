extern crate alloc;
use alloc::vec::Vec;
use embassy_time::Timer;
use esp_hal::{
    gpio::AnyPin,
    i2c::master::{Config, Error, I2c},
    peripheral::Peripheral,
    Blocking,
};

#[derive(Copy, Clone)]
pub struct PDO {
    pub id: u8,
    pub voltage: u16,
    pub current: u16,
    pub message_id: u8,
}

impl PDO {
    fn from(data: &[u8; 4], id: u8, message_id: u8) -> Self {
        Self {
            id,
            voltage: ((((data[2] as u16 & 0x0F) << 6) | ((data[1] as u16 >> 2) & 0x3F)) * 50),
            current: ((((data[1] as u16 & 0x03) << 8) | data[0] as u16) * 10),
            message_id
        }
    }
}

pub struct Fusb<'a> {
    i2c: I2c<'a, Blocking>,
    i2c_address: u8,
}
impl<'a> Fusb<'a> {
    pub fn new(
        sda: AnyPin,
        scl: AnyPin,
        i2c: impl Peripheral<P = impl esp_hal::i2c::master::Instance> + 'a,
        i2c_address: u8,
    ) -> Fusb<'a> {
        let i2c = I2c::new(i2c, Config::default())
            .unwrap()
            .with_sda(sda)
            .with_scl(scl);
        Self { i2c, i2c_address }
    }
}

impl Fusb<'_> {
    pub async fn scan_pds(&mut self) -> Result<Vec<PDO>, Error> {
        let mut pdo_vec: Vec<PDO> = Vec::new();

        // Reset: SW_RES and PD_RES
        self.write_reg(0x0C, 0x03)?;
        // Power: enable all
        self.write_reg(0x0b, 0x0f)?;
        // Control0: unmask all
        self.write_reg(0x06, 0x04)?;
        // Control3: enable three packet retries
        self.write_reg(0x09, 0x07)?;

        // now find out CC connection line
        // Switch0: connect adc to cc1
        self.write_reg(0x02, 0x07)?;
        Timer::after_millis(1).await;
        // read Status0
        let cc1lvl = self.read_reg(0x40)? & 0x03;
        // Switch0: connect adc to cc2
        self.write_reg(0x02, 0x0b)?;
        Timer::after_millis(1).await;
        // read Status0
        let cc2lvl = self.read_reg(0x40)? & 0x03;
        if cc2lvl == cc1lvl {
            //no cc communication detected, no PD available
            return Ok(pdo_vec);
        }

        let mut switches1 = self.read_reg(0x03)? & 0xF8;
        if cc1lvl > cc2lvl {
            //cc1 has PD
            self.write_reg(0x02, 0x07)?;
            // Switches1: AUTO_CRC and cc1
            switches1 |= 0x01;
        } else {
            //cc2 has PD
            // Switches1: AUTO_CRC and cc2
            switches1 |= 0x02;
        }
        //set cc pin as data line
        self.write_reg(0x03, switches1)?;

        //return Ok(pdo_vec);
        //set auto crc separately (voltage goes to 0 if done together with previous)
        switches1 |= 0x04;
        self.write_reg(0x03, switches1)?; //CRASHES HERE!!!!
        //return Ok(pdo_vec);

        // Control0: flush FIFO TX buffer
        self.write_reg(0x06, 0x44)?;
        // Control1: flush FIFO RX buffer
        self.write_reg(0x07, 0x04)?;
        // RESET: reset PD logic
        self.write_reg(0x0C, 0x02)?;

        // STATUS1: read buffer status
        // bit is set when reg is not empty
        let mut timeout: u16 = 0;
        while (self.read_reg(0x41)? & 0x20) > 0 {
            //clear interrupts
            self.read_reg(0x40)?;
            self.read_reg(0x3E)?;
            self.read_reg(0x3F)?;
            // wait until reg contains something
            Timer::after_millis(1).await;
            if timeout > 50 {
                return Ok(pdo_vec);
            }
            timeout += 1;
        }

        let mut header_sops: [u8; 3] = [0u8; 3];
        //discard header
        // read first three bytes ((1B)header + (2B)SOPs)
        self.read_bmc(&mut header_sops)?;

        // mask PDO amount
        let message_size: u8 = (header_sops[2] >> 4) & 0x07;
        //let mut pdo: PDO = PDO::from(&[0u8;4]);
        //let mut index_pdo: u8 = 255;
        let message_id : u8 = (header_sops[2] >> 1) & 0x07;

        for i in 0..message_size {
            let mut bmc_data: [u8; 4] = [0u8; 4];
            self.read_bmc(&mut bmc_data)?;
            //pdo = PDO::from(&bmc_data);
            pdo_vec.push(PDO::from(&bmc_data, i, message_id));

            /*
            if pdo.voltage == voltage_mV {
                index_pdo = i;
            }
            */
        }

        //discard CRC
        let mut crc: [u8; 4] = [0u8; 4];
        self.read_bmc(&mut crc)?;

        // Control1: flush FIFO RX buffer
        self.write_reg(0x07, 0x04)?;

        Ok(pdo_vec)
    }

    pub fn request_pdo(
        &mut self,
        pdo: PDO,
        current_milliampere: u16,
        max_current_milliampere: u16,
    ) -> Result<(), Error> {

        let max_current_bits: u16 = max_current_milliampere / 10;
        let current_bits: u16 = current_milliampere / 10;

        let message_id: u8 = (pdo.message_id + 1) % 8;

        let bmc: &[u8] = &[
            //sop
            0x12,
            0x12,
            0x12,
            0x13,
            0x86,
            //content
            0x82, 
            0x10 | ((message_id & 0x07) << 1),
            max_current_bits as u8,
            ((max_current_bits >> 8) & 0x03) as u8 | ((current_bits << 2) & 0xFC) as u8,
            ((current_bits & 0x3C0) >> 6) as u8,
            ((pdo.id + 1) << 4) | 0x01,
            //eop
            0xff, 
            0x14,
            0xfe,
            0xa1,
        ];

        self.write_bmc(bmc)
    }

    fn write_reg(&mut self, reg: u8, val: u8) -> Result<(), Error> {
        self.i2c.write(self.i2c_address, &[reg, val])
    }
    fn read_reg_to_buf(&mut self, reg: u8, buf: &mut u8) -> Result<(), Error> {
        self.i2c.write_read(self.i2c_address, &[reg], &mut [*buf])
    }

    #[inline]
    fn read_reg(&mut self, reg: u8) -> Result<u8, Error> {
        let mut buf: [u8; 1] = [0];
        self.i2c.write_read(self.i2c_address, &[reg], &mut buf)?;
        Ok(buf[0])
    }
    fn write_bmc(&mut self, data: &[u8]) -> Result<(), Error> {
        self.i2c.write(self.i2c_address, &[&[0x43], data].concat())
    }
    fn read_bmc(&mut self, data: &mut [u8]) -> Result<(), Error> {
        self.i2c.write_read(self.i2c_address, &[0x43], data)
    }
}
