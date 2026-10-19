# 3D Printable Enclosure Assets & CAD Models

This directory contains the 3D-printable snap-fit enclosure files for the Sipeed Longan Nano & INA219 DC Current & Power Monitor (`prj_current_monitor`).

> [!IMPORTANT]
> **Authoritative Engineering Reference & Single Source of Truth**:  
> For complete dimensional blueprints, physical PCB stackup, 3D print tolerances, pinouts, and AI CAD design directives, refer to:  
> [**`docs/HARDWARE_CASE_DESIGN_SPECS.md`**](../../docs/HARDWARE_CASE_DESIGN_SPECS.md)

---

## Directory Contents

- **Parametric OpenSCAD Source**:
  - [`longan_nano_ina219_case.scad`](longan_nano_ina219_case.scad): Fully parameterized 3D model for Base, Lid, and dual-part print beds.
- **Automated Mesh Export Script**:
  - [`generate_meshes.ps1`](generate_meshes.ps1): PowerShell script that invokes OpenSCAD to render watertight CGAL meshes (`.stl`, `.3mf`) and orthographic PNG previews.
- **Production Slicer Files & Meshes (Rev 4)**:
  - Combined Print Bed (Base + Lid + Buttons): [`case_both_print.3mf`](case_both_print.3mf), [`case_both_print.stl`](case_both_print.stl)
  - Base Shell (Dual INA219 Bays + Wire Arches): [`case_base.3mf`](case_base.3mf), [`case_base.stl`](case_base.stl)
  - Low-Profile Lid (Support-free, USB-C/Solder Relief): [`case_lid_print.3mf`](case_lid_print.3mf), [`case_lid_print.stl`](case_lid_print.stl)
  - Captive Dual-Button Plunger Insert: [`case_buttons.3mf`](case_buttons.3mf), [`case_buttons.stl`](case_buttons.stl)
  - Assembly & Exploded Reference: [`case_assembled.stl`](case_assembled.stl), [`case_exploded.stl`](case_exploded.stl)
- **Empirical 3D Print Verification Photos (Rev 3)**:
  - [`case_rev3_assembled_active.jpg`](case_rev3_assembled_active.jpg): Powered on via USB-C with live ST7735 LCD current display and connected sensor leads.
  - [`case_rev3_internal_layout.jpg`](case_rev3_internal_layout.jpg): Open comparison showing Longan Nano cradle, JTAG clearance, INA219 standoffs, wiring, and lid underside geometry.
  - [`case_rev3_assembled_top.jpg`](case_rev3_assembled_top.jpg): Top-down view showing LCD bezel window framing, chamfered button holes, and ventilation louvers.
  - [`case_rev3_side_profile.jpg`](case_rev3_side_profile.jpg): Side elevation showing ultra-low-profile 16.6 mm height and interlocking seam.
  - [`case_rev3_rear_wire_window.jpg`](case_rev3_rear_wire_window.jpg): Rear elevation showing sensor wire exit cutout.
  - [`case_rev3_usb_port_side.jpg`](case_rev3_usb_port_side.jpg): Side elevation showing centered USB Type-C connector cutout.
