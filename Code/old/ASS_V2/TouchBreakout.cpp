
#include "TouchBreakout.h"


uint16_t TouchPoint::min(uint16_t* measurementsArray) {
  uint16_t out = measurementsArray[0];
  for (uint8_t i = 1; i < NUMBER_OF_MEASUREMENTS; i++) {
    if (out > measurementsArray[i]) {
      out = measurementsArray[i];
    }
  }
  return out;
}

uint16_t TouchPoint::max(uint16_t* measurementsArray) {
  uint16_t out = measurementsArray[0];
  for (uint8_t i = 1; i < NUMBER_OF_MEASUREMENTS; i++) {
    if (out < measurementsArray[i]) {
      out = measurementsArray[i];
    }
  }
  return out;
}

uint16_t TouchPoint::average(uint16_t* measurementsArray) {
  uint16_t out = 0;
  for (uint8_t i = 0; i < NUMBER_OF_MEASUREMENTS; i++) {
    out += measurementsArray[i];
  }
  return out / NUMBER_OF_MEASUREMENTS;
}


TouchPoint::TouchPoint(uint8_t xpPinIn, uint8_t ypPinIn, uint8_t xmPinIn, uint8_t ymPinIn) {
  xpPin = xpPinIn;
  ypPin = ypPinIn;
  xmPin = xmPinIn;
  ymPin = ymPinIn;
}

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

uint16_t TouchPoint::getY() {
  pinMode(ypPin, OUTPUT);
  pinMode(ymPin, OUTPUT);
  pinMode(xpPin, INPUT);
  pinMode(xmPin, INPUT);
  digitalWrite(ypPin, HIGH);
  digitalWrite(ymPin, LOW);
  delayMicroseconds(50);
  uint16_t Ymeasured[NUMBER_OF_MEASUREMENTS] = { 0 };
  for (uint8_t i = 0; i < NUMBER_OF_MEASUREMENTS; i++) {
    Ymeasured[i] = analogRead(xmPin);
    delayMicroseconds(5);
  }

  if (max(Ymeasured) - min(Ymeasured) < MAX_SWING_ACCURACY_Y) {
    return average(Ymeasured);
  }
  return 0;
}

uint16_t TouchPoint::getZ() {
  return 0;
}