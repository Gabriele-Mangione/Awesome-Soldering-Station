
#include "FUSB302.h"
#include <cstdint>

// todo
#define FUSB302_SLADDR 0x1

TwoWire i2c;

bool writeReg(uint8_t reg, uint8_t val) {
  i2c.beginTransmission(FUSB302_SLADDR);
  i2c.write(reg);
  i2c.write(val);
  return i2c.endTransmission();
}
uint8_t readReg(uint8_t reg) {
  i2c.beginTransmission(FUSB302_SLADDR);
  i2c.write(reg);
  i2c.endTransmission();
  i2c.requestFrom(FUSB302_SLADDR, 1, true);
  return i2c.read();
}
bool readBMC(uint8_t* data, uint8_t len){
  i2c.beginTransmission(FUSB302_SLADDR);
  //FIFO reg
  i2c.write(0x43);
  i2c.endTransmission();
  i2c.requestFrom(FUSB302_SLADDR, len, true);
  uint8_t i = 0;
  while(i2c.available()){
    data[i++] = i2c.read();
  }
  if(i == len){
      return 0;
  }
  return 1;
}


uint8_t Fusb302::init(uint8_t sda, uint8_t scl, uint8_t i2cfreq) {
  i2c.begin(scl, scl, i2cfreq);

  // Reset: SW_RES
  writeReg(0x0C, 0x01);
  // Power: enable all
  writeReg(0x0b, 0x0f);
  // Control0: unmask all
  writeReg(0x06, 0x00);
  // Control3: enable three packet retries
  writeReg(0x09, 0x07);

  // now find out CC connection line
  // Switch0: connect adc to cc1
  writeReg(0x02, 0x07);
  // read Status0
  uint8_t cc1lvl = readReg(0x40);
  // Switch0: connect adc to cc2
  writeReg(0x02, 0x0b);
  // read Status0
  uint8_t cc2lvl = readReg(0x40);
  if (cc2lvl == cc1lvl) {
    // no usb detected
    return 1;
  }
  if (cc1lvl > cc2lvl) {
    // cc1 connected
    // Switches1: AUTO_CRC and cc1
    writeReg(0x03, 0x25);
    // Switch0: connect adc to cc1
    writeReg(0x02, 0x07);
  } else {
    // cc2 connected
    // Switches1: AUTO_CRC and cc1
    writeReg(0x03, 0x26);
  }

  // Control0: flush FIFO TX buffer
  writeReg(0x06,0x40);
  // Control1: flush FIFO RX buffer
  writeReg(0x07,0x04);
  // RESET: reset PD logic
  writeReg(0x0C,0x02);

  // STATUS1: read buffer status
  // bit is set when reg is not empty
  while(readReg(0x41) & 0x20){
      //wait until reg contains something
      delay(1);
  }
  uint8_t data[80];
  readBMC(data, 80);


}
