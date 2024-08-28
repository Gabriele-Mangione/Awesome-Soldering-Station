
#ifndef _TOUCHBREAKOUT_
#define _TOUCHBREAKOUT_

#include <Arduino.h>

#define NUMBER_OF_MEASUREMENTS 10
#define MAX_SWING_ACCURACY_X 96
#define MAX_SWING_ACCURACY_Y 200


class TouchPoint {
private:
  uint8_t xpPin = -1;
  uint8_t ypPin = -1;
  uint8_t xmPin = -1;
  uint8_t ymPin = -1;

  uint16_t min(uint16_t* measurementsArray);
  uint16_t max(uint16_t* measurementsArray);
  uint16_t average(uint16_t* measurementsArray);

public:
  TouchPoint(uint8_t xpPin, uint8_t ypPin, uint8_t xmPin, uint8_t ymPin);
  uint16_t getX();
  uint16_t getY();
  uint16_t getZ();
};

#endif