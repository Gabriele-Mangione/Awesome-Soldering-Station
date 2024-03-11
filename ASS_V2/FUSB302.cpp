
#include "FUSB302.h"
#include <cstdint>

// todo
#define FUSB302_SLADDR 0x1

class PDO {
private:
public:
  float voltage;
  float current;
  PDO() {}
  PDO(uint8_t data[4]) {
    voltage = (((uint16_t)(data[2] & 0x0F) << 6) | ((data[1] >> 2) & 0x3F))*0.05;
    current = (((uint16_t)(data[1] & 0x03) << 8) | data[0])*0.01;
  }
  void fromData(uint8_t data[4]) {
    voltage = (((uint16_t)(data[2] & 0x0F) << 6) | ((data[1] >> 2) & 0x3F))*0.05;
    current = (((uint16_t)(data[1] & 0x03) << 8) | data[0])*0.01;
  }
};

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
bool readBMC(uint8_t *data, uint8_t len) {
  i2c.beginTransmission(FUSB302_SLADDR);
  // FIFO reg
  i2c.write(0x43);
  i2c.endTransmission();
  i2c.requestFrom(FUSB302_SLADDR, len, true);
  uint8_t i = 0;
  while (i2c.available()) {
    data[i++] = i2c.read();
  }
  if (i == len) {
    return 0;
  }
  return 1;
}
bool writeBMC(uint8_t *data, uint8_t len) {
  i2c.beginTransmission(FUSB302_SLADDR);
  // FIFO reg
  i2c.write(0x43);
  for(uint8_t i = 0; i< len; i++){
      i2c.write(data[i]);
  }
  return i2c.endTransmission();
}
bool requestPDO(uint8_t id, float current, float maxCurrent){
    const uint8_t sop_seq[5] = {0x12,0x12,0x12,0x13,0x80};
    const uint8_t eop_seq[4] = {0xff,0x14,0xfe,0xa1};

    uint8_t pdoBytes[6];
    pdoBytes[0] = 0x82;
    pdoBytes[1] = 0x10 | ((id& 0x07)<<1);

    uint16_t maxCurrentBytes = maxCurrent * 20;
    uint16_t currentBytes = current * 20;

    pdoBytes[2] = maxCurrentBytes & 0xFF;
    pdoBytes[3] = ((maxCurrentBytes & 0x300) >> 8) | ( (currentBytes & 0x3F) << 2);
    pdoBytes[4] = (currentBytes & 0x3C0) >> 6;
    pdoBytes[5] = 0x11;

    writeBMC(sop_seq, 5);
    writeBMC(pdoBytes, 6);
    writeBMC(eop_seq, 4);
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
  writeReg(0x06, 0x40);
  // Control1: flush FIFO RX buffer
  writeReg(0x07, 0x04);
  // RESET: reset PD logic
  writeReg(0x0C, 0x02);

  // STATUS1: read buffer status
  // bit is set when reg is not empty
  while (readReg(0x41) & 0x20) {
    // wait until reg contains something
    delay(1);
  }
  uint8_t messageSize;

  {
    // read first two bytes (SOPs)
    uint8_t data[2];
    readBMC(data, 2);

    // mask PDO amount
    messageSize = (data[1] >> 4) & 0x70;
  }

  PDO pdo[8];

  for (uint8_t i = 0; i < messageSize; i++) {
    uint8_t data[4];
    readBMC(data, 4);
    pdo[i].fromData(data);
  }
  requestPDO(1,1.,3.);

  for (uint8_t i = 0; i < messageSize; i++) {
    Serial.printf("PDO%i:\n\tv:%f\n\ti:%f", i, pdo[i].voltage, pdo[i].current);
  }
}
