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
- **Outer LCD Body & Glass Dimensions** *(Empirically verified on 2.54 mm PCB pin grid and physical test print `IMG_8187.JPG`)*:
  - Total outer LCD module body (glass + left black plastic bezel): **`28.00 mm` length × `13.50 mm` breadth**, spanning `X_pcb = 7.40 mm` to `35.40 mm` (`14.20 mm` nominal pocket width with foam adhesive margin).
  - Left black plastic bezel step: `X_pcb = 7.40 mm` to `10.50 mm` (`3.10 mm` step).
  - Smooth black LCD glass top surface: `X_pcb = 10.50 mm` to `35.40 mm` (`24.90 mm` glass length, touching the left edge of the tactile button housings).
- **Recommended Case Window Opening & Underside Pocket**:
  - Top Viewport Cutout: `27.20 mm` inner length (`28.00 mm` outer beveled length) × `11.40 mm` inner breadth (`12.60 mm` outer beveled breadth), centered at `X_pcb = 21.10 mm`, `Y_pcb = 10.00 mm`.
  - Underside Lid Recessed Pocket: `28.90 mm` length (`X_pcb = 6.80..35.70 mm`) × `15.20 mm` breadth × `1.40 mm` depth to house the full `13.50 × 28.00 mm` LCD module flush against the lid.
- **CRITICAL FPC Cable Relief Requirement**:
  - The flexible ribbon cable wraps tightly around the left edge of the display (`X_pcb = 3.00..8.00 mm`) to connect to the solder pads.
  - **Do NOT clamp or apply downward pressure** on the folded ribbon edge.
  - The enclosure top lid MUST feature an internal recess / relief pocket of at least `5.0 mm × 8.5 mm` with `1.40 mm` depth over the flex fold to prevent trace fatigue or shearing.

### 2.4 Button, Connector & Peripheral Cutout Positions
- **USB Type-C Port**:
  - Location: Right edge, centered on the 20.0 mm board width.
  - Recommended Wall Cutout: `11.5 mm` width × `4.5–5.0 mm` height (provides clearance for USB-C cable overmold shroud).
- **Tactile Buttons (`BOOT0` & `RESET`)** *(Calibrated to 2.54 mm pin header grid & physical test print `IMG_8187.JPG`)*:
  - Both buttons sit between `Pin 14` (`B13` / `B1`) and `Pin 15` (`B12` / `B10`) — accounting for the `3.80 mm` (`+1` pin pitch) center notch gap — at **`X_pcb = 37.50 mm`** in the enclosure cradle (`~8.0 mm` from the right PCB edge).
  - `BOOT0` (PA8): `X_pcb = 37.50 mm`, `Y_pcb = 13.30 mm` from front (`VB..RST`) edge.
  - `RESET` (NRST): `X_pcb = 37.50 mm`, `Y_pcb = 6.45 mm` from front (`VB..RST`) edge (`6.85 mm` center-to-center Y spacing).
  - Recommended Button Cutouts: Circular holes `Ø 4.00 mm` with top ergonomic chamfer.
- **MicroSD (TF) Card Slot**:
  - Location: Bottom side of PCB (`1.80 mm` profile height).
  - Note: External enclosure card slots are omitted when firmware provides USB Mass Storage (`prj_usb_msc` / `prj_current_monitor` Screen 6 USB Drive Mode).
- **JTAG / UART 4-Pin Edge Header (`3V3 R0 T0 GND`)**:
  - 4 pads at the left edge (`X_pcb = 0.0 mm`, centered on board width). When a 4-pin male header is soldered horizontally (`IMG_8185.JPG`), pins extend `~8.5 mm` to the left (`-X`); the left cradle stop pillars must leave a `≥ 12.0 mm` center gap.

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
---  [CENTER NOTCH: 3.8 mm (Inner Root Width: 17.8 mm)] -----
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
---  [CENTER NOTCH: 3.8 mm (Inner Root Width: 17.8 mm)] -----
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

#### 3.1.1 Mechanical Dimensions & Mounting *(Empirically verified in `IMG_8184.JPG`)*
- **PCB Outer Dimensions**:
  - **Width (along 6-pin header & `VIN- / VIN+` edge, X-axis)**: `25.50 mm` (`1.00"`).
  - **Length (from `VCC..VIN+` header edge to `VIN- / VIN+` edge, Y-axis)**: `22.30 mm` (`0.88"`).
- **PCB Thickness**: `1.60 mm`.
- **Mounting Holes**: 4 corner through-holes, `Ø 2.60–2.80 mm`.
  - **Corner Hole Center-to-Center Spacing**: **`20.40 mm`** (`8 × 2.54 mm = 20.32 mm` along the 25.5 mm width) × **`17.20 mm`** (along the 22.3 mm length).
  - **Recommended 3D-Printed Standoff Poles**: `4.0 mm` standoff height (clears bottom-side solder joints/wires) with tapered alignment poles (`Ø 1.90 mm` base tapering to `Ø 1.50 mm` tip) and `0.60 mm` per-side lateral PCB clearance (`26.7 × 23.5 mm` pocket).
- **Z-Axis Height Breakdown**:
  - Bare board + SMD components: `~3.20 mm` (+ wire jumper loops `~5.0–7.5 mm`).
  - With 2-pin 3.5 mm Screw Terminal: `~12.00 mm` total height above PCB.

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

#### Topology 1: Side-by-Side Dual-Board Snap-Fit Enclosure (Desktop Monitor, Rev 2)
- **Reference Model**: [`longan_nano_ina219_case.scad`](../prj_current_monitor/enclosure/longan_nano_ina219_case.scad)
- **Layout**: Side-by-side non-overlapping bays — Sipeed Longan Nano in the front-right bay (USB-C flush with the `+X` right wall, FPC ribbon and 4-pin JTAG header on the `-X` left edge) and Blue INA219 breakout in the rear-left bay (`25.5 mm` X width × `22.3 mm` Y length, `VIN-/VIN+` flush with the `+Y` rear wall).
- **Overall Case Dimensions**: `75.0 mm` (width) × `55.0 mm` (length/depth) × `17.5 mm` (low-profile assembled height), `2.0 mm` wall thickness, `0.8–1.2 mm` PCB pocket clearance, `2.0 mm` corner radius.
- **Two-Part Snap-Fit Enclosure**:
  - **Base (Lower Half, `14.2 mm` height)**: Houses the Longan Nano corner cradle (`47.2 × 21.0 mm` pocket at `Z = 9.5 mm`, omitting pinching notch pegs and leaving a `12.6 mm` left gap for the 4-pin JTAG header), 4× INA219 corner standoffs (`20.4 mm` X × `17.2 mm` Y spacing, `4.0 mm` height with tapered `Ø 1.9 -> 1.5 mm` poles and `26.7 × 23.5 mm` lateral guide pocket), `11.5 × 4.8 mm` right-wall USB-C cutout, `11.5 × 8.2 mm` rear-wall wire window, full 360° perimeter counterbore step, 4-wall snap-fit detent grooves (Front, Rear, Left, Right), and fingernail pry notches. (MicroSD slots omitted since USB MSC Drive Mode is built-in).
  - **Lid (Ultra-Low-Profile Upper Half, `2.4 mm` plate + `2.0 mm` 360° interlocking skirt)**: Houses the `27.2 × 11.4 mm` inner (`28.0 × 12.6 mm` outer beveled) LCD viewing window centered at `X_pcb = 21.10 mm`, internal LCD pocket (`28.9 × 15.2 × 1.4 mm`, sized for the full `13.5 × 28.0 mm` LCD module) with mandatory `5.0 × 8.5 × 1.4 mm` FPC flex cable relief pocket on the left edge, dual `Ø 4.0 mm` chamfered BOOT0/RESET button access holes at `X_pcb = 37.50 mm`, `Y_pcb = 6.45 / 13.30 mm`, dual `Ø 4.0 mm` wire/terminal access holes, 4× thermal ventilation slots directly over the `R100` shunt resistor, and a full 360° interlocking perimeter skirt with 4-wall snap beads that prevent sliding in both X and Y directions.
- **Inter-Board Wiring**: 4 internal jumper wires (3V3, GND, SCL, SDA) routed through the open central wiring courtyard between Longan Nano pins (Edge 1, Pins 5, 6, 16, 1) and the INA219 6-pin header.

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
| **PCB Cradle Clearance** | `0.50–0.60 mm` per side | `1.0–1.2 mm` total pocket clearance prevents FDM inner-wall shrinkage from pinching PCBs. |
| **360° Lid Skirt Clearance** | `0.25 mm` per side | Continuous 4-wall interlocking skirt + 4-wall snap beads prevent X/Y lid sliding. |
| **LCD Viewport Window** | `27.20 mm × 11.40 mm` (`28.0 × 12.6 mm` bevel) | Centered at `X_pcb = 21.10 mm`; exposes full `28.0 mm` LCD length while hiding red PCB. |
| **LCD Module Pocket** | `28.90 mm × 15.20 mm × 1.40 mm` | Underside recess spanning `X_pcb = 6.8..35.7 mm` to fit the `13.5 × 28.0 mm` LCD module. |
| **FPC Flex Relief Notch** | `5.00 mm × 8.50 mm × 1.40 mm` | **MANDATORY**: Relief channel at `X_pcb = 3.0..8.0 mm` for folded display flex ribbon. |
| **USB Type-C Cutout** | `11.50 mm × 4.80 mm` | Allows clearance for USB-C connector shell and cable housing overmold. |
| **Button Access Holes** | `2 × Ø 4.00 mm` holes | Located at `X_pcb = 37.50 mm`, `Y_pcb = 6.45 & 13.30 mm` (calibrated to `IMG_8187.JPG`). |
| **INA219 Standoff Grid** | `20.40 mm (X) × 17.20 mm (Y)` | Matches `25.5 mm (X) × 22.3 mm (Y)` PCB corner holes with `Ø 1.9 -> 1.5 mm` tapered pins. |
| **Thermal Dissipation** | 4× slots (`1.6 × 8.0 mm`) | Located over the R100 shunt resistor. At 3.2A, the shunt dissipates $P = I^2 R \approx 1.02\text{ W}$. |

---

## 5. Concise AI Prompt Template for Generating Case Code

*Copy-paste the prompt below into an LLM or CAD generation tool to produce enclosure source code:*

```text
Generate a 3D-printable two-piece snap-fit enclosure (Base and Low-Profile Lid) in OpenSCAD for a DC Power Monitor.
Hardware Modules to House:
1. Microcontroller Board: Sipeed Longan Nano (GD32VF103CBT6 RISC-V) with 0.96" IPS LCD:
   - PCB Size: 46.2 mm (L) x 20.0 mm (W) x 1.0 mm (H); cradle pocket 47.2 x 21.0 mm (no center-notch pegs).
   - Display: 0.96" LCD (outer module dimensions 13.5 mm breadth x 28.0 mm length at X_pcb = 7.4..35.4 mm).
   - Top LCD Window: 27.2 x 11.4 mm inner (28.0 x 12.6 mm outer bevel), centered at X_pcb = 21.10 mm, Y_pcb = 10.0 mm; underside pocket 28.9 x 15.2 x 1.4 mm.
   - Display Flex Cable: 8-pin FPC ribbon folded over left edge (X_pcb = 3.0..8.0 mm); REQUIRES 5x8.5x1.4 mm relief pocket.
   - Left JTAG Header: 4-pin header on left edge protruding 8.5 mm; leave 12.6 mm gap between left cradle stops.
   - USB-C Port: Protrudes 3.0 mm from right edge, needs 11.5 x 4.8 mm wall cutout.
   - No MicroSD card cutout (USB MSC mode is used).
   - Buttons: BOOT0 and RESET at X_pcb = 37.50 mm, Y_pcb = 6.45 mm and 13.30 mm; 4.0 mm chamfered holes.
2. Sensor Board: Blue INA219 DC Current Sensor Breakout:
   - Orientation: VIN-/VIN+ at Rear (+Y) wall, 6-pin header at Front (-Y) side.
   - PCB Size: 25.5 mm (X width) x 22.3 mm (Y length) x 1.6 mm (H); pocket 26.7 x 23.5 mm.
   - 4x Corner Standoffs: 20.4 mm (X) x 17.2 mm (Y) center spacing, 4.0 mm tall, tapered 1.9->1.5 mm poles.
   - Wall Cutout: 11.5 x 8.2 mm rear wire entry window + dual 4.0 mm top access holes + 4x thermal slots.
Case Constraints:
- Layout: Side-by-side (75.0 mm W x 55.0 mm L x 16.6 mm H: 14.2 mm Base + 2.4 mm Lid).
- Full 360-degree interlocking skirt + 4-wall snap detents (Front, Rear, Left, Right) to prevent X/Y sliding.
```
