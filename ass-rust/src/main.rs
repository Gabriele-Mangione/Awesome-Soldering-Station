use esp_idf_svc::hal::{prelude::Peripherals, i2c::{I2cDriver, I2cConfig}};

mod mpu6050;

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    let sda = peripherals.pins.gpio10;
    let scl = peripherals.pins.gpio11;
    let int = peripherals.pins.gpio12;

    let i2c_configs = I2cConfig::new().baudrate(400.kHz().into()?;
    let mut mpu = MPU6050::new(I2cDriver::new(peripherals.i2c0,sda,scl, &i2c_configs));

    log::info!("Hello, world!");
    loop {


    }
}
