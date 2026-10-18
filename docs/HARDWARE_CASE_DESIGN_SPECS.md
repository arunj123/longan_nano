# Hardware Specifications & Mechanical Enclosure Design Reference
**Sipeed Longan Nano (GD32VF103CBT6) & INA219 Current Sensor Breakout Modules**

**Document Target**: Machine-readable & AI-directed CAD/Enclosure Engineering Reference  
**Primary Purpose**: Directing AI agents and parametric CAD tools (OpenSCAD, FreeCAD, CadQuery, Blender) to design 3D-printable cases, brackets, and snap-fit enclosures.  
**Datasheet Source**: [sipeed_longan_nano_datasheet_102991302.pdf](sipeed_longan_nano_datasheet_102991302.pdf) (Seeed Studio SKU 102991302)  
**Related Firmware Project**: [`prj_current_monitor`](../prj_current_monitor/README.md) & [`longan_nano_ina219_case.scad`](../prj_current_monitor/enclosure/longan_nano_ina219_case.scad)

---

## 1. Reference Media & Visual Assets

| Asset Name | File Link | Description |
| :--- | :--- | :--- |
| **Datasheet PDF** | [sipeed_longan_nano_datasheet_102991302.pdf](sipeed_longan_nano_datasheet_102991302.pdf) | Original manufacturer specification document (Seeed SKU 102991302). |
| **Board Mechanical Drawing** | [longan_nano_mechanical_drawing.jpg](longan_nano_mechanical_drawing.jpg) | Dimensioned mechanical blueprint with pin spacing, length, and notch specs. |
| **Physical Scale Reference** | [longan_nano_ruler_scale.jpg](longan_nano_ruler_scale.jpg) | High-res photo of Longan Nano with mm and inch precision ruler scales. |
| **Longan Nano Top View** | [longan_nano_top_view.jpg](longan_nano_top_view.jpg) | Top-down view showing LCD glass, bezel, buttons, and pin labels. |
| **Longan Nano Bottom View** | [longan_nano_bottom_view.jpg](longan_nano_bottom_view.jpg) | Solder-side view showing MicroSD (TF) socket and JTAG pads. |
| **Acrylic Case Profile View** | [longan_nano_acrylic_case_side_view.jpg](longan_nano_acrylic_case_side_view.jpg) | Side profile of board inside stock case illustrating vertical stackup. |
| **INA219 Blue Module Kit** | [ina219_blue_breakout_kit.jpg](ina219_blue_breakout_kit.jpg) | Blue INA219 breakout, 6-pin header, and 3.5 mm 2-pin screw terminal block. |
| **INA219 Blue Top View** | [ina219_blue_top_view.jpg](ina219_blue_top_view.jpg) | Top view showing R100 (0.1Ω) 2512 shunt, IC, A0/A1 jumpers, terminal pads. |
| **INA219 Blue Bottom View** | [ina219_blue_bottom_view.jpg](ina219_blue_bottom_view.jpg) | Bottom silkscreen view (0–26V, 3.2A, 3–5V) and 4 corner mounting holes. |

---

## 2. Sipeed Longan Nano (GD32VF103CBT6) Hardware & Geometry

### 2.1 Mechanical Envelope & PCB Dimensions
- **Overall PCB Length**: `46.10 mm`
- **Overall PCB Width**: `20.00 mm`
- **PCB Substrate Thickness**: `1.00 mm` (standard red FR4)
- **Sub-segment Lengths**:
  - Far edge to center notch: `19.60 mm`
  - Center notch width: `3.80 mm` (semi-circular side index cutouts)
  - Center notch to USB edge: `22.60 mm` (19.60 + 3.80 + 22.60 = 46.00 ~ 46.10 mm)
- **Pin Header Footprint**:
  - Dual-row DIP form factor: standard `2.54 mm` (`100 mil`) pitch
  - Row-to-row spacing: `17.78 mm` (`700 mil` / `0.70 in`), fits standard breadboard center trough
  - Pin count: 34 pins total (2 rows of 17 pins)
  - Left group: 8 pins per row
  - Center notch: occupies gap equivalent to ~1 pin pitch
  - Right group: 9 pins per row

### 2.2 Vertical Component Stackup (Z-Axis)
- **Top Side Components (above PCB surface)**:
  - **USB Type-C Receptacle**: `3.16 mm` height, `8.94 mm` width, protrudes `~2.80–3.00 mm` beyond PCB right edge.
  - **0.96" IPS RGB LCD Module**: `~2.80–3.00 mm` total height above PCB (includes foam adhesive mounting tape).
  - **Tactile SMD Buttons (BOOT0 & RESET)**: `~2.20–2.50 mm` button body height; actuator nub travel `~0.30 mm`.
- **Bottom Side Components (below PCB surface)**:
  - **MicroSD (TF) Card Socket**: `1.80 mm` thickness, `11.50 mm` width, `12.00 mm` length. Push-pull slot opening faces towards the left edge (opposite USB-C).
  - **Soldered Pin Headers**: Standard straight headers extend `~6.00 mm` below PCB (+`2.50 mm` black plastic spacer if installed). If headers are omitted/soldered flush, bottom clearance requirement is `~2.20 mm`.
- **Minimum Enclosure Internal Clearance**:
  - Enclosure cavity with pin headers: `~12.50 mm` minimum Z height.
  - Enclosure cavity without headers (flush-soldered or low-profile): `~6.50 mm` minimum Z height.

### 2.3 0.96" IPS LCD Display Specifications & Case Cutout
- **Controller & Interface**: ST7735 SPI LCD (SPI0 peripheral, 8-pin 0.5 mm pitch FPC flex connector).
- **Resolution**: `160 × 80` RGB pixels.
- **Active Display Window Area**: `21.70 mm` length × `10.80 mm` width (0.96-inch diagonal).
- **Outer Glass / Bezel Dimensions**: `28.00 mm` length × `14.50 mm` width × `1.40 mm` glass thickness.
- **Recommended Case Window Opening**: `24.50 mm` length × `14.00 mm` width.
- **CRITICAL FPC Cable Relief Requirement**:
  - The flexible ribbon cable wraps tightly around the left edge of the display glass to connect to the solder pads.
  - **Do NOT clamp or apply downward pressure** on the folded ribbon edge.
  - The enclosure top lid MUST feature an internal recess / relief pocket of at least `4.0 mm × 6.0 mm` with `1.80 mm` minimum depth over the flex fold to prevent trace fatigue or shearing.

### 2.4 Button, Connector & Peripheral Cutout Positions
- **USB Type-C Port**:
  - Location: Right edge, centered on the 20.0 mm board width.
  - Recommended Wall Cutout: `11.0 mm` width × `5.0 mm` height (provides clearance for USB-C cable overmold shroud).
- **Tactile Buttons**:
  - `BOOT0` (PA8): Located at top-right, adjacent to USB-C port (`X ≈ 42.0 mm`, `Y ≈ 17.0 mm` from bottom-left origin).
  - `RESET` (NRST): Located at bottom-right, adjacent to USB-C port (`X ≈ 42.0 mm`, `Y ≈ 3.0 mm` from bottom-left origin).
  - Recommended Button Cutouts: Circular holes `Ø 3.50–4.50 mm` or 3D-printed flexible compliant pushers.
- **MicroSD (TF) Card Slot**:
  - Location: Bottom side of PCB, opening faces left edge.
  - Recommended Card Eject Cutout: `13.0 mm` width × `2.50–3.00 mm` height on the left end-wall.
- **JTAG Test Header**:
  - 4 gold edge pads at the left edge (`JTDO`, `JTDI`, `JTCK`, `JTMS`). Standard 2.54 mm pitch; usually kept internal unless external debugging port is needed.

### 2.5 Complete Pinout Mapping (Longan Nano)
```
                  TOP ROW (Edge 1)
Pin  Name   Function / Description
---  ----   -------------------------------------------------
 1   G      Ground (GND)
 2   G      Ground (GND)
 3   B9     GPIOB.9 / I2C0_SDA (Remap) / CAN_TX / TIMER3_CH3
 4   B8     GPIOB.8 / I2C0_SCL (Remap) / CAN_RX / TIMER3_CH2
 5   B7     GPIOB.7 / I2C0_SDA (Default INA219 SDA)
 6   B6     GPIOB.6 / I2C0_SCL (Default INA219 SCL)
 7   B5     GPIOB.5 / SPI2_MOSI / I2S2_SD
 8   B4     GPIOB.4 / SPI2_MISO
---  [CENTER NOTCH: 3.8 mm] --------------------------
 9   A12    GPIOA.12 / USB_DP (D+) / USART0_RTS / CAN_TX
10   A11    GPIOA.11 / USB_DM (D-) / USART0_CTS / CAN_RX
11   A8     GPIOA.8 / BOOT0 User Button (Active-High)
12   B15    GPIOB.15 / SPI1_MOSI (SD Card MOSI)
13   B14    GPIOB.14 / SPI1_MISO (SD Card MISO w/ pull-up)
14   B13    GPIOB.13 / SPI1_SCK (SD Card SCK)
15   B12    GPIOB.12 / SPI1_NSS (SD Card CS / PB12)
16   3V3    3.3V Regulated Power Output (from onboard LDO)
17   5V     5.0V Raw Power Input (from USB VBUS)

                 BOTTOM ROW (Edge 2)
Pin  Name   Function / Description
---  ----   -------------------------------------------------
 1   VB     V_BAT (Battery backup domain)
 2   C15    GPIOC.15 / 32.768 kHz RTC Crystal Output
 3   C14    GPIOC.14 / 32.768 kHz RTC Crystal Input
 4   C13    GPIOC.13 / Red Onboard LED (Active-Low)
 5   A0     GPIOA.0 / ADC01_IN0 / TIMER1_CH0
 6   A1     GPIOA.1 / ADC01_IN1 / Green Onboard LED (Active-Low)
 7   A2     GPIOA.2 / ADC01_IN2 / Blue Onboard LED (Active-Low)
 8   A3     GPIOA.3 / ADC01_IN3 / USART1_RX
---  [CENTER NOTCH: 3.8 mm] --------------------------
 9   A4     GPIOA.4 / SPI0_NSS / DAC0_OUT
10   A5     GPIOA.5 / SPI0_SCK (LCD SCK)
11   A6     GPIOA.6 / SPI0_MISO / ADC01_IN6
12   A7     GPIOA.7 / SPI0_MOSI (LCD MOSI)
13   B0     GPIOB.0 / LCD DC (Data/Command select)
14   B1     GPIOB.1 / LCD RST (Reset)
15   B10    GPIOB.10 / LCD Backlight / USART2_TX / I2C1_SCL
16   B11    GPIOB.11 / USART2_RX / I2C1_SDA
17   RST    MCU Hardware Reset (Active-Low, RESET button)
```

---

## 3. Current Sensor Breakout Modules (INA219)

### 3.1 Variant A: Blue INA219 Breakout Board (Standard with Screw Terminal)
*High-side DC current, bus voltage, and power monitor with screw terminal block.*

#### 3.1.1 Mechanical Dimensions & Mounting
- **PCB Outer Dimensions**: `25.50 mm` length × `22.30 mm` width (nominal `0.90" × 0.80"`).
- **PCB Thickness**: `1.60 mm`.
- **Mounting Holes**: 4 corner through-holes, `Ø 2.80–3.00 mm` (designed for M2 or M2.5 screws/standoffs).
  - Corner Hole Spacing: `~21.0 mm` (lengthwise center-to-center) × `~17.8 mm` (widthwise center-to-center).
- **Z-Axis Height Breakdown**:
  - Bare board + SMD components: `~3.20 mm`.
  - With 2-pin 3.5 mm Screw Terminal: `~12.00 mm` total height above PCB.
  - With straight 6-pin male header: `~8.50 mm` pin height below PCB.

#### 3.1.2 Connectors & Interfaces
- **Screw Terminal Block (Power / Load Path)**:
  - 2-pin, `3.50 mm` pitch rising-cage screw terminal (green).
  - Terminal footprint: `10.00 mm` width × `8.00 mm` depth × `10.00 mm` height.
  - Pins: `VIN+` (Power supply input) and `VIN-` (Load connection).
  - Compatible Wire: 14 to 26 AWG solid or stranded wire.
  - Case Opening: Requires side wall cutout `11.0 mm` wide × `9.0 mm` high, with top access holes (`Ø 4.0 mm`) for terminal screwdriver blades.
- **6-Pin Logic & Telemetry Header**:
  - 1 × 6 pin, `2.54 mm` (`100 mil`) pitch standard header.
  - Pinout:
    1. `VCC`: +3.0V to +5.5V DC supply (typically +3.3V from Longan Nano `3V3` pin).
    2. `GND`: Ground reference (common ground with Longan Nano `GND`).
    3. `SCL`: I2C Serial Clock (connects to Longan Nano `PB6`).
    4. `SDA`: I2C Serial Data (connects to Longan Nano `PB7`).
    5. `VIN-`: Negative current sense / Load side (in parallel with screw terminal).
    6. `VIN+`: Positive current sense / Source side (in parallel with screw terminal).

#### 3.1.3 Electrical & Measurement Characteristics
- **IC**: Texas Instruments `INA219B` (SOIC-8 or SOT-23-8).
- **Shunt Resistor**: `R100` = `0.100 Ω`, 1%, 2W, 2512 SMD package (`6.3 mm × 3.2 mm`).
- **Bus Voltage Range**: `0 to +26 VDC` (High-side voltage sensing, common-mode).
- **Current Measurement Range**:
  - Full-scale: `±3.20 A` with `0.80 mA` LSB resolution (12-bit ADC @ ±320 mV PGA range).
  - High-precision (div8 gain): `±400 mA` with `0.10 mA` LSB resolution.
- **Accuracy**: `±1%` across temperature range.
- **Operating Current**: `~1.0 mA` (normal operation), `~15 µA` (sleep mode).
- **I2C Address Jumpers (`A0`, `A1`)**:
  - Default (both open): `0x40` (7-bit address).
  - Selectable addresses: `0x40`, `0x41`, `0x44`, `0x45`.

---

### 3.2 Variant B: Red INA219 Breakout Board (Ultra-Compact Variant)
*Miniaturized breakout without bulky screw terminal for ultra-compact inline enclosures.*

#### 3.2.1 Mechanical Dimensions & Layout
- **PCB Outer Dimensions**: `20.00 mm` length × `16.40 mm` width.
- **PCB Thickness**: `1.60 mm`.
- **Total Height**: `~3.00 mm` (low-profile SMD components, no terminal block fitted).
- **Mounting Holes**: 2 corner mounting holes (`Ø 2.0 mm`).
- **Connections**: Solder pads / 2.54 mm pin header pads for direct wire lead soldering or custom cabling.

#### 3.2.2 Electrical Specifications
- **Chip**: INA219 (SOT-23-8).
- **Supply Voltage**: `3.3 V to 5.5 V DC`.
- **Current Consumption**: `~1.0 mA` operating, `15 µA` sleep.
- **Current Range**: `±3.20 A` (12-bit resolution, 0.1 Ω sense resistor, ±1% accuracy).
- **Operating Temperature**: `-25 °C to +85 °C`.
- **Application**: Ideal for ultra-compact inline dongle cases or cable-mounted sensor enclosures where 3.5 mm screw terminals are too large.

---

## 4. Case & Enclosure Design Directives (AI Prompts & Guidelines)

When prompting an AI to generate OpenSCAD, CadQuery, FreeCAD, or STL 3D models for this system, use the following engineering parameters and design topologies.

### 4.1 Enclosure Architectural Topologies

#### Topology 1: Side-by-Side Dual-Board Enclosure (Desktop Monitor)
- **Reference Model**: [`longan_nano_ina219_case.scad`](../prj_current_monitor/enclosure/longan_nano_ina219_case.scad)
- **Layout**: Longan Nano mounted on the left; Blue INA219 breakout mounted on the right.
- **Overall Case Dimensions**: `75.0 mm` (width) × `55.0 mm` (depth) × `20.0–24.0 mm` (height).
- **Two-Part Enclosure**:
  - **Base (Lower Half, `14.0 mm` height)**: Houses internal alignment posts, USB-C side slot, MicroSD side slot, and front wire terminal port.
  - **Lid (Upper Half, `10.0 mm` height)**: Houses LCD viewing window, LCD flex strain-relief shelf, BOOT/RESET button holes, and rear ventilation slots.
- **Inter-Board Wiring**: 4 internal jumper wires (3V3, GND, SCL, SDA) between Longan Nano pins (Edge 1, Pins 5, 6, 16, 1) and INA219 header.

#### Topology 2: Stacked "Sandwich" Compact Enclosure (Handheld Meter)
- **Layout**: Longan Nano mounted directly above the INA219 module.
- **Overall Case Dimensions**: `~55.0 mm` (length) × `~28.0 mm` (width) × `~22.0 mm` (height).
- **Advantages**: Minimal footprint; resembles a USB power meter / inline dongle.
- **Considerations**: Requires vertical standoffs between boards (`~8.0 mm` clearance) to avoid shorting bottom-side TF card slot to INA219 components.

#### Topology 3: Inline Pass-Through Cable Dongle (Using Red INA219)
- **Layout**: Longan Nano with Red INA219 mounted inline on the back side.
- **Overall Case Dimensions**: `~52.0 mm` × `~24.0 mm` × `~12.0 mm`.
- **Connections**: Direct soldered wires exiting strain-relief grommets at both ends.

---

### 4.2 Critical Mechanical Tolerances & 3D Print Guidelines

| Parameter | Recommended Dimension | Rationale / Tolerance Rule |
| :--- | :--- | :--- |
| **Wall Thickness** | `1.80–2.00 mm` | Provides structural rigidity for FDM (PLA/PETG) with 4 perimeters (0.4 mm nozzle). |
| **Tolerance Clearance Gap** | `0.40–0.60 mm` | Prevents PCB binding while preventing excessive rattle; accounts for 3D printer shrinkage. |
| **LCD Viewport Window** | `24.50 mm × 14.00 mm` | Fully exposes 160×80 IPS active area (`21.7 × 10.8 mm`) without exposing perimeter adhesive. |
| **LCD Glass Shelf Pocket** | `27.50 mm × 17.50 mm × 1.80 mm` | Recessed pocket in lid to capture display bezel and prevent vertical displacement. |
| **FPC Flex Relief Notch** | `4.00 mm × 6.00 mm × 1.80 mm` | **MANDATORY**: Relief channel for folded display flex ribbon; prevents flex damage. |
| **USB Type-C Cutout** | `11.00 mm × 5.00 mm` | Allows clearance for USB-C cable connector housing overmold. |
| **MicroSD Card Cutout** | `13.00 mm × 3.00 mm` | Allows insertion/ejection of MicroSD card using fingernail without opening case. |
| **Terminal Wire Cutout** | `11.00 mm × 9.50 mm` | Front/side aperture for 2-wire high-current load input/output leads. |
| **Terminal Screw Access** | `2 × Ø 4.00 mm` holes | Top-lid through-holes aligned with terminal screws for easy tightening. |
| **Button Access Holes** | `2 × Ø 4.00–4.50 mm` holes | Aligned over BOOT0 and RESET actuators; permits stylus activation or printed pushers. |
| **Thermal Dissipation** | 4× slots (`3.5 × 1.5 mm`) | Located over the R100 shunt resistor. At 3.2A, the shunt dissipates $P = I^2 R \approx 1.02\text{ W}$. |
| **Mounting Boss / Posts** | Base `Ø 3.0 mm`, nipple `Ø 1.4 mm` | Alignment posts for board corner holes; nipples friction-fit into PCB mounting holes. |

---

## 5. Concise AI Prompt Template for Generating Case Code

*Copy-paste the prompt below into an LLM or CAD generation tool to produce enclosure source code:*

```text
Generate a 3D-printable two-piece snap-fit enclosure (Base and Lid) in OpenSCAD for a DC Power Monitor.
Hardware Modules to House:
1. Microcontroller Board: Sipeed Longan Nano (GD32VF103CBT6 RISC-V) with 0.96" IPS LCD:
   - PCB Size: 46.1 mm (L) x 20.0 mm (W) x 1.0 mm (H).
   - Display: 0.96" LCD mounted on top, active area 21.7 mm x 10.8 mm, outer glass 28.0 mm x 14.5 mm.
   - Display Flex Cable: 8-pin FPC ribbon folded over left edge; REQUIRES 4x6 mm x 1.8 mm deep relief pocket.
   - USB-C Port: Protrudes 3.0 mm from right edge, needs 11.0 mm x 5.0 mm wall cutout.
   - MicroSD Slot: Located on bottom side, needs 13.0 mm x 3.0 mm card access slot.
   - Buttons: BOOT0 and RESET tactile pushbuttons near USB-C, need 4.0 mm diameter access holes.
2. Sensor Board: Blue INA219 DC Current Sensor Breakout:
   - PCB Size: 25.5 mm (L) x 22.3 mm (W) x 1.6 mm (H).
   - Terminal Block: 2-pin 3.5 mm green screw terminal block (10x8x10 mm) on top edge.
   - Wall Cutout: 11.0 mm x 9.5 mm wire entry window + dual 4.0 mm screwdriver access holes on lid.
   - 4x M2.5 mounting holes at corners (21.0 mm x 17.8 mm center spacing).
   - Thermal Slots: 4x 3.5 mm ventilation slots over R100 shunt resistor (1W dissipation at 3.2A).
Case Constraints:
- Layout: Side-by-side (approx 75 mm W x 55 mm L x 22 mm H).
- Wall thickness: 2.0 mm. Tolerance clearance: 0.6 mm. Corner radius: 2.0 mm.
- Output clean, parametric, well-commented OpenSCAD script with modules for Base and Lid.
```
