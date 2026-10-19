// ============================================================================
// Parametric Two-Piece Snap-Fit Enclosure for DC Power Monitor (Rev 4)
// Hardware Modules:
//   1. Sipeed Longan Nano (GD32VF103CBT6 RISC-V) with 0.96" 160x80 IPS LCD
//   2. Dual Blue INA219 High-Side DC Current Sensor Breakouts (I2C 0x40 & 0x41)
//
// Authoritative Mechanical Reference & Blueprints: docs/HARDWARE_CASE_DESIGN_SPECS.md
//
// Upgrades from Rev 3:
//   1. Dual Blue INA219 Current Sensor support (occupying Rear-Left & Rear-Right bays)
//   2. 3D-printable Captive Dual-Button Plunger Insert for RESET & BOOT0
//   3. Relocated Rear (+Y) snap-fit tab to the solid bridge between the two INA219 bays
//   4. Lowered Longan post height by 0.5 mm (7.0 mm) + Lid USB-C/solder relief pocket
//      to eliminate the lid gap on the USB-C side
//   5. Internal zip-tie / wire-loop strain-relief anchors behind INA219 wire exits
//   6. Replaced jagged figure-8 screw holes with clean hull() pill slots (or omit via toggle)
// ============================================================================

$fn = 64; // Smooth cylinder tessellation

// ----------------------------------------------------------------------------
// 0. Feature Toggles & Render Mode Selection
// ----------------------------------------------------------------------------
enable_dual_ina219 = true;  // true = 2x INA219 mounts/cutouts/vents; false = single INA219
enable_screw_slot  = false; // false = omit top terminal holes (for direct-soldered wires)

// Mode aliases for robust command-line invocation (supports both -D mode="base" and -D mode=base)
base       = "base";
lid        = "lid";
lid_print  = "lid_print";
buttons    = "buttons";
print      = "print";
assembled  = "assembled";
exploded   = "exploded";
ghost      = "ghost";

// Modes: "base", "lid", "lid_print", "buttons", "print", "assembled", "exploded", "ghost"
mode = "print";

// ----------------------------------------------------------------------------
// 1. Global Enclosure Constraints & Print Tolerances
// ----------------------------------------------------------------------------
case_w          = 75.0;  // Overall enclosure width (X-axis, mm)
case_l          = 55.0;  // Overall enclosure length/depth (Y-axis, mm)
base_h          = 14.2;  // Lower Base shell height (mm)
lid_h           = 2.4;   // Ultra-low-profile flat Upper Lid thickness (mm)
case_h          = base_h + lid_h; // Overall assembled height (16.6 mm)

wall            = 2.0;   // Outer wall and floor thickness (mm)
corner_r        = 2.0;   // Outer vertical corner fillet radius (mm)

// 360-degree interlocking perimeter lip & 4-side snap-fit parameters
lip_h           = 2.0;   // Height of lid mating skirt extending into base (mm)
lip_thick       = 1.0;   // Thickness of mating skirt wall (mm)
lip_clearance   = 0.25;  // Per-side radial clearance between base step and lid skirt (mm)
snap_undercut   = 0.40;  // Radial engagement of snap-fit ridge (mm)
snap_len        = 12.0;  // Width of front snap-fit tab (mm)
rear_snap_len   = 8.0;   // Width of rear snap-fit tab (fits between Dual INA219 cutouts)

// ----------------------------------------------------------------------------
// 2. Sipeed Longan Nano Hardware Specifications
// ----------------------------------------------------------------------------
longan_l        = 46.2;  // Nominal PCB length (X-axis, mm)
longan_w        = 20.0;  // Nominal PCB width (Y-axis, mm)
longan_t        = 1.2;   // Actual FR4 PCB thickness (mm)

// LCD Module (0.96" 160x80 IPS, 13.5 mm breadth x 28.0 mm length)
lcd_glass_x_ofs = 7.4;   // Left (-X) edge of 28.0 mm LCD module body from PCB left edge (mm)
lcd_glass_l     = 28.0;  // Full black LCD module length (mm)
lcd_glass_w     = 14.2;  // Outer LCD module width + foam tape margin (mm)
lcd_win_cx_ofs  = 21.10; // Center of LCD top window from PCB left edge (mm)
lcd_win_l       = 27.2;  // Top viewport inner length (mm)
lcd_win_w       = 11.4;  // Top viewport inner breadth (mm)
lcd_pocket_d    = 1.4;   // Underside recess depth in lid ceiling (mm)

// 8-pin FPC Display Flex Cable Relief Pocket (folded over left edge of LCD, X_pcb = 3.0..8.0 mm)
fpc_relief_l    = 5.0;   // FPC relief pocket X length (mm)
fpc_relief_w    = 8.5;   // FPC relief pocket Y width (mm)
fpc_relief_d    = 1.4;   // FPC relief pocket Z depth into lid underside (mm)

// USB Type-C Port (protrudes ~3.0 mm from right edge of Longan Nano)
usbc_protrude   = 3.0;   // Protrusion past right PCB edge (mm)
usbc_cut_w      = 11.5;  // Wall cutout width (Y-axis, mm)
usbc_cut_h      = 5.2;   // Wall cutout height in Base (Z = 9.0..14.2 mm, lowered 0.4 mm)

// Tactile Pushbuttons & 3D-Printable Captive Button Insert Parameters
btn_hole_d      = 4.0;   // Lid through-hole diameter (mm)
btn_stem_d      = 3.3;   // Printable button stem diameter (0.35 mm radial clearance)
btn_flange_d    = 5.2;   // Printable button retention flange diameter (mm)
btn_flange_h    = 0.8;   // Printable button flange thickness (mm)
btn_proud_h     = 0.9;   // Height button dome extends above top of lid (mm)
btn_x_ofs       = 37.50; // Button center X offset from Longan left edge (mm)
btn_reset_y     = 6.45;  // RESET button Y offset from Longan front (-Y) edge (mm)
btn_boot_y      = 13.30; // BOOT0 button Y offset from Longan front (-Y) edge (mm)

// Placement of Longan Nano in Right/Front Bay
longan_right_gap = 0.5;
longan_x        = case_w - wall - longan_right_gap - longan_l; // 26.3 mm
longan_y        = 5.5;                      // Front bay Y origin (spans 5.5..25.5 mm)
longan_cy       = longan_y + longan_w / 2;  // 15.5 mm centerline
longan_post_h   = 7.0;                      // Lowered 0.5 mm (from 7.5) for flush USB-C lid fit!
longan_pcb_z    = wall + longan_post_h;     // 9.0 mm (PCB bottom Z)

// ----------------------------------------------------------------------------
// 3. Dual Blue INA219 Current Sensor Breakout Specifications
// ----------------------------------------------------------------------------
ina_w           = 25.5;  // PCB width along X-axis (mm)
ina_l           = 22.3;  // PCB length along Y-axis (mm)
ina_t           = 1.6;   // PCB thickness (mm)
ina_hole_dx     = 20.4;  // Mounting hole X spacing (mm)
ina_hole_dy     = 17.2;  // Mounting hole Y spacing (mm)
ina_pin_d1      = 1.9;   // Alignment pole base diameter (mm)
ina_pin_d2      = 1.5;   // Alignment pole tapered tip diameter (mm)
term_cut_w      = 10.0;  // Rear wall wire entry window width (mm)
term_cut_h      = 6.8;   // Rear wall wire window height (Z = 7.4..14.2 mm, protects lower PCB)
term_screw_d    = 4.0;   // Top access slot width (mm)
vent_count      = 4;     // Number of ventilation slots per sensor
vent_slot_t     = 1.6;   // Slot narrow opening width (mm)
vent_pitch      = 3.2;   // Center-to-center spacing of thermal slots (mm)

// Sensor #1 (Rear-Left Bay, I2C 0x40) & Sensor #2 (Rear-Right Bay, I2C 0x41)
ina1_x          = wall + 2.5;                       // 4.5 mm (spans X = 4.5..30.0 mm)
ina2_x          = case_w - wall - 2.5 - ina_w;      // 45.0 mm (spans X = 45.0..70.5 mm)
ina_y           = case_l - wall - 0.9 - ina_l;      // 29.8 mm (spans Y = 29.8..52.1 mm)
ina_cy          = ina_y + ina_l / 2;                // 40.95 mm Y centerline
ina_post_h      = 4.0;                              // 4.0 mm under-board clearance
ina_pcb_z       = wall + ina_post_h;                // 6.0 mm (PCB bottom Z; PCB top = 7.6 mm)

ina_x_list      = enable_dual_ina219 ? [ina1_x, ina2_x] : [ina1_x];

// ----------------------------------------------------------------------------
// 4. Top-Level Render Dispatcher
// ----------------------------------------------------------------------------
if (mode == "base") {
    base();
} else if (mode == "lid") {
    lid();
} else if (mode == "lid_print") {
    translate([0, case_l, lid_h]) rotate([180, 0, 0]) lid();
} else if (mode == "buttons") {
    button_insert_print();
} else if (mode == "print") {
    base();
    translate([case_w + 10, case_l, lid_h]) rotate([180, 0, 0]) lid();
    translate([case_w + 15, -12, 0]) button_insert_print();
} else if (mode == "assembled") {
    base();
    translate([0, 0, base_h]) lid();
    button_insert_assembled();
} else if (mode == "ghost") {
    %base();
    %translate([0, 0, base_h]) lid();
    button_insert_assembled();
    ghost_hardware();
} else { // "exploded"
    base();
    translate([0, 0, base_h + 6]) button_insert_assembled();
    translate([0, 0, base_h + 14]) lid();
}

// ============================================================================
// Helper Modules
// ============================================================================
module rounded_box(w, l, h, r = corner_r) {
    r_clamped = max(0.1, min(r, min(w, l) / 2));
    hull() {
        translate([r_clamped, r_clamped, 0])         cylinder(r = r_clamped, h = h);
        translate([w - r_clamped, r_clamped, 0])     cylinder(r = r_clamped, h = h);
        translate([r_clamped, l - r_clamped, 0])     cylinder(r = r_clamped, h = h);
        translate([w - r_clamped, l - r_clamped, 0]) cylinder(r = r_clamped, h = h);
    }
}

module rounded_slot_z(w, l, h) {
    r = min(w, l) / 2;
    hull() {
        if (w >= l) {
            translate([r, r, 0]) cylinder(r = r, h = h);
            translate([w - r, r, 0]) cylinder(r = r, h = h);
        } else {
            translate([r, r, 0]) cylinder(r = r, h = h);
            translate([r, l - r, 0]) cylinder(r = r, h = h);
        }
    }
}

// ============================================================================
// Module: Captive Dual-Button Plunger Insert (Prints flat in ~2 minutes)
// ============================================================================
// Total height Calculation:
//   Switch top is at Z = 9.0 (PCB bottom) + 1.2 (PCB) + 2.5 (Switch) = 12.7 mm.
//   Plunger bottom starts at Z_assembled = 13.0 mm (0.3 mm pre-travel clearance).
//   Top of lid is at Z = 16.6 mm; button top dome reaches Z = 16.6 + 0.9 = 17.5 mm.
//   Total plunger height = 17.5 - 13.0 = 4.5 mm.
module button_insert_print() {
    total_h = (case_h + btn_proud_h) - (longan_pcb_z + longan_t + 2.5 + 0.3); // 4.5 mm
    dy      = btn_boot_y - btn_reset_y; // 6.85 mm center-to-center

    union() {
        for (y_ofs = [0, dy]) {
            translate([0, y_ofs, 0]) {
                // 1. Bottom captive retention flange
                cylinder(d = btn_flange_d, h = btn_flange_h);
                // 2. Main cylindrical plunger stem
                cylinder(d = btn_stem_d, h = total_h - 0.4);
                // 3. Ergonomic chamfered top cap
                translate([0, 0, total_h - 0.4])
                    cylinder(d1 = btn_stem_d, d2 = btn_stem_d - 0.8, h = 0.4);
            }
        }
        // 4. Flexible anti-rotation tether connecting the two flanges (1.6 mm wide x 0.6 mm thick)
        translate([-0.8, 0, 0])
            cube([1.6, dy, 0.6]);
    }
}

module button_insert_assembled() {
    z_base = longan_pcb_z + longan_t + 2.5 + 0.3; // 13.0 mm
    color([0.95, 0.35, 0.15])
        translate([longan_x + btn_x_ofs, longan_y + btn_reset_y, z_base])
            button_insert_print();
}

// ============================================================================
// Module: Base (Lower Enclosure Half)
// ============================================================================
module base() {
    step_wall = 1.2;

    union() {
        difference() {
            // 1. Outer rounded shell
            rounded_box(case_w, case_l, base_h, corner_r);

            // 2. Main interior cavity
            translate([wall, wall, wall])
                rounded_box(case_w - 2*wall, case_l - 2*wall, base_h, max(0.2, corner_r - wall));

            // 3. Full 360-degree upper perimeter counterbore step
            translate([step_wall, step_wall, base_h - lip_h])
                rounded_box(
                    case_w - 2*step_wall, case_l - 2*step_wall,
                    lip_h + 1.0, max(0.4, corner_r - step_wall)
                );

            // 4. Symmetric 4-Wall Snap-Fit Detent Grooves
            // Front (-Y) snap groove
            translate([longan_x + longan_l/2 - (snap_len + 2)/2, step_wall - snap_undercut, base_h - lip_h + 0.35])
                cube([snap_len + 2, snap_undercut + 1.0, 0.9]);

            // Rear (+Y) snap groove — centered at X = 37.5 mm between INA219 #1 and #2!
            translate([case_w/2 - (rear_snap_len + 2)/2, case_l - step_wall - 1.0, base_h - lip_h + 0.35])
                cube([rear_snap_len + 2, snap_undercut + 1.0, 0.9]);

            // Left (-X) wall snap groove
            translate([step_wall - snap_undercut, 20.0, base_h - lip_h + 0.35])
                cube([snap_undercut + 1.0, 14.0, 0.9]);

            // Right (+X) wall snap groove
            translate([case_w - step_wall - 1.0, 30.0, base_h - lip_h + 0.35])
                cube([snap_undercut + 1.0, 14.0, 0.9]);

            // 5. Fingernail pry notches
            translate([-0.5, case_l/2 - 4, base_h - 0.7])
                cube([1.1, 8.0, 1.4]);
            translate([case_w - 0.6, case_l - 12, base_h - 0.7])
                cube([1.1, 8.0, 1.4]);

            // 6. Longan Nano USB-C Wall Cutout (aligned to lowered Z = 9.0 mm PCB)
            translate([case_w - wall - 1.5, longan_cy - usbc_cut_w/2, base_h - usbc_cut_h])
                cube([wall + 3.0, usbc_cut_w, usbc_cut_h + 1.0]);

            // 7. Rear Wire Windows for INA219 #1 (and INA219 #2 if enabled)
            for (ix = ina_x_list) {
                icx = ix + ina_w / 2;
                translate([icx - term_cut_w/2, case_l - wall - 1.5, base_h - term_cut_h])
                    cube([term_cut_w, wall + 3.0, term_cut_h + 1.0]);
            }
        }

        // 8. Longan Nano Precision Corner Cradle
        longan_cradle();

        // 9. INA219 Mounting Standoffs + Wire Strain-Relief Tie Posts (1 or 2 modules)
        for (ix = ina_x_list) {
            ina219_mounts(ix, ina_y);
        }
    }
}

module longan_cradle() {
    tol_x = 0.5;
    tol_y = 0.5;
    lip_z = 1.6;
    z_overlap = 0.5;

    for (end_idx = [0, 1]) {
        pad_x  = (end_idx == 0) ? longan_x : (longan_x + longan_l - 4.5);
        rail_x = (end_idx == 0) ? (longan_x - tol_x) : (longan_x + longan_l + tol_x - 6.5);

        translate([pad_x, longan_y - tol_y, wall - z_overlap])
            cube([4.5, 2.8 + tol_y, longan_post_h + z_overlap]);
        translate([rail_x, longan_y - tol_y - 1.4, wall - z_overlap])
            cube([6.5, 1.4, longan_post_h + lip_z + z_overlap]);

        translate([pad_x, longan_y + longan_w - 2.8, wall - z_overlap])
            cube([4.5, 2.8 + tol_y, longan_post_h + z_overlap]);
        translate([rail_x, longan_y + longan_w + tol_y, wall - z_overlap])
            cube([6.5, 1.4, longan_post_h + lip_z + z_overlap]);
    }

    translate([longan_x + longan_l/2 - 3.0, longan_y - tol_y - 1.4, wall - z_overlap]) {
        cube([6.0, 2.6 + tol_y, longan_post_h + z_overlap]);
        cube([6.0, 1.4, longan_post_h + lip_z + z_overlap]);
    }
    translate([longan_x + longan_l/2 - 3.0, longan_y + longan_w - 1.2, wall - z_overlap]) {
        cube([6.0, 2.6 + tol_y, longan_post_h + z_overlap]);
        translate([0, 1.2 + tol_y, 0])
            cube([6.0, 1.4, longan_post_h + lip_z + z_overlap]);
    }

    translate([longan_x - tol_x - 1.5, longan_y - tol_y - 1.4, wall - z_overlap])
        cube([1.5, 3.7 + tol_y, longan_post_h + lip_z + z_overlap]);
    translate([longan_x - tol_x - 1.5, longan_y + longan_w - 2.3, wall - z_overlap])
        cube([1.5, 3.7 + tol_y, longan_post_h + lip_z + z_overlap]);

    translate([longan_x + longan_l + tol_x, longan_y - tol_y - 1.4, wall - z_overlap])
        cube([case_w - wall - (longan_x + longan_l + tol_x) + 0.2, 4.0 + tol_y, longan_post_h + lip_z + z_overlap]);
    translate([longan_x + longan_l + tol_x, longan_y + longan_w - 2.6, wall - z_overlap])
        cube([case_w - wall - (longan_x + longan_l + tol_x) + 0.2, 4.0 + tol_y, longan_post_h + lip_z + z_overlap]);
}

module ina219_mounts(mod_x, mod_y) {
    z_overlap = 0.5;
    tol       = 0.6;
    mod_cx    = mod_x + ina_w / 2;
    mod_cy    = mod_y + ina_l / 2;

    for (dx = [-ina_hole_dx/2, ina_hole_dx/2]) {
        for (dy = [-ina_hole_dy/2, ina_hole_dy/2]) {
            translate([mod_cx + dx, mod_cy + dy, wall - z_overlap]) {
                cylinder(d = 4.6, h = ina_post_h + z_overlap);
                cylinder(d1 = ina_pin_d1, d2 = ina_pin_d2, h = ina_post_h + ina_t + 1.6 + z_overlap);
            }
        }
    }

    // Left and Right lateral guide ribs
    translate([mod_x - tol - 1.2, mod_y + 4.0, wall - z_overlap])
        cube([1.2, ina_l - 8.0, ina_post_h + ina_t + z_overlap]);
    translate([mod_x + ina_w + tol, mod_y + 4.0, wall - z_overlap])
        cube([1.2, ina_l - 8.0, ina_post_h + ina_t + z_overlap]);

    // Internal Zip-Tie / Wire Strain-Relief Arch under the rear sense pad area
    translate([mod_cx - 4.0, mod_y + ina_l - 5.0, wall - z_overlap])
        difference() {
            cube([8.0, 3.0, ina_post_h - 0.4 + z_overlap]);
            translate([1.8, -0.5, z_overlap + 1.2])
                cube([4.4, 4.0, ina_post_h]);
        }
}

// ============================================================================
// Module: Lid (Upper Cover with Captive Button Pocket & Dual Sensor Vents)
// ============================================================================
module lid() {
    lcd_glass_x   = longan_x + lcd_glass_x_ofs;
    lcd_glass_y   = longan_cy - lcd_glass_w / 2;
    lcd_win_x     = longan_x + lcd_win_cx_ofs - lcd_win_l/2;
    lcd_win_y     = longan_cy - lcd_win_w / 2;
    screw_y       = ina_y + ina_l - 3.8;
    screw_dx      = 2.15;
    vent_center_y = ina_y + 11.5;

    difference() {
        union() {
            // 1. Solid low-profile top plate
            rounded_box(case_w, case_l, lid_h, corner_r);
            // 2. Full 360-Degree Interlocking Perimeter Skirt
            lid_snap_skirt();
        }

        // 3. LCD Viewing Window Cutout & Top Bevel
        translate([lcd_win_x, lcd_win_y, -1.0])
            cube([lcd_win_l, lcd_win_w, lid_h + 2.0]);
        hull() {
            translate([lcd_win_x, lcd_win_y, lid_h - 0.5])
                cube([lcd_win_l, lcd_win_w, 0.1]);
            translate([lcd_win_x - 0.5, lcd_win_y - 0.6, lid_h - 0.05])
                cube([lcd_win_l + 0.8, lcd_win_w + 1.2, 0.1]);
        }

        // 4. Underside Recessed Pocket for LCD Module
        translate([lcd_glass_x - 0.6, lcd_glass_y - 0.5, -0.5])
            cube([lcd_glass_l + 0.9, lcd_glass_w + 1.0, lcd_pocket_d + 0.5]);

        // 5. Underside Relief Pocket for Folded 8-Pin FPC Ribbon
        translate([longan_x + 3.0, longan_cy - fpc_relief_w/2, -0.5])
            cube([fpc_relief_l, fpc_relief_w, fpc_relief_d + 0.5]);

        // 6. NEW: Underside Relief Pocket for USB-C Shell & Top Pin Solder Joints (0.8 mm deep)
        // Eliminates the lid gap seen in case_rev3_usb_port_side.jpg!
        translate([longan_x + longan_l - 5.0, longan_cy - 5.2, -0.5])
            cube([6.5, 10.4, 0.8 + 0.5]);
        translate([longan_x + 1.0, longan_y - 0.2, -0.5])
            cube([longan_l - 2.0, 2.6, 0.8 + 0.5]);
        translate([longan_x + 1.0, longan_y + longan_w - 2.4, -0.5])
            cube([longan_l - 2.0, 2.6, 0.8 + 0.5]);

        // 7. Tactile Button Shaft Through-Holes + Underside Captive Flange Pocket (1.4 mm deep)
        for (by = [longan_y + btn_reset_y, longan_y + btn_boot_y]) {
            translate([longan_x + btn_x_ofs, by, -1.0])
                cylinder(d = btn_hole_d, h = lid_h + 2.0);
        }
        // Stadium-shaped underside counterbore pocket for the captive dual-button flange
        hull() {
            translate([longan_x + btn_x_ofs, longan_y + btn_reset_y, -0.5])
                cylinder(d = btn_flange_d + 0.6, h = lcd_pocket_d + 0.5);
            translate([longan_x + btn_x_ofs, longan_y + btn_boot_y, -0.5])
                cylinder(d = btn_flange_d + 0.6, h = lcd_pocket_d + 0.5);
        }

        // 8. Thermal Ventilation Slots & Optional Pill Slot for Each INA219 Sensor
        for (ix = ina_x_list) {
            icx = ix + ina_w / 2;
            if (enable_screw_slot) {
                hull() {
                    translate([icx - screw_dx, screw_y, -1.0]) cylinder(d = term_screw_d, h = lid_h + 2.0);
                    translate([icx + screw_dx, screw_y, -1.0]) cylinder(d = term_screw_d, h = lid_h + 2.0);
                }
            }
            for (i = [0 : vent_count - 1]) {
                slot_x = icx - ((vent_count - 1) * vent_pitch)/2 + i * vent_pitch - vent_slot_t/2;
                translate([slot_x, vent_center_y - 4.0, -1.0])
                    rounded_slot_z(vent_slot_t, 8.0, lid_h + 2.0);
            }
        }
    }
}

module lid_snap_skirt() {
    step_wall   = 1.2;
    inset       = step_wall + lip_clearance;
    skirt_w     = case_w - 2 * inset;
    skirt_l     = case_l - 2 * inset;
    skirt_r     = max(0.3, corner_r - inset);
    skirt_tot_h = lip_h + 0.5;

    difference() {
        translate([inset, inset, -lip_h])
            difference() {
                rounded_box(skirt_w, skirt_l, skirt_tot_h, skirt_r);
                translate([lip_thick, lip_thick, -0.5])
                    rounded_box(
                        skirt_w - 2*lip_thick, skirt_l - 2*lip_thick,
                        skirt_tot_h + 1.0, max(0.2, skirt_r - lip_thick)
                    );
            }

        // Clear skirt at Right (+X) USB-C port & pry notch
        translate([case_w - wall - 1.0, longan_cy - usbc_cut_w/2 - 1.0, -lip_h - 0.5])
            cube([wall + 2.0, usbc_cut_w + 2.0, lip_h + 0.5]);
        translate([case_w - wall - 1.0, case_l - 13.0, -lip_h - 0.5])
            cube([wall + 2.0, 10.0, lip_h + 0.5]);

        // Clear skirt at Left (-X) pry notch
        translate([0, case_l/2 - 5.0, -lip_h - 0.5])
            cube([wall + 2.0, 10.0, lip_h + 0.5]);

        // Clear skirt at Rear (+Y) INA219 wire windows (1 or 2 modules)
        for (ix = ina_x_list) {
            icx = ix + ina_w / 2;
            translate([icx - term_cut_w/2 - 1.0, case_l - wall - 1.0, -lip_h - 0.5])
                cube([term_cut_w + 2.0, wall + 2.0, lip_h + 0.5]);
        }
    }

    // Outward Snap-Fit Detent Beads on ALL 4 Walls
    // Front (-Y) snap bead
    translate([longan_x + longan_l/2 - snap_len/2, inset, -lip_h])
        translate([0, -snap_undercut, 0.35])
            hull() {
                cube([snap_len, snap_undercut + 0.1, 0.45]);
                translate([0, snap_undercut, -0.35]) cube([snap_len, 0.1, 1.1]);
            }

    // Rear (+Y) snap bead — centered at X = 37.5 mm between INA219 #1 and #2
    translate([case_w/2 - rear_snap_len/2, case_l - inset - lip_thick, -lip_h])
        translate([0, lip_thick - 0.1, 0.35])
            hull() {
                cube([rear_snap_len, snap_undercut + 0.1, 0.45]);
                translate([0, 0, -0.35]) cube([rear_snap_len, 0.1, 1.1]);
            }

    // Left (-X) snap bead
    translate([inset, 21.0, -lip_h])
        translate([-snap_undercut, 0, 0.35])
            hull() {
                cube([snap_undercut + 0.1, 12.0, 0.45]);
                translate([snap_undercut, 0, -0.35]) cube([0.1, 12.0, 1.1]);
            }

    // Right (+X) snap bead
    translate([case_w - inset - lip_thick, 31.0, -lip_h])
        translate([lip_thick - 0.1, 0, 0.35])
            hull() {
                cube([snap_undercut + 0.1, 12.0, 0.45]);
                translate([0, 0, -0.35]) cube([0.1, 12.0, 1.1]);
            }
}

// ============================================================================
// Ghost Hardware Visualization Module
// ============================================================================
module ghost_hardware() {
    translate([longan_x, longan_y, longan_pcb_z]) {
        color([0.85, 0.15, 0.15]) cube([longan_l, longan_w, longan_t]);
        color([0.1, 0.1, 0.15])
            translate([lcd_glass_x_ofs, (longan_w - 13.5)/2, longan_t])
                cube([lcd_glass_l, 13.5, 2.8]);
        color([0.75, 0.75, 0.78])
            translate([longan_l - 4.5, (longan_w - 8.94)/2, longan_t])
                cube([4.5 + usbc_protrude, 8.94, 3.16]);
        color([0.95, 0.95, 0.95]) {
            translate([btn_x_ofs, btn_reset_y, longan_t]) cylinder(d = 2.4, h = 2.5);
            translate([btn_x_ofs, btn_boot_y, longan_t])  cylinder(d = 2.4, h = 2.5);
        }
    }

    for (ix = ina_x_list) {
        translate([ix, ina_y, ina_pcb_z]) {
            color([0.1, 0.35, 0.65])
                difference() {
                    cube([ina_w, ina_l, ina_t]);
                    for (dx = [-ina_hole_dx/2, ina_hole_dx/2])
                        for (dy = [-ina_hole_dy/2, ina_hole_dy/2])
                            translate([ina_w/2 + dx, ina_l/2 + dy, -0.5])
                                cylinder(d = 2.6, h = ina_t + 1.0);
                }
            color([0.15, 0.15, 0.15])
                translate([(ina_w - 6.3)/2, 11.5 - 1.6, ina_t])
                    cube([6.3, 3.2, 1.2]);
        }
    }
}
