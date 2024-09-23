
#include "FUSB302.h"
#include <cstdint>
#include <cstdio>
#include <stdint.h>

// todo
#define FUSB302_SLADDR 0x22

class PDO {
private:
public:
  uint32_t voltage;
  uint32_t current;
  PDO() {}
  PDO(uint8_t data[4]) {
      /*
    voltage = (((uint16_t)(data[2] & 0x0F) << 6) | ((data[1] & 0xFC) >> 2) )*0.05;
    current = (((uint16_t)(data[1] & 0x03) << 8) | data[0])*0.01;
    */
      fromData(data);
  }
  void fromData(uint8_t data[4]) {
    voltage = (((uint16_t)(data[2] & 0x0F) << 6) | ((data[1] >> 2) & 0x3F) )*50;
    current = (((uint16_t)(data[1] & 0x03) << 8) | data[0])*10;
  }
};

TwoWire* i2c;


bool writeReg(uint8_t reg, uint8_t val) {
  i2c->beginTransmission(FUSB302_SLADDR);
  i2c->write(reg);
  i2c->write(val);
  return i2c->endTransmission();
}
uint8_t readReg(uint8_t reg) {
  i2c->beginTransmission(FUSB302_SLADDR);
  i2c->write(reg);
  i2c->endTransmission();
  i2c->requestFrom(FUSB302_SLADDR, 1, true);
  return i2c->read();
}
bool readBMC(uint8_t *data, uint8_t len) {
  i2c->beginTransmission(FUSB302_SLADDR);
  // FIFO reg
  i2c->write(0x43);
  i2c->endTransmission();
  i2c->requestFrom(FUSB302_SLADDR, len, true);
  uint8_t i = 0;
  while (i2c->available()) {
    data[i++] = i2c->read();
  }
  if (i == len) {
    return 0;
  }
  return 1;
}
bool writeBMC(const uint8_t *data, uint8_t len) {
  i2c->beginTransmission(FUSB302_SLADDR);
  // FIFO reg
  i2c->write(0x43);
  for(uint8_t i = 0; i< len; i++){
      i2c->write(data[i]);
  }
  return i2c->endTransmission();
}
bool requestPDO(uint8_t id, uint32_t current_mA, uint32_t maxCurrent_mA){
    /*

    const uint8_t sop_seq[5] = {0x12,0x12,0x12,0x13,0x86};
    const uint8_t eop_seq[4] = {0xff,0x14,0xfe,0xa1};

    uint8_t pdoBytes[6];
    pdoBytes[0] = 0x82;
    pdoBytes[1] = 0x10 | ((0 & 0x07)<<1); // 0 is message id 

    uint16_t maxCurrentBytes = maxCurrent_mA/10;
    uint16_t currentBytes = current_mA/10;

    pdoBytes[2] = maxCurrentBytes & 0xFF;
    pdoBytes[3] = ((maxCurrentBytes & 0x300) >> 8) | ( (currentBytes & 0x3F) << 2);
    pdoBytes[4] = (currentBytes & 0x3C0) >> 6;
    pdoBytes[5] = ((id+1) << 4)|0x01;


    writeBMC(sop_seq, 5);
    writeBMC(pdoBytes, 6);
    return writeBMC(eop_seq, 4);
    */

    uint8_t reqBytes[15] = {0x12,0x12,0x12,0x13,0x86,0,0,0,0,0,0,0xff,0x14,0xfe,0xa1};

    reqBytes[5] = 0x82;
    reqBytes[6] = 0x10 | ((0 & 0x07)<<1); // 0 is message id 

    uint16_t maxCurrentBytes = maxCurrent_mA/10;
    uint16_t currentBytes = current_mA/10;

    reqBytes[7] = maxCurrentBytes & 0xFF;
    reqBytes[8] = ((maxCurrentBytes & 0x300) >> 8) | ( (currentBytes & 0x3F) << 2);
    reqBytes[9] = (currentBytes & 0x3C0) >> 6;
    reqBytes[10] = (2 << 4)|0x01;

    return writeBMC(reqBytes, 15);
}

uint8_t Fusb302::init(uint8_t sda, uint8_t scl, uint32_t i2cfreq) {
    //Serial.printf("init called\n");
    fs::File file = FFat.open("/fusb.txt", FILE_APPEND);
    //Serial.printf("File open successful? (1=yes): %i\n", file);
    i2c = &Wire;
    i2c->begin(sda, scl, 400000);
    //Serial.printf("begun i2c: ");
  // Reset: SW_RES and PD_RES
  writeReg(0x0C, 0x03);
  // Power: enable all
  writeReg(0x0b, 0x0f);
  // Control0: unmask all
  writeReg(0x06, 0x04);
  // Control3: enable three packet retries
  writeReg(0x09, 0x07);


  // now find out CC connection line
  // Switch0: connect adc to cc1
  writeReg(0x02, 0x07);
  delay(1);
  // read Status0
  uint8_t cc1lvl = readReg(0x40) & 0x03;
  // Switch0: connect adc to cc2
  writeReg(0x02, 0x0b);
  delay(1);
  // read Status0
  uint8_t cc2lvl = readReg(0x40) &  0x03;

  if (cc2lvl == cc1lvl) {
    // no usb detected
    Serial.printf("File print successful? (1=yes): %i\n", file.printf("No usbc with PD was detected, cclvl: %i, %i\n", cc1lvl, cc2lvl));
    Serial.printf("No usbc with PD was detected\n");
    file.close();
    Serial.printf("after closing file\n");
    return 1;
  }
  uint8_t switches1 = readReg(0x03) & 0xF8;
  if (cc1lvl > cc2lvl) {
    // cc1 connected

    // Switch0: connect adc to cc1
    writeReg(0x02, 0x07);

    // Switches1: AUTO_CRC and cc1
    switches1 |= 0x01;
  } else {
    // cc2 connected

    // Switches1: AUTO_CRC and cc2
    switches1 |= 0x02;
  }

  //set cc pin as data line
  writeReg(0x03, switches1);

  //set auto crc separately (voltage goes to 0 if done together with previous)
  switches1 |= 0x04;
  writeReg(0x03, switches1);

  // Control0: flush FIFO TX buffer
  writeReg(0x06, 0x44);
  // Control1: flush FIFO RX buffer
  writeReg(0x07, 0x04);
  // RESET: reset PD logic
  writeReg(0x0C, 0x02);

  //Serial.printf("Enter read STATUS1 loop\n");
  // STATUS1: read buffer status
  // bit is set when reg is not empty
  uint32_t timeout = 0;
  uint8_t iActivity = 0;
  while (readReg(0x41) & 0x20) {
      //clear interrupts
    iActivity |= readReg(0x40) & 0x40;
    readReg(0x3E);
    readReg(0x3F);
    // wait until reg contains something
    delay(1);
    if(timeout++ > 500){
        Serial.printf("timeout in status1 loop\n", file.printf("timeout in status1 loop, cclvl: %i, %i, i_ACTIVITY: %i\n", cc1lvl, cc2lvl, iActivity));
        file.close();
        return 1;
    }
  }

  //Serial.printf("exited status1 loop\n", file.printf("exited status1 loop, cclvl: %i, %i\n", cc1lvl, cc2lvl));

  uint8_t messageSize;
  uint8_t myLittleGarbageBin[4];

  {
    uint8_t data[2];
    //discard header
      readBMC(myLittleGarbageBin, 1);
    // read first two bytes (SOPs)
    readBMC(data, 2);

    // mask PDO amount
    messageSize = (data[1] >> 4) & 0x07;
  }

  PDO pdo[8];
  uint8_t id9v = 255;

  for (uint8_t i = 0; i < messageSize; i++) {
    uint8_t data[4];
    readBMC(data, 4);
    pdo[i].fromData(data);
    if(pdo[i].voltage == 9000){
        id9v = i;
    }
  }
  //discard CRC
  readBMC(myLittleGarbageBin, 4);
  if(id9v != 255){
  // Control1: flush FIFO RX buffer
  writeReg(0x07, 0x04);
    requestPDO(id9v,3000,3000);
  }

  Serial.printf("Following %i PDOs were scanned:\n", messageSize);
    file.printf("Found 9v at %i\n", id9v);
    file.printf("Following %i PDOs were scanned:\n", messageSize);
  for (uint8_t i = 0; i < messageSize; i++) {
    file.printf("PDO%i:\n\tv:%i\n\ti:%i\n", i, pdo[i].voltage, pdo[i].current);
    Serial.printf("PDO%i:\n\tv:%i\n\ti:%i\n", i, pdo[i].voltage, pdo[i].current);
  }
  file.close();

  return 0;
}
