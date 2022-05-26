#include <Adafruit_GFX.h>    // Core graphics library
#include <SPI.h>
#include <Adafruit_ILI9341.h>
#include "TouchScreen.h"
//#include "Adafruit_ILI9341_AS.h"

// These are the four touchscreen analog pins
#define YP A2  // must be an analog pin, use "An" notation!
#define XM A3  // must be an analog pin, use "An" notation!
#define YM 9   // can be any digital pin
#define XP 8   // can be any digital pin
// The display uses hardware SPI, plus #9 & #10
#define TFT_CS 10
#define TFT_DC 9

Adafruit_ILI9341 tft = Adafruit_ILI9341(TFT_CS, TFT_DC);

TouchScreen ts = TouchScreen(XP, YP, XM, YM, 340); //X+ to X- 340 Ohm

// This is calibration data for the raw touch data to the screen coordinates
#define TS_MINX 150
#define TS_MINY 120
#define TS_MAXX 920
#define TS_MAXY 940

#define MINPRESSURE 10
#define MAXPRESSURE 1000


// Size of the color selection boxes and the paintbrush size
#define BOXSIZE 40
#define PENRADIUS 3

#define SOLDER_OD 6

int actualTemp;
int goalTemp = 380;

void setup(void) {

  Serial.begin(9600);
  while (!Serial);     // used for leonardo debugging

  Serial.println(F("Touch Paint!"));

  tft.begin();
  tft.setRotation(3);
  tft.fillScreen(ILI9341_BLACK);

  // make the color selection boxes

  tft.fillRect(240, 20, 60, 80, ILI9341_GREEN);
  tft.fillRect(240, 140, 60, 80, ILI9341_RED);
  tft.drawLine(220, 0, 220, 240, ILI9341_WHITE);
  tft.drawLine(220, 120, 320, 120, ILI9341_WHITE);

  // select the current color 'red'
  tft.setTextSize(3);
  tft.setCursor(20, 20);
  tft.println("GOAL TEMP:");
  tft.setCursor(20, 140);
  tft.println("ACT. TEMP:");
  tft.setCursor(192, 60);
  tft.println("°C");
  tft.setCursor(192, 180);
  tft.println("°C");
}


void loop()
{
  // Retrieve a point  
  TSPoint p = ts.getPoint();

  // Scale from ~0->1000 to tft.width using the calibration #'s
  int x = map(p.y, TS_MINY, TS_MAXY, tft.width(), 0);
  int y = map(p.x, TS_MINX, TS_MAXX, 0, tft.height());
  //activate Soldering Iron if goalTemp is not yet reached
  if (actualTemp < goalTemp) {
    //71/255 = 25%
    analogWrite(SOLDER_OD, 71);
  }else{
    digitalWrite(SOLDER_OD, LOW);
  }

  // we have some minimum pressure we consider 'valid'
  // pressure of 0 means no pressing!
  if (p.z < MINPRESSURE || p.z > MAXPRESSURE) {
    return;
  }


  if (x > 240 && x < 300) {
    if (y > 20 && y < 100) {
      if (goalTemp < 450) {
        goalTemp++;
        tft.fillRect(20, 60, 200, 40, ILI9341_BLACK);
      }
      tft.fillRect(240, 20, 60, 80, ILI9341_YELLOW);
      delay(100);
      tft.fillRect(240, 20, 60, 80, ILI9341_GREEN);
    }
    else if (y > 140 && y < 220) {
      if (goalTemp > 200) {
        goalTemp--;
        tft.fillRect(20, 60, 200, 40, ILI9341_BLACK);
      }
      tft.fillRect(240, 140, 60, 80, ILI9341_YELLOW);
      delay(100);
      tft.fillRect(240, 140, 60, 80, ILI9341_RED);
    }
  }

  tft.setCursor(20, 60);
  tft.print(goalTemp);
}
