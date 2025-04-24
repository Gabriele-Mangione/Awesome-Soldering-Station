
#include "Arduino.h"


#define MPU_INTERRUPT_PIN A7

volatile bool toggler = false;
volatile bool magnet = false;
ISR(INT0_vect) {
    //only motion detection is active, so no need to check what interrupt has occurred
    /*
    i2cMpu.beginTransmission(0x4C);
    i2cMpu.write(0x13);
    i2cMpu.endTransmission();
    i2cMpu.requestFrom(0x4C, 1);
    //while (!i2cMpu.available() && (millis() - currentTime < 100));
        //timeout after 100ms
    while(i2cMpu.available()){
        if (i2cMpu.read() & 0x04) {
        //Motion detected
        }
    }
    */
    toggler = HIGH;
}

ISR(PCINT0_vect){

    magnet = !digitalRead(PIN_PA3);

}

/*
//interrupt not wired (meant for communication)
ISR(PCINT1_vect){
  
}
*/

class I2C {
    private:
    const uint8_t sda, scl, delay_us;
    //state is false when communication runs, and true when communication stops
    bool state = true;
    void start_bit(){
        digitalWrite(sda, LOW);
        delayMicroseconds(delay_us);
        digitalWrite(scl, LOW);
        delayMicroseconds(delay_us);
        state = false;
    }
    void write_bit(bool value){
        digitalWrite(sda, value);
        delayMicroseconds(delay_us);
        digitalWrite(scl, HIGH);
        delayMicroseconds(2*delay_us);
        digitalWrite(scl, LOW);
        delayMicroseconds(delay_us);
    }

    bool read_bit(){
        digitalWrite(sda, HIGH);
        pinMode(sda, INPUT_PULLUP);
        delayMicroseconds(delay_us);
        digitalWrite(scl, HIGH);
        delayMicroseconds(delay_us);
        bool value = digitalRead(sda);
        delayMicroseconds(delay_us);
        digitalWrite(scl, LOW);
        pinMode(sda, OUTPUT);
        delayMicroseconds(delay_us);
        return value;
    }
    void stop_bit(){
        delayMicroseconds(delay_us);
        digitalWrite(scl, HIGH);
        delayMicroseconds(delay_us);
        digitalWrite(sda, HIGH);
        delayMicroseconds(delay_us);
        state = true;
    }

    uint8_t bytesToRead = 0;


    public:
    I2C(uint8_t _sda,uint8_t _scl, uint8_t _delay_us) : sda(_sda), scl(_scl), delay_us(_delay_us){}; 
    begin(){
        pinMode(sda, OUTPUT);
        pinMode(scl, OUTPUT);
        digitalWrite(scl, HIGH);
        digitalWrite(sda, HIGH);
    }
    bool beginTransmission(uint8_t address){
        if(!state){
            return true;
        }
        start_bit();
        for(int8_t i = 6; i >= 0; i--){
            write_bit((address >> i) & 0x01);
        }
        write_bit(0); //write

        bool ack = read_bit();

        if(ack){
            stop_bit();
        }
        return ack;
    }
    bool write(uint8_t data, bool stop = false){
        for(int8_t i = 7; i >= 0; i--){
            write_bit((data >> i) & 0x01);
        }
        bool ack = read_bit();

        if(stop || ack){
            stop_bit();
        }
        return ack;
    }

    bool requestFrom(uint8_t address, uint8_t bytes){
        if(!state){
            return true;
        }
        start_bit();
        for(int8_t i = 6; i >= 0; i--){
            write_bit((address >> i) & 0x01);
        }
        write_bit(1); //write

        bool ack = read_bit();

        if(ack){
            stop_bit();
        }else{
            bytesToRead = bytes;
        }
        return ack;

    }

    uint8_t available(){
        return bytesToRead;
    }

    uint8_t read(bool stop = false){
        uint8_t data = 0;
        for(int8_t i = 7; i >= 0; i--){
            data |= (uint8_t)read_bit() << i;
        }
        bool ack = read_bit();

        if(stop || ack){
            stop_bit();
        }
        if(!ack){
            bytesToRead--;
        }
        return data;
    }
    bool endTransmission(){
        bool prev_state = state;
        stop_bit();
        return prev_state;
    }
};

bool mpuInitMVDT() {
//create new i2c driver
  I2C TinyWireM(PIN_PA6, PIN_PA4,1);
  TinyWireM.begin();
  delay(1);
  //set to standby
  TinyWireM.beginTransmission(0x4C);
  TinyWireM.write(0x07);
  TinyWireM.write(0x00);
  if (TinyWireM.endTransmission())
    return true;
  //interrupt masking
  TinyWireM.beginTransmission(0x4C);
  TinyWireM.write(0x06);
  TinyWireM.write(0x04);
  if (TinyWireM.endTransmission())
    return true;
  //activate Anymotion
  TinyWireM.beginTransmission(0x4C);
  TinyWireM.write(0x09);
  TinyWireM.write(0x04); //1D for every motion type
  if (TinyWireM.endTransmission())
    return true;
  //set range and scale 
  TinyWireM.beginTransmission(0x4C);
  TinyWireM.write(0x20);
  TinyWireM.write(0x09);
  if (TinyWireM.endTransmission())
    return true;
  //set samplerate to max
  TinyWireM.beginTransmission(0x4C);
  TinyWireM.write(0x08);
  TinyWireM.write(0x05);
  if (TinyWireM.endTransmission())
    return true;
    //xyz gain (not necessary says datasheet)
    /*
  TinyWireM.beginTransmission(0x4C);
  TinyWireM.write(0x27);
  TinyWireM.write(0x05);
  TinyWireM.write(0x05);
  TinyWireM.write(0x05);
  if (TinyWireM.endTransmission())
    return true;
    */
  
  //set Anymotion Threshold and debounce
  TinyWireM.beginTransmission(0x4C);
  TinyWireM.write(0x43);
  //15 bit threshold
  TinyWireM.write(0x4F);
  TinyWireM.write(0x00);
  //debounce
  TinyWireM.write(0x03);
  if (TinyWireM.endTransmission())
    return true;
  //set to wake (no more register writing from here on)
  TinyWireM.beginTransmission(0x4C);
  TinyWireM.write(0x07);
  TinyWireM.write(0x01);
  if (TinyWireM.endTransmission())
    return true;

  return false;
}

// one wire functions
/*
class OneWireSlave {
    private:
    const uint8_t pin;

    

    public: 
    OneWireSlave(uint8_t _pin ): pin(_pin){};


};
*/

void setup(){
    cli(); //disable interrupts
    delay(10);
    //setting up movement sensor
    
  while (mpuInitMVDT()){
      delay(10);
  }
  //setup interrupt
  pinMode(PIN_PA3, INPUT_PULLUP); //RMT
  pinMode(PIN_PB2, INPUT_PULLUP); //INT_Gyro
  MCUCR|=_BV(ISC01); //falling edge
GIMSK=_BV(PCIE0)| _BV(INT0); //mask pin change interrupt 0
  //GIFR=_BV(PCIF0); // pin change interrupt flag 0
PCMSK0=_BV(PCINT3); //set interrupt 3, pin PA3
//PCMSK1=_BV(PCINT10); //set interrupt 10, pin PB2

  //attachInterrupt(MPU_INTERRUPT_PIN, movementDetectionISR, FALLING);
  pinMode(PIN_PA5, OUTPUT);
  pinMode(PIN_PA4, OUTPUT);
  pinMode(PIN_PA6, OUTPUT);

    sei(); //enable interrupts
}

bool toggle = false;
void loop(){
  delay(1);
  /*
  bool state = digitalRead(A3);
  digitalWrite(A5, state);
  */
  //toggler = LOW;
  //digitalWrite(PIN_PA5, !digitalRead(PIN_PB2));

  if (toggler == HIGH) {
    toggler = LOW;
  I2C TinyWireM(PIN_PA6, PIN_PA4,1);
    TinyWireM.beginTransmission(0x4C);
    TinyWireM.write(0x14);
    TinyWireM.endTransmission();
    TinyWireM.requestFrom(0x4C, 1);
    //while (TinyWireM.available()){
    //while(i2cMpu.available()){
        if (TinyWireM.read() & 0x04) {
        //Motion detected
        digitalWrite(A5, HIGH);
        }
    //}
        //digitalWrite(A5, HIGH);
  }
  else if (magnet == HIGH){
    digitalWrite(A5, HIGH);
  }
  else {
    digitalWrite(A5, LOW);
  }
}
