**Reference:** [Original Blog Post: https://fatherofmachines.blogspot.com/p/longan-nano-gd32vf103.html](https://fatherofmachines.blogspot.com/p/longan-nano-gd32vf103.html)

# Longan Nano GD32VF103

The **Longan Nano** is a development board sold by [seeedstudio](https://www.seeedstudio.com/Sipeed-Longan-Nano-RISC-V-GD32VF103CBT6-Development-Board-p-4205.html) for approximately 10€. It features a **GD32VF103** RISC-V core running at 108 MHz.

### Key Features
* **Processor:** GD32VF103 RV32IMAC+ECLIC Gigadevice Bumblebee Risc-V core at 108 MHz.
* **Screen:** 80x160 0.96'' OLED screen (SPI interface).
* **Connectivity:** USB-C Programmer, 3 UARTs, 3 SPI/I2C/I2S.
* **Timers:** 64-bit 27MHz SysTick timer, RTC timer, 1 advanced timer, and 4 timers with PWM, event counting, and quadrature decoding.
* **Analog:** 2 ADC, 2 DAC.
* **Other:** RGB LED, plastic case, buttons, and pinstrips.

### Why the Longan Nano?
The author of the blog post chose the Longan Nano to overcome the limitations of a previous 8-bit MCU. The goals for the project were to:
* Build a more precise motor controller.
* Integrate a screen for debugging and profiling.
* Learn about the **RISC-V** architecture.
* Practice with C++ features.

The Longan Nano met all these requirements while being an affordable option.

***

### Pinout and Peripherals

The following tables detail the pin configurations and peripheral mappings for the Longan Nano.

#### Longan Nano Pinout Table

| Pin | GPIO  | Function 1 | Function 2 | Function 3 | Function 4 | Function 5 | Function 6 |
|:---:|:-----:|:-----------|:-----------|:-----------|:-----------|:-----------|:-----------|
| 1   | **VBAT** | VBAT       |            |            |            |            |            |
| 2   | PC13  | RTCTAMPER  |            |            |            |            |            |
| 3   | PC14  | OSC32IN    |            |            |            |            |            |
| 4   | PC15  | OSC32OUT   |            |            |            |            |            |
| 5   | **PD0** | OSCIN      |            |            |            |            |            |
| 6   | **PD1** | OSCOUT     |            |            |            |            |            |
| 7   | VSSA  | NRST       |            |            |            |            |            |
| 8   | VDDA  |            |            |            |            |            |            |
| 9   | **PA0** | ADC0_IN0   | TIMER4_CH0 | USART1_CTS | TIMER0_CH0_ETI | WKUP |            |
| 10  | **PA1** | ADC0_IN1   | TIMER4_CH1 | USART1_RTS | TIMER1_CH1 |            |            |
| 11  | **PA2** | ADC0_IN2   | TIMER4_CH2 | USART1_TX  | TIMER1_CH2 |            |            |
| 12  | **PA3** | ADC0_IN3   | TIMER4_CH3 | USART1_RX  | TIMER1_CH3 |            |            |
| 13  | **PA4** | ADC0_IN4   | DAC0_OUT0  |            |            | SPI2_NSS | I2S2_WS    |
| 14  | **PA5** | ADC0_IN5   | DAC0_OUT1  |            |            | SPI0_SCK |            |
| 15  | **PA6** | ADC0_IN6   | TIMER0_CH0 | TIMER0_BRKIN | TIMER0_CH0_ON | SPI0_MISO |            |
| 16  | **PA7** | ADC0_IN7   | TIMER0_CH1 | TIMER0_CH1_ON | SPI0_MOSI |            |            |
| 17  | **PB0** | ADC0_IN8   | TIMER2_CH2 | TIMER0_CH2_ON |            |            |            |
| 18  | **PB1** | ADC0_IN9   | TIMER2_CH3 | TIMER0_CH3_ON |            |            |            |
| 19  | **PB2** | BOOT1      |            |            |            |            |            |
| 20  | **PB10** | USART2_TX  | I2C1_SCL   | TIMER1_CH2 |            |            |            |
| 21  | **PB11** | USART2_RX  | I2C1_SDA   | TIMER1_CH3 |            |            |            |
| 22  | VSS_1 |            |            |            |            |            |            |
| 23  | VDD_1 |            |            |            |            |            |            |
| 24  | **PB12** | USART2_CK  | TIMER0_BRKIN | SPI1_NSS | I2S1_WS    |            | CAN1_TX    |
| 25  | **PB13** | USART2_CTS | TIMER0_CH0_ON | SPI1_SCK | I2S1_CLK   |            | CAN1_RX    |
| 26  | **PB14** | USART2_RTS | TIMER0_CH1_ON | SPI1_MISO | I2S1_SD    |            |            |
| 27  | **PB15** | CK_OUT0    | TIMER0_CH2_ON | SPI1_MOSI | I2S1_SD    |            |            |
| 28  | **PA8** | USART0_CK  | TIMER0_CH0 | USBFS_DM   |            |            |            |
| 29  | **PA9** | USART0_TX  | TIMER0_CH1 | USBFS_DP   |            |            |            |
| 30  | **PA10** | USART0_RX  | TIMER0_CH2 | USBFS_ID   |            |            |            |
| 31  | **PA11** | USART0_CTS | TIMER0_CH3 | USBFS_DM   | CAN0_RX    |            |            |
| 32  | **PA12** | USART0_RTS | TIMER0_ETI | USBFS_DP   | CAN0_TX    |            |            |
| 33  | **PA13** | JTMS       |            |            |            |            |            |
| 34  | **PA14** | JTCK       |            |            |            |            |            |
| 35  | VSS_2 |            |            |            |            |            |            |
| 36  | VDD_2 |            |            |            |            |            |            |
| 37  | **PA15** | JTCK       | SPI2_NSS   | I2S2_WS    | TIMER1_CH0_ETI | SPI0_NSS |            |
| 38  | **PB3** | JTDI       | SPI2_SCK   | I2S2_CK    | TIMER1_CH1 | SPI0_SCK |            |
| 39  | **PB4** | NJTRST     | SPI2_MISO  | I2S2_SD    | TIMER1_CH2 | SPI0_MISO |            |
| 40  | **PB5** | I2S2_SD    | SPI2_MOSI  | I2C0_SMBA  | TIMER1_CH3 | SPI0_MOSI | CAN1_RX    |
| 41  | **PB6** | I2C0_SCL   | I2C0_SCL   | I2C0_SCL   | USART0_TX  |            |            |
| 42  | **PB7** | TIMER3_CH1 | I2C0_SDA   | I2C0_SDA   | USART0_RX  |            |            |
| 43  | **PB8** | BOOT0      |            |            |            |            |            |
| 44  | **PB9** | TIMER3_CH2 | I2C0_SCL   | CAN0_RX    |            |            |            |
| 45  | **PB9** | TIMER3_CH3 | I2C0_SDA   | CAN0_TX    |            |            |            |

### Peripherals by Channel

#### **JTAG**
| Function | Pins |
|:---|:---|
| **JTMS** | 33, PA13 |
| **JTCK** | 34, PA14; 37, PA15 |
| **JTDO** | 38, PB3 |
| **NJTRST** | 39, PB4 |

#### **ADC**
| Channel | Pins |
|:---|:---|
| **Channel 0** | 9, PA0 |
| **Channel 1** | 10, PA1 |
| **Channel 2** | 11, PA2 |
| **Channel 3** | 12, PA3 |
| **Channel 4** | 13, PA4 |
| **Channel 5** | 14, PA5 |
| **Channel 6** | 15, PA6 |
| **Channel 7** | 16, PA7 |
| **Channel 8** | 17, PB0 |
| **Channel 9** | 18, PB1 |

#### **DAC**
| Channel | Pins |
|:---|:---|
| **Channel 0** | 13, PA4 |
| **Channel 1** | 14, PA5 |

#### **UART**
| Function | Pins |
|:---|:---|
| **CK** | 28, PA8 |
| **TX** | 29, PA9; 11, PA2; 20, PB10; 41, PB6 |
| **RX** | 30, PA10; 12, PA3; 21, PB11; 42, PB7 |
| **CTS** | 31, PA11; 9, PA0; 24, PB12 |
| **RTS** | 32, PA12; 10, PA1; 26, PB14 |

#### **SPI**
| Function | Pins |
|:---|:---|
| **NSS** | 25, PB12; 13, PA4; 37, PA15 |
| **SCK** | 26, PB14; 14, PA5; 38, PB3 |
| **MISO** | 27, PB15; 15, PA6; 39, PB4 |
| **MOSI** | 28, PB15; 16, PA7; 40, PB5 |

#### **Timer**
| Function | Pins |
|:---|:---|
| **BRKIN** | 15, PA6; 24, PB12 |
| **ETI** | 9, PA0; 32, PA12; 37, PA15 |
| **CH0 I/O** | 9, PA0; 15, PA6; 25, PB13; 28, PA8 |
| **CH1 I/O** | 10, PA1; 16, PA7; 26, PB14; 29, PA9 |
| **CH2 I/O** | 11, PA2; 17, PB0; 30, PA10; 20, PB10 |
| **CH3 I/O** | 12, PA3; 18, PB1; 31, PA11; 21, PB11 |
| **CH0 ON** | 15, PA6; 25, PB13 |
| **CH1 ON** | 16, PA7; 26, PB14 |
| **CH2 ON** | 17, PB0; 27, PB15 |
| **CH3 ON** | 18, PB1; 28, PB15 |

#### **CAN**
| Function | Pins |
|:---|:---|
| **RX** | 31, PA11; 44, PB9; 40, PB5 |
| **TX** | 32, PA12; 45, PB9; 41, PB6 |

#### **I2C**
| Function | Pins |
|:---|:---|
| **SMBA** | 40, PB5 |
| **SCL** | 41, PB6; 20, PB10; 44, PB9 |
| **SDA** | 42, PB7; 21, PB11; 45, PB9 |

#### **I2S**
| Function | Pins |
|:---|:---|
| **WS** | 13, PA4; 24, PB12; 37, PA15 |
| **CLK** | 25, PB13; 38, PB3 |
| **SD** | 26, PB14; 27, PB15; 39, PB4; 40, PB5 |