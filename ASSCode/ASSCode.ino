
#include <Adafruit_GFX.h> // Core graphics library
#include <SPI.h>
#include <Adafruit_ILI9341.h>
#include "TouchScreen.h"

// Touchscreen pins
#define YP A2
#define XM A3
#define YM 9
#define XP 8

// The display uses hardware SPI, plus #9 & #10
#define TFT_CS 10
#define TFT_DC 9

Adafruit_ILI9341 tft = Adafruit_ILI9341(TFT_CS, TFT_DC);

TouchScreen ts = TouchScreen(XP, YP, XM, YM, 340); // X+ to X- 340 Ohm

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

int actualTemp = 0;
int goalTemp = 380;
bool presetsMode = false;
unsigned long timeComparison = 0;

void setup(void)
{
  tft.begin();
  tft.setRotation(3);
  tft.fillScreen(ILI9341_BLACK);
  drawWorkingMode();
}

/**
 * @brief Draw the preset select screen on the display
 *
 */
void drawPresetMode()
{
  tft.fillRect(25, 30, 120, 80, ILI9341_DARKGREY);
  tft.drawRect(25, 30, 120, 80, ILI9341_LIGHTGREY);
  tft.fillRect(25, 140, 120, 80, ILI9341_DARKGREY);
  tft.drawRect(25, 140, 120, 80, ILI9341_LIGHTGREY);
  tft.fillRect(170, 30, 120, 80, ILI9341_DARKGREY);
  tft.drawRect(170, 30, 120, 80, ILI9341_LIGHTGREY);
  tft.fillRect(170, 140, 120, 80, ILI9341_DARKGREY);
  tft.drawRect(170, 140, 120, 80, ILI9341_LIGHTGREY);

  tft.setTextSize(2);
  tft.setTextColor(ILI9341_BLACK);
  tft.setCursor(67, 62);
  tft.write("320");
  tft.setCursor(212, 62);
  tft.write("350");
  tft.setCursor(67, 172);
  tft.write("380");
  tft.setCursor(212, 172);
  tft.write("400");

  tft.fillRect(300, 105, 20, 30, ILI9341_DARKGREY);
  tft.drawRect(300, 105, 20, 30, ILI9341_LIGHTGREY);
}

/**
 * @brief Draw the temperature screen on the display
 *
 */
void drawWorkingMode()
{

  tft.fillRect(240, 20, 60, 80, ILI9341_GREEN);
  tft.fillRect(240, 140, 60, 80, ILI9341_RED);
  tft.drawLine(220, 0, 220, 240, ILI9341_WHITE);
  tft.drawLine(220, 120, 320, 120, ILI9341_WHITE);

  tft.setTextColor(ILI9341_WHITE);
  tft.setTextSize(3);
  tft.setCursor(20, 20);
  tft.println("GOAL TEMP:");
  tft.setCursor(20, 140);
  tft.println("ACT. TEMP:");
  tft.setCursor(150, 60);
  tft.println("C");
  tft.setCursor(150, 180);
  tft.println("C");

  //the degree sign is not available on the display font, so a smaller 'o' is instead written
  tft.setTextSize(1);
  tft.setCursor(143, 176);
  tft.println("o");
  tft.setCursor(143, 56);
  tft.println("o");

  tft.fillRect(0, 105, 20, 30, ILI9341_DARKGREY);
  tft.drawRect(0, 105, 20, 30, ILI9341_LIGHTGREY);
}

void loop()
{
  //do once every second
  if (millis() - timeComparison > 1000)
  {
    timeComparison = millis();
    // deactivate Output in order to read the temperature
    digitalWrite(SOLDER_OD, LOW);
    // wait 5 milliseconds to prevent bad measurements
    delay(5);
    // read the amplified temperature voltage and convert it into temperature
    actualTemp = map(analogRead(A5), 0, 1024, 43, 650);
    // activate Soldering Iron if goalTemp is not yet reached
    if (actualTemp < goalTemp)
    {
      digitalWrite(SOLDER_OD, HIGH);
    }
    else
    {
      digitalWrite(SOLDER_OD, LOW);
    }

    //reload the shown actual temperature 
    if (!presetsMode)
    {
      tft.fillRect(20, 180, 100, 24, ILI9341_BLACK);
      tft.setTextColor(ILI9341_WHITE);
      tft.setTextSize(3);
      tft.setCursor(20, 180);
      tft.print(actualTemp);
    }
  }

  //read touch breakout input
  TSPoint p = ts.getPoint();
  int x = map(p.y, TS_MINY, TS_MAXY, tft.width(), 0);
  p.y = map(p.x, TS_MINX, TS_MAXX, 0, tft.height());
  p.x = x;
  /*
    if (p.z < MINPRESSURE || p.z > MAXPRESSURE) {
      return;
    }
  */
 //if display is in the preset select screen, device starts with presetsMode = 0
  if (presetsMode)
  {
    if (p.x > 25 && p.x < 145)
    {
      if (p.y > 30 && p.y < 110)
      {
        //if preset button 1 is pressed
        drawPresetMode();
        tft.fillRect(26, 31, 118, 78, ILI9341_BLUE);
        goalTemp = 320;
        tft.setTextSize(2);
        tft.setTextColor(ILI9341_BLACK);
        tft.setCursor(67, 62);
        tft.write("320");
      }
      else if (p.y > 140 && p.y < 220)
      {
        //if preset button 3 is pressed
        drawPresetMode();
        tft.fillRect(26, 141, 118, 78, ILI9341_BLUE);
        goalTemp = 380;
        tft.setTextSize(2);
        tft.setTextColor(ILI9341_BLACK);
        tft.setCursor(67, 172);
        tft.write("380");
      }
    }
    else if (p.x > 170 && p.x < 290)
    {
      if (p.y > 30 && p.y < 110)
      {
        //if preset button 2 is pressed
        drawPresetMode();
        tft.fillRect(171, 31, 118, 78, ILI9341_BLUE);
        goalTemp = 350;
        tft.setTextSize(2);
        tft.setTextColor(ILI9341_BLACK);
        tft.setCursor(212, 62);
        tft.write("350");
      }
      else if (p.y > 140 && p.y < 220)
      {
        //if preset button 4 is pressed
        drawPresetMode();
        tft.fillRect(171, 141, 118, 78, ILI9341_BLUE);
        goalTemp = 400;
        tft.setTextSize(2);
        tft.setTextColor(ILI9341_BLACK);
        tft.setCursor(212, 172);
        tft.write("400");
      }
    }
    //if the 'back' button is pressed, go to the 'working mode'
    if (p.x > 280 && p.x < 320 && p.y > 90 && p.y < 150)
    {
      tft.fillScreen(ILI9341_BLACK);
      drawWorkingMode();
      presetsMode = false;
    }
  }
  //if device is in 'working mode' 
  else 
  {
    if (p.x > 240 && p.x < 300)
    {
      if (p.y > 20 && p.y < 100)
      {
        //if the green button is pressed
        if (goalTemp < 450)
        {
        //the max value of goalTemp is 450
          goalTemp++;
          tft.fillRect(20, 60, 100, 24, ILI9341_BLACK);
        }
        tft.fillRect(240, 20, 60, 80, ILI9341_YELLOW);
        delay(100);
      }
      else if (p.y > 140 && p.y < 220)
      {
        //if the red button is pressed
        if (goalTemp > 200)
        {
        //the min value of goalTemp is 200
          goalTemp--;
          tft.fillRect(20, 60, 100, 24, ILI9341_BLACK);
        }
        tft.fillRect(240, 140, 60, 80, ILI9341_YELLOW);
        delay(100);
      }
    }
    else
    {
      tft.fillRect(240, 140, 60, 80, ILI9341_RED);
      tft.fillRect(240, 20, 60, 80, ILI9341_GREEN);
    }
    //reload the shown goal temperature
    tft.setTextColor(ILI9341_WHITE);
    tft.setTextSize(3);
    tft.setCursor(20, 60);
    tft.print(goalTemp);

    // if the 'presets button' is pressed, go to preset mode
    if (p.x > 0 && p.x < 40 && p.y > 90 && p.y < 150)
    {
      tft.fillScreen(ILI9341_BLACK);

      drawPresetMode();
      presetsMode = true;
    }
  }
}
