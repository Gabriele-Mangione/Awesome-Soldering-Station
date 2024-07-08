#include <ATTinyCore.h>


#ifndef _FUSB302_
#define _FUSB302_


#include <Arduino.h>
#include "FFat.h"
#include <Wire.h>
#include <cstdint>
namespace Fusb302 {

    uint8_t init(uint8_t sda, uint8_t scl, uint32_t i2cfreq);


}


#endif
