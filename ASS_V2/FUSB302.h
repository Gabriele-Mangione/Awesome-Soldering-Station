
#ifndef _FUSB302_
#define _FUSB302_


#include <Wire.h>
#include <cstdint>
namespace Fusb302 {

    uint8_t init(uint8_t sda, uint8_t scl, uint8_t i2cfreq);


}


#endif
