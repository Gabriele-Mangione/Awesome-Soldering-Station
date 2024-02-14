use esp_idf_svc::{hal::{i2c::{I2cDriver, I2cConfig}, peripheral::Peripheral, gpio::{InputPin, OutputPin}, delay::{BLOCK, FreeRtos}}, sys::EspError};

const I2C_ADDR : u8 = 0x68;

#[derive(Debug, Clone, Copy)]
struct MPU6050{
    i2c_driver: I2cDriver,
}


impl MPU6050{

    pub fn new(&self, i2c: impl Peripheral<P = I2C>, sda: impl Peripheral<P = impl InputPin + OutputPin>, scl: impl Peripheral<P = impl InputPin + OutputPin>) -> Result<MPU6050, u8>{
        let i2c_config = I2cConfig::new().baudrate(400.kHz().into());
        self::i2c_driver = I2cDriver::new(i2c, sda, scl, &i2c_config);
    }

    pub fn init_motionalert(&self)-> Result<(), EspError> {
        let register_addr = 56;
        let data: [u8; 2] = [register_addr, 0x40];
        //reset
        self.i2c_driver.write(I2C_ADDR, &(0x6B, 0x80), BLOCK)?;
        FreeRtos::delay_ms(10);
        //wakeup
        self.i2c_driver.write(I2C_ADDR, &[0x6B as u8, 0x00 as u8], BLOCK)?;
        FreeRtos::delay_ms(5);
        self.i2c_driver.write(I2C_ADDR, &[0x6B as u8, 0x00 as u8], BLOCK)?;
        //interrupt config
        self.i2c_driver.write(I2C_ADDR, &[0x38 as u8, 0x40 as u8], BLOCK)
    }

}
