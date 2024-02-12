use embedded_hal::blocking::i2c::{Write,WriteRead};


#[derive(Debug, Clone, Copy)]
struct MPU6050 <I2C>{
    let i2c_driver : I2C,
    let i2c_addr : u8,
}


impl<I2C> MPU6050{

    pub fn new(i2c_driver: I2C, i2c_addr: u8) -> Result<Self, u8>{


    }

    pub fn init_motionalert()-> Result<(&mut self), u8> {
        let register_addr = 0;
        let mut buffer = [0u8];
        self.i2c_driver.write_read(self.i2c_addr, &register_addr, &mut buffer)
    }

}
