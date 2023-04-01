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

const char* ssid = /*"UPC14A647F"*/ "Coldspot";
const char* password = /*"wyruhcm5yXkQ"*/ "Hotstoppassword";

#define YP 1
#define YM 14
#define XP 13
#define XM 2
#define TFT_GREY 0x5AEB


#define SOLDERTEMP_PIN 8
#define SOLDER_OD 9

TFT_eSPI tft = TFT_eSPI();  // Invoke custom library

TouchPoint TouchScreen = TouchPoint(XP, YP, XM, YM);

//for adafruit touchscreen library
//TouchScreen ts = TouchScreen(XP, YP, XM, YM, 340); // X+ to X- 340 Ohm

float sx = 0, sy = 1, mx = 1, my = 0, hx = -1, hy = 0;  // Saved H, M, S x & y multipliers
float sdeg = 0, mdeg = 0, hdeg = 0;
uint16_t osx = 120, osy = 120, omx = 120, omy = 120, ohx = 120, ohy = 120;  // Saved H, M, S x & y coords
uint16_t x0 = 0, x1 = 0, yy0 = 0, yy1 = 0;
uint32_t targetTime = 0;  // for next 1 second timeout

static uint8_t conv2d(const char* p);                                                 // Forward declaration needed for IDE 1.6.x
uint8_t hh = conv2d(__TIME__), mm = conv2d(__TIME__ + 3), ss = conv2d(__TIME__ + 6);  // Get H, M, S from compile time

bool initial = 1;

void setup(void) {
  //Serial.begin(115200);
  pinMode(SOLDERTEMP_PIN, INPUT);
  pinMode(SOLDER_OD, OUTPUT);

  tft.init();
  tft.setRotation(0);

  //tft.fillScreen(TFT_BLACK);
  //tft.fillScreen(TFT_RED);
  //tft.fillScreen(TFT_GREEN);
  //tft.fillScreen(TFT_BLUE);
  //tft.fillScreen(TFT_BLACK);
  tft.fillScreen(TFT_GREY);

  tft.setTextColor(TFT_WHITE, TFT_GREY);  // Adding a background colour erases previous text automatically

  // Draw clock face
  tft.fillCircle(120, 120, 118, TFT_GREEN);
  tft.fillCircle(120, 120, 110, TFT_BLACK);

  // Draw 12 lines
  for (int i = 0; i < 360; i += 30) {
    sx = cos((i - 90) * 0.0174532925);
    sy = sin((i - 90) * 0.0174532925);
    x0 = sx * 114 + 120;
    yy0 = sy * 114 + 120;
    x1 = sx * 100 + 120;
    yy1 = sy * 100 + 120;

    tft.drawLine(x0, yy0, x1, yy1, TFT_GREEN);
  }

  // Draw 60 dots
  for (int i = 0; i < 360; i += 6) {
    sx = cos((i - 90) * 0.0174532925);
    sy = sin((i - 90) * 0.0174532925);
    x0 = sx * 102 + 120;
    yy0 = sy * 102 + 120;
    // Draw minute markers
    tft.drawPixel(x0, yy0, TFT_WHITE);

    // Draw main quadrant dots
    if (i == 0 || i == 180) tft.fillCircle(x0, yy0, 2, TFT_WHITE);
    if (i == 90 || i == 270) tft.fillCircle(x0, yy0, 2, TFT_WHITE);
  }

  tft.fillCircle(120, 121, 3, TFT_WHITE);

  // Draw text at position 120,260 using fonts 4
  // Only font numbers 2,4,6,7 are valid. Font 6 only contains characters [space] 0 1 2 3 4 5 6 7 8 9 : . - a p m
  // Font 7 is a 7 segment font and only contains characters [space] 0 1 2 3 4 5 6 7 8 9 : .
  //tft.drawCentreString("Time flies",120,260,4);

  targetTime = millis() + 1000;
  initOTA();
}
unsigned long touchTime = 0;
  static unsigned long timeComparison = millis();

void loop() {
  ArduinoOTA.handle();
  uint8_t goalTemp = 380;

  if (millis() - timeComparison > 100) {
    timeComparison = millis();
    // deactivate Output in order to read the temperature
    digitalWrite(SOLDER_OD, LOW);
    // wait 5 milliseconds to prevent bad measurements
    delay(2);
    // read the amplified temperature voltage and convert it into temperature
    uint8_t actualTemp = map(analogRead(SOLDERTEMP_PIN), 0, 4096, 43, 650);
    // activate Soldering Iron if goalTemp is not yet reached
    if (actualTemp < goalTemp) {
      digitalWrite(SOLDER_OD, HIGH);
    } else {
      digitalWrite(SOLDER_OD, LOW);
    }
  }

  uint16_t x = TouchScreen.getX();
  uint16_t y = TouchScreen.getY();

  if (touchTime < millis()) {
    touchTime += 50;

    tft.setCursor(0, 10);
    tft.print(x);
    tft.println("   ");
    tft.print(y);
    tft.println("   ");
  }
  //TSPoint p = ts.getPoint();
  //Serial.printf("x: %i, y: %i\n", x, y);
  if (targetTime < millis()) {
    targetTime += 1000;
    ss++;  // Advance second
    if (ss == 60) {
      ss = 0;
      mm++;  // Advance minute0
      if (mm > 59) {
        mm = 0;
        hh++;  // Advance hour
        if (hh > 23) {
          hh = 0;
        }
      }
    }

    // Pre-compute hand degrees, x & y coords for a fast screen update
    sdeg = ss * 6;                      // 0-59 -> 0-354
    mdeg = mm * 6 + sdeg * 0.01666667;  // 0-59 -> 0-360 - includes seconds
    hdeg = hh * 30 + mdeg * 0.0833333;  // 0-11 -> 0-360 - includes minutes and seconds
    hx = cos((hdeg - 90) * 0.0174532925);
    hy = sin((hdeg - 90) * 0.0174532925);
    mx = cos((mdeg - 90) * 0.0174532925);
    my = sin((mdeg - 90) * 0.0174532925);
    sx = cos((sdeg - 90) * 0.0174532925);
    sy = sin((sdeg - 90) * 0.0174532925);

    if (ss == 0 || initial) {
      initial = 0;
      // Erase hour and minute hand positions every minute
      tft.drawLine(ohx, ohy, 120, 121, TFT_BLACK);
      ohx = hx * 62 + 121;
      ohy = hy * 62 + 121;
      tft.drawLine(omx, omy, 120, 121, TFT_BLACK);
      omx = mx * 84 + 120;
      omy = my * 84 + 121;
    }

    // Redraw new hand positions, hour and minute hands not erased here to avoid flicker
    tft.drawLine(osx, osy, 120, 121, TFT_BLACK);
    osx = sx * 90 + 121;
    osy = sy * 90 + 121;
    tft.drawLine(osx, osy, 120, 121, TFT_RED);
    tft.drawLine(ohx, ohy, 120, 121, TFT_WHITE);
    tft.drawLine(omx, omy, 120, 121, TFT_WHITE);
    tft.drawLine(osx, osy, 120, 121, TFT_RED);

    tft.fillCircle(120, 121, 3, TFT_RED);
  }
}

static uint8_t conv2d(const char* p) {
  uint8_t v = 0;
  if ('0' <= *p && *p <= '9')
    v = *p - '0';
  return 10 * v + *++p - '0';
}

void initOTA() {

  //Serial.begin(115200);
  //Serial.println("Booting");
  WiFi.mode(WIFI_STA);
  WiFi.begin(ssid, password);
  while (WiFi.waitForConnectResult() != WL_CONNECTED) {
    //Serial.println("Connection Failed! Rebooting...");
    delay(5000);
    ESP.restart();
  }

  // Port defaults to 3232
  // ArduinoOTA.setPort(3232);

  // Hostname defaults to esp3232-[MAC]
  ArduinoOTA.setHostname("AwesomeSolderingStation");

  // No authentication by default
  ArduinoOTA.setPassword("esp32");

  // Password can be set with it's md5 value as well
  // MD5(admin) = 21232f297a57a5a743894a0e4a801fc3
  // ArduinoOTA.setPasswordHash("21232f297a57a5a743894a0e4a801fc3");

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
  /*
  Serial.println("Ready");
  Serial.print("IP address: ");
  Serial.println(WiFi.localIP());*/
}