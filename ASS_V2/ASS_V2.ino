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

#include <SPI.h>
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

TaskHandle_t SolderTask;
TaskHandle_t WiFiTask;

//for adafruit touchscreen library
//TouchScreen ts = TouchScreen(XP, YP, XM, YM, 340); // X+ to X- 340 Ohm
void setup(void) {
    delay(1000);
  xTaskCreatePinnedToCore(SolderProcess, "Solder Task", 10000, NULL, 1, &SolderTask, 1);
  xTaskCreatePinnedToCore(WiFiProcess, "WiFi Task", 10000, NULL, 0, &WiFiTask, 0);
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

void SolderProcess(void* pvParameters) {
  TFT_eSPI tft = TFT_eSPI();
  TouchPoint TouchScreen = TouchPoint(XP, YP, XM, YM);

  Button AugButton(&tft, 200, 30, 100, 70, 5);
  Button DecButton(&tft, 200, 140, 100, 70, 5);

  pinMode(SOLDERTEMP_PIN, INPUT);
  pinMode(SOLDER_OD, OUTPUT);

  tft.init();
  tft.setRotation(1);
  tft.setTextSize(4);

  //tft.fillScreen(TFT_BLACK);
  //tft.fillScreen(TFT_RED);
  //tft.fillScreen(TFT_GREEN);
  //tft.fillScreen(TFT_BLUE);
  //tft.fillScreen(TFT_BLACK);
  tft.fillScreen(TFT_BLACK);

  unsigned long solderTime = 0;
  unsigned long displayTime = 0;
  uint16_t goalTemp = 380;
  uint16_t actualTemp = 0;

  RingedBuffer<float, 50> temperatureBuffer;
  while (true) {
    if (millis() - displayTime > 10) {  //100 fps, every 10 ms
      displayTime = millis();
      uint16_t x = map(TouchScreen.getX(), 4000, 500, 0, 320);
      uint16_t y = map(TouchScreen.getY(), 500, 3800, 0, 240);

      x = x > 320 ? 0 : x;
      y = y > 240 ? 0 : y;

      if (AugButton.isPressed(x, y)) {
        AugButton.setColor(0xdfe0);
        goalTemp++;
      } else {
        AugButton.setColor(TFT_GREEN);
      }

      if (DecButton.isPressed(x, y)) {
        DecButton.setColor(0xfa80);
        goalTemp--;
      } else {
        DecButton.setColor(TFT_RED);
      }

      tft.setCursor(0, 10);
      tft.println("Set Temp:");
      tft.print(goalTemp);
      tft.println("   ");
      tft.println("Meas Temp:");
      tft.print(actualTemp);
      tft.println("   ");
    }

    if (millis() - solderTime > 20) {
      solderTime = millis();
      // deactivate Output in order to read the temperature
      digitalWrite(SOLDER_OD, LOW);
      // wait 1 millisecond to prevent bad measurements from em from switching voltage
      delay(5);
      // read the amplified temperature voltage and convert it into temperature

      temperatureBuffer.push((float)analogRead(SOLDERTEMP_PIN));

      actualTemp = map((uint16_t)temperatureBuffer.avg(), 0, 4095, 20, 600);

      // activate Soldering Iron if goalTemp is not yet reached
      if (actualTemp < goalTemp) {
        digitalWrite(SOLDER_OD, HIGH);
      } else {
        digitalWrite(SOLDER_OD, LOW);
      }
    }
  }
}

void WiFiProcess(void* pvParameters) {
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

void loop() {}
