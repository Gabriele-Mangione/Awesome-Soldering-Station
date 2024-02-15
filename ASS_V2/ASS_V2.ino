/*
// The ESP32 and TFT the pins used for testing are:
#define TFT_CS   47//33  // Chip select control pin (library pulls permanently low
#define TFT_DC   48 //15  // Data Command control pin - must use a pin in the range 0-31
#define TFT_RST  0 //32  // Reset pin, toggles on startup

#define TFT_WR    43  // Write strobe control pin - must use a pin in the range 0-31
#define TFT_RD    44  // Read strobe control pin

//RST
//Lite
//4 TouchPins

#define TFT_D0   42  // Must use pins in the range 0-31 for the data bus
#define TFT_D1   41  // so a single register write sets/clears all bits.
#define TFT_D2   40 //26  // Pins can be randomly assigned, this does not affect
#define TFT_D3   39 //25  // TFT screen update performance.
#define TFT_D4   38
#define TFT_D5   37
#define TFT_D6   36 //27
#define TFT_D7   35 //14
*/

//TODO check if iron is connected, through i2c ack bit:
//i2c_master_cmd_begin returns ESP_FAIL when not receiving ACK
//Register 56 of mpu must be set to 0x40 for interrupt on motion detection

//TODOOOO implement new TFT_eSPI lib version!!!


#include <SPI.h>
#include <Wire.h>
#include <TFT_eSPI.h>  // Hardware-specific library
#include "TouchBreakout.h"
#include <WiFi.h>
#include <ESPmDNS.h>
#include <WiFiUdp.h>
#include <ArduinoOTA.h>
#include "ringedBuffer.h"

#define SSID_WIFI "Coldspot"
#define PW_WIFI "Hotstoppassword"

#define YP 1
#define YM 14
#define XP 13
#define XM 2

#define SOLDERTEMP_PIN 8
#define SOLDER_OD 9

TaskHandle_t solderTask;
TaskHandle_t wiFiTask;
TaskHandle_t displayTask;
TaskHandle_t sensorTask;

SemaphoreHandle_t semaphoreMVDT;

const uint8_t INTERRUPT_PIN = 12;
const uint8_t SCL_PIN = 11;
const uint8_t SDA_PIN = 10;

uint16_t goalTemp = 380;
uint16_t actualTemp = 0;
int standbyTime = 60;

enum DeviceMode {
  RUNNING,
  STANDBY
} deviceMode;

//for adafruit touchscreen library
//TouchScreen ts = TouchScreen(XP, YP, XM, YM, 340); // X+ to X- 340 Ohm
void setup(void) {
  delay(1000);
  xTaskCreatePinnedToCore(solderProcess, "Solder Task", 10000, NULL, 1, &solderTask, 1);
  xTaskCreatePinnedToCore(wiFiProcess, "WiFi Task", 10000, NULL, 0, &wiFiTask, 0);
  xTaskCreatePinnedToCore(displayProcess, "Display Task", 10000, NULL, 0, &displayTask, 0);
  xTaskCreatePinnedToCore(sensorProcess, "Sensor and MPU Task", 10000, NULL, 0, &sensorTask, 0);
}

class Button {
private:
  uint16_t x;
  uint16_t y;
  uint16_t w;
  uint16_t h;
  uint16_t r;
  uint16_t colour;
  TFT_eSPI* tft = NULL;

public:
  Button(TFT_eSPI* _tft, uint16_t _x, uint16_t _y, uint16_t _w, uint16_t _h, uint16_t _r) {
    tft = _tft;
    x = _x;
    y = _y;
    w = _w;
    h = _h;
    r = _r;
  }

  void setColor(uint16_t _colour) {
    if (colour != _colour) {
      tft->fillRoundRect(x, y, w, h, r, _colour);
      colour = _colour;
    }
  }

  bool isPressed(uint16_t _x, uint16_t _y) {
    return _x > x && _x < (x + w) && _y > y && _y < (y + h);
  }
};

void solderProcess(void* pvParameters) {

  pinMode(SOLDERTEMP_PIN, INPUT);
  pinMode(SOLDER_OD, OUTPUT);


  RingedBuffer<float, 30> temperatureBuffer;
  while (true) {
    // deactivate Output in order to read the temperature
    digitalWrite(SOLDER_OD, LOW);
    // wait 2 milliseconds to prevent bad measurements from em of switching voltage
    delay(2);
    // read the amplified temperature voltage and convert it into temperature
    temperatureBuffer.push((float)analogRead(SOLDERTEMP_PIN));
    actualTemp = map((uint16_t)temperatureBuffer.avg(), 0, 4095, 20, 600);
    switch (deviceMode) {
      case RUNNING:
        {

          // activate Soldering Iron if goalTemp is not yet reached
          float deltaTemp = goalTemp - actualTemp;
          if (actualTemp < goalTemp) {
            digitalWrite(SOLDER_OD, HIGH);
          } else {
            digitalWrite(SOLDER_OD, LOW);
          }
          if (deltaTemp > 0) {
            delay((uint16_t)deltaTemp /10);
          }
          delay(1);
        }
        break;
      case STANDBY:
        {
          digitalWrite(SOLDER_OD, LOW);
        }
        break;
      default:
        {
        }
        break;
    }
  }
}

void displayProcess(void* pvParameters) {
  TFT_eSPI tft = TFT_eSPI();
  TouchPoint TouchScreen = TouchPoint(XP, YP, XM, YM);

  Button AugButton(&tft, 200, 30, 100, 70, 5);
  Button DecButton(&tft, 200, 140, 100, 70, 5);

  tft.init();
  tft.setRotation(1);
  tft.setTextSize(3);

  //tft.fillScreen(TFT_BLACK);
  //tft.fillScreen(TFT_RED);
  //tft.fillScreen(TFT_GREEN);
  //tft.fillScreen(TFT_BLUE);
  //tft.fillScreen(TFT_BLACK);
  tft.fillScreen(TFT_BLACK);

  unsigned long blinkTimer = 0;


  while (true) {

    if (millis() - blinkTimer > 500) {
      blinkTimer = millis();
    }

    switch (deviceMode) {
      case RUNNING:
        {
          static unsigned long lastSecond = 0;
          if (millis() / 1000 != lastSecond) {
            lastSecond = millis() / 1000;
            standbyTime--;
          }
          if (standbyTime < 1) {
            deviceMode = DeviceMode::STANDBY;
          }
        }
        break;
      case STANDBY:
        {
        }
        break;
      default:
        {
        }
        break;
    }
    uint16_t x = map(TouchScreen.getX(), 4000, 500, 0, 320);
    uint16_t y = map(TouchScreen.getY(), 500, 3800, 0, 240);

    x = x > 320 ? 0 : x;
    y = y > 240 ? 0 : y;

    if (AugButton.isPressed(x, y)) {
      AugButton.setColor(0xDFE0);
      goalTemp++;
    } else {
      AugButton.setColor(TFT_GREEN);
    }

    if (DecButton.isPressed(x, y)) {
      DecButton.setColor(0xFA80);
      goalTemp--;
    } else {
      DecButton.setColor(TFT_RED);
    }

    tft.setCursor(0, 10);
    tft.setTextColor(0xFFFF, 0x0000);
    tft.printf("Set Temp:\n");
    tft.setTextSize(5);
    tft.printf("%3i C\n", goalTemp);
    tft.setTextSize(3);
    tft.printf("Meas Temp:\n");
    tft.setTextSize(5);
    tft.printf("%3i C\n", actualTemp);
    tft.setTextSize(3);
    if (deviceMode == DeviceMode::STANDBY && millis() - blinkTimer < 250) {
      tft.setTextColor(0x841F, 0x0000);
      tft.printf("STANDBY");
      tft.setTextColor(0xFFFF, 0x0000);
    } else {
      tft.printf("       ");
    }
    tft.setCursor(0, 216);
    tft.printf("timer: %3i", standbyTime);
    delay(10);
  }
}

void wiFiProcess(void* pvParameters) {
  //WiFi Setup
  WiFi.mode(WIFI_STA);
  WiFi.begin(SSID_WIFI, PW_WIFI);
  //Serial.print("Connecting to WiFi");

  while (WiFi.status() != WL_CONNECTED) {
    //Serial.print(".");
    delay(1000);
  }
  //Serial.print("\nConnected!");

  //OTA Setup
  ArduinoOTA.setHostname("AwesomeSolderingStation");
  ArduinoOTA.setPassword("esp32");

  ArduinoOTA
    .onStart([]() {
      String type;
      if (ArduinoOTA.getCommand() == U_FLASH)
        type = "sketch";
      else  // U_SPIFFS
        type = "filesystem";

      // NOTE: if updating SPIFFS this would be the place to unmount SPIFFS using SPIFFS.end()
      //Serial.println("Start updating " + type);
    })
    .onEnd([]() {
      //Serial.println("\nEnd");
    })
    .onProgress([](unsigned int progress, unsigned int total) {
      //Serial.printf("Progress: %u%%\r", (progress / (total / 100)));
    })
    .onError([](ota_error_t error) {
      /*Serial.printf("Error[%u]: ", error);
      if (error == OTA_AUTH_ERROR) Serial.println("Auth Failed");
      else if (error == OTA_BEGIN_ERROR) Serial.println("Begin Failed");
      else if (error == OTA_CONNECT_ERROR) Serial.println("Connect Failed");
      else if (error == OTA_RECEIVE_ERROR) Serial.println("Receive Failed");
      else if (error == OTA_END_ERROR) Serial.println("End Failed");*/
    });

  ArduinoOTA.begin();

  while (true) {
    ArduinoOTA.handle();
  }
}

bool mpuInitMVDT() {
  Wire.begin(SDA_PIN, SCL_PIN, 10000);
  delay(10);
  // reset
  Wire.beginTransmission(0x68);
  Wire.write(0x6B);
  Wire.write(0x80);
  if (Wire.endTransmission())
    return true;
  delay(10);
  // wakeup
  Wire.beginTransmission(0x68);
  Wire.write(0x6B);
  Wire.write(0x00);
  if (Wire.endTransmission())
    return true;
  delay(5);
  Wire.beginTransmission(0x68);
  Wire.write(0x6B);
  Wire.write(0x00);
  if (Wire.endTransmission())
    return true;

  //set DLPF to 1 kHz
  Wire.beginTransmission(0x68);
  Wire.write(0x1A);
  Wire.write(0x01);
  if (Wire.endTransmission())
    return true;

  //set acc config
  Wire.beginTransmission(0x68);
  Wire.write(0x1C);
  Wire.write(0x10);
  if (Wire.endTransmission())
    return true;

  //set gyro config
  Wire.beginTransmission(0x68);
  Wire.write(0x1B);
  Wire.write(0x10);
  if (Wire.endTransmission())
    return true;

  //set motion threshold
  Wire.beginTransmission(0x68);
  Wire.write(0x1F);
  Wire.write(0x02);  //2 mg per unit
  if (Wire.endTransmission())
    return true;
  //set motion det duration
  Wire.beginTransmission(0x68);
  Wire.write(0x20);
  Wire.write(0x05);  //1 ms per unit
  if (Wire.endTransmission())
    return true;
  //set ZERO motion threshold
  Wire.beginTransmission(0x68);
  Wire.write(0x21);
  Wire.write(0x04);  //2 mg per unit
  if (Wire.endTransmission())
    return true;
  //set ZERO motion det duration
  Wire.beginTransmission(0x68);
  Wire.write(0x22);
  Wire.write(0x02);  //64 ms per unit
  if (Wire.endTransmission())
    return true;

  //interrupt config
  Wire.beginTransmission(0x68);
  Wire.write(0x37);
  Wire.write(0xC0);
  if (Wire.endTransmission())
    return true;

  // interrupt enable as Movement detection
  Wire.beginTransmission(0x68);
  Wire.write(0x38);
  Wire.write(0x40);
  if (Wire.endTransmission())
    return true;

  return false;
}

void IRAM_ATTR movementDetectionISR() {
  xSemaphoreGiveFromISR(semaphoreMVDT, NULL);
}

void sensorProcess(void* pvParameters) {

  pinMode(INTERRUPT_PIN, INPUT_PULLUP);

  attachInterrupt(INTERRUPT_PIN, movementDetectionISR, FALLING);
  semaphoreMVDT = xSemaphoreCreateBinary();

  while (mpuInitMVDT())
    ;

  while (true) {
    xSemaphoreTake(semaphoreMVDT, portMAX_DELAY);
    //read interrupt status register to be sure MPU is connected and right interrupt is triggered
    Wire.beginTransmission(0x68);
    Wire.write(58);
    Wire.endTransmission();
    Wire.requestFrom(0x68, 1);
    unsigned long currentTime = millis();
    while (!Wire.available() && millis() - currentTime < 100)
      ;  //timeout after 100ms
    if (Wire.read() & 0x40) {
      //Motion detected
      deviceMode = DeviceMode::RUNNING;
      standbyTime = 60;
    }
  }
}

void loop() {}
