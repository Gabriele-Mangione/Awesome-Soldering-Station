# ✨ Awesome Soldering Station ✨

---

## Project Overview 💡

This project introduces an **open-source DIY soldering station** featuring **USB-C Power Delivery** and an intuitive **touchscreen interface**. The goal is to provide a very affordable, highly functional, modifiable, and robust soldering solution built with modern embedded technologies for the DIY community!

![](https://github.com/Gabriele-Mangione/Awesome-Soldering-Station/blob/Rust/images/finishedProduct.jpg?raw=true)

A key design feature is its compatibility with **jack connector soldering tips**, specifically those used by Weller soldering stations, ensuring broad tip availability and performance.

While the primary development is focused on the **Rust** codebase for the latest iteration, a fully functional C++ version exists on the `2.0` branch, serving as a tested foundation.

---

## Technical Architecture ⚙️

The station's intelligence is distributed across two microcontrollers:

* **ESP32-S3 (Station Mainboard):** This microcontroller drives the core functionality of the station. It's programmed in **Rust** utilizing the `esp-hal` open-source framework. Firmware updates are streamlined and accessible, allowing for easy modification and programming directly through the **USB-C port** used for power delivery.

* **ATtiny24 (Soldering Handle):** A compact microcontroller embedded within the soldering handle, responsible for precise temperature sensing and control. It's programmed in **C++** within the Arduino environment and uploaded via an external ISP.

---

## Hardware Design & Construction 🛠️

The hardware is meticulously designed in **Altium Designer** and comprises two distinct printed circuit boards:

* **Station Mainboard:** This board houses the ESP32-S3, power delivery circuitry, display interface, and other essential components for the station's operation.
![](https://github.com/Gabriele-Mangione/Awesome-Soldering-Station/blob/Rust/images/StationBoard.jpg?raw=true)

* **Soldering Handle Board:** A compact PCB designed to fit within the handle, facilitating sensor integration and handle-specific controls, crucial for the jack connector tips.
![](https://github.com/Gabriele-Mangione/Awesome-Soldering-Station/blob/Rust/images/HandleBoard.jpg?raw=true)

These two boards are interconnected by a cable, chosen for both its both comfort and flexibility during use, while ensuring reliable high-current power transmission without impedance issues. (I tested a lot of different ones)

The enclosure for the main station and the handle itself are both **3D printed**. This approach allows for a custom, ergonomic design and simplifies prototyping and production.
![](https://github.com/Gabriele-Mangione/Awesome-Soldering-Station/blob/Rust/images/3DModels.png?raw=true)


## Development Status & Technologies Used 💻

* **Hardware Design:** Altium Designer
* **Station Firmware:** Embedded Rust with `esp-hal` (active development branch)
* **Handle Firmware:** C++ using the Arduino environment
* **Case & Handle:** 3D Printed, designed with Blender
* **Tip Compatibility:** Designed for **Weller jack connector soldering tips**

---

## Getting Started (Prerequisites) 📝

#### For the Hardware:
If you take the DIY route:
* Order PCBs from your favourite PCB manufacturer (I use AISLER and JLCPCB)
* Order Parts from my Digi-Key Lists:
    * Handle: https://www.digikey.ch/en/mylists/list/VL3OIYCQ1Z
    * Station: https://www.digikey.ch/en/mylists/list/MW1GWRHM6E
* 3D print the files in the 3D-Models Folder
* Solder and mount everything together!

If you don't have the needed tools, you can opt to buy the finished product directly from me.
If so, feel free to contact me at Gabriele.Mangione@hotmail.com
#### For ESP32-S3 Firmware (Rust):
* **`esp-hal` Toolchain:** Refer to [https://docs.esp-rs.org/book/](https://docs.esp-rs.org/book/) for detailed setup instructions.
    * **Note:** For Windows users, Rust must be installed via `rustup` for compatibility.

* TODO: insert libs

#### For ATtiny24 Firmware (C++):
* **Arduino IDE** or **Arduino CLI**
* Appropriate ATtiny24 board definitions for the Arduino environment.