// ============================================================================
// Parametric Two-Piece Snap-Fit Enclosure for DC Power Monitor
// Hardware Modules:
//   1. Sipeed Longan Nano (GD32VF103CBT6 RISC-V) with 0.96" 160x80 IPS LCD
//   2. Blue INA219 High-Side DC Current Sensor Breakout (with Screw Terminal)
// ============================================================================

$fn = 64; // Smooth cylinder tessellation

// ----------------------------------------------------------------------------
// 1. Global Enclosure Constraints & Print Tolerances
// ----------------------------------------------------------------------------
case_w          = 75.0;  // Overall enclosure width (X-axis, mm)
case_l          = 55.0;  // Overall enclosure length/depth (Y-axis, mm)
case_h          = 22.0;  // Overall assembled height (Z-axis, mm)

wall            = 2.0;   // Outer wall, floor, and ceiling thickness (mm)
clearance       = 0.6;   // Mechanical fit tolerance clearance (mm)
corner_r        = 2.0;   // Outer vertical corner fillet radius (mm)

base_h          = 14.0;  // Lower Base shell height (mm)
lid_h           = case_h - base_h; // Upper Lid shell height (8.0 mm)

// Snap-fit interlocking lip & detent parameters
lip_h           = 2.2;   // Height of lid mating skirt extending into base (mm)
lip_thick       = 1.1;   // Thickness of mating skirt wall (mm)
snap_undercut   = 0.45;  // Radial engagement of snap-fit ridge (mm)
snap_len        = 14.0;  // Width of front/back snap-fit tabs (mm)

// ----------------------------------------------------------------------------
// 2. Sipeed Longan Nano Hardware Specifications
// ----------------------------------------------------------------------------
longan_l        = 46.1;  // PCB length (X-axis, mm)
longan_w        = 20.0;  // PCB width (Y-axis, mm)
longan_t        = 1.0;   // FR4 PCB thickness (mm)

// LCD Module (0.96" IPS, mounted on top of Longan Nano)
lcd_active_l    = 21.7;  // Active display area length (mm)
lcd_active_w    = 10.8;  // Active display area width (mm)
lcd_glass_l     = 28.0;  // Outer LCD glass bezel length (mm)
lcd_glass_w     = 14.5;  // Outer LCD glass bezel width (mm)
lcd_win_l       = 24.5;  // Top viewport cutout length (mm)
lcd_win_w       = 14.0;  // Top viewport cutout width (mm)
lcd_glass_x_ofs = 9.5;   // Left edge of 28mm LCD glass from PCB left edge (mm)
lcd_pocket_d    = 1.8;   // LCD glass shelf & FPC relief pocket depth (mm)

// 8-pin FPC Display Flex Cable Relief Pocket (folded over left edge of LCD)
fpc_relief_l    = 4.0;   // Relief pocket X length (mm)
fpc_relief_w    = 6.0;   // Relief pocket Y width (mm)
fpc_relief_d    = 1.8;   // Relief pocket Z depth (mm)

// USB Type-C Port (protrudes 3.0 mm from right edge of Longan Nano)
usbc_protrude   = 3.0;   // Protrusion past right PCB edge (mm)
usbc_cut_w      = 11.0;  // Wall cutout width (Y-axis, mm)
usbc_cut_h      = 5.0;   // Wall cutout height (Z-axis, mm)

// MicroSD (TF) Slot (bottom side of Longan Nano)
sd_cut_w        = 13.0;  // Card access slot width (mm)
sd_cut_h        = 3.0;   // Card access slot height (mm)

// Tactile Pushbuttons (BOOT0 and RESET near USB-C)
btn_hole_d      = 4.0;   // Button access hole diameter (mm)
btn_x_ofs       = 40.2;  // Button center X offset from Longan left edge (mm)
btn_reset_y     = 6.0;   // RESET button Y offset from Longan front edge (mm)
btn_boot_y      = 14.0;  // BOOT0 button Y offset from Longan front edge (mm)

// Center side index notches on Longan Nano PCB
notch_x_ofs     = 21.5;  // Center of 3.8 mm side index notches from left edge (mm)
notch_d         = 3.8;   // Width/diameter of semi-circular side notches (mm)

// Placement of Longan Nano in Right/Front Bay (USB-C flush with +X Right wall)
longan_x        = case_w - wall - longan_l; // 26.9 mm (right edge at X = 73.0 mm)
longan_y        = 5.5;                      // Front bay Y origin (spans 5.5..25.5 mm)
longan_cy       = longan_y + longan_w / 2;  // 15.5 mm centerline
longan_post_h   = 7.0;                      // Standoff height above base floor (mm)
longan_pcb_z    = wall + longan_post_h;     // 9.0 mm (PCB bottom Z)

// ----------------------------------------------------------------------------
// 3. Blue INA219 Current Sensor Breakout Specifications
// ----------------------------------------------------------------------------
ina_l           = 25.5;  // PCB length (Y-axis, mm)
ina_w           = 22.3;  // PCB width (X-axis, mm)
ina_t           = 1.6;   // PCB thickness (mm)

// 4x M2.5 Corner Mounting Holes
ina_hole_dx     = 17.8;  // Center-to-center spacing along width (X-axis, mm)
ina_hole_dy     = 21.0;  // Center-to-center spacing along length (Y-axis, mm)
ina_hole_d      = 2.5;   // Nominal M2.5 hole diameter (mm)

// 2-Pin 3.5 mm Green Screw Terminal Block (10 x 8 x 10 mm on top/+Y edge)
term_w          = 10.0;  // Terminal block width (X-axis, mm)
term_d          = 8.0;   // Terminal block depth (Y-axis, mm)
term_h          = 10.0;  // Terminal block height (Z-axis, mm)
term_pitch      = 3.5;   // Terminal screw pitch (mm)
term_cut_w      = 11.0;  // Wall cutout wire entry width (mm)
term_cut_h      = 9.5;   // Wall cutout wire entry height (mm)
term_screw_d    = 4.0;   // Dual screwdriver access hole diameter on lid (mm)

// Thermal Ventilation Slots over R100 Shunt Resistor (1W dissipation at 3.2A)
vent_count      = 4;     // Number of ventilation slots
vent_slot_w     = 3.5;   // Characteristic slot width/length (mm)
vent_slot_t     = 1.6;   // Slot narrow opening width (mm)
vent_pitch      = 3.2;   // Center-to-center spacing of thermal slots (mm)

// Placement of Blue INA219 in Left/Rear Bay (Terminal block flush with +Y Rear wall)
ina_x           = wall + 2.0;                         // 4.0 mm (spans X = 4.0..26.3 mm)
ina_y           = case_l - wall - clearance/2 - ina_l;// 27.2 mm (spans Y = 27.2..52.7 mm)
ina_cx          = ina_x + ina_w / 2;                  // 15.15 mm X centerline
ina_cy          = ina_y + ina_l / 2;                  // 39.95 mm Y centerline
ina_post_h      = 2.5;                                // Standoff height above base floor (mm)
ina_pcb_z       = wall + ina_post_h;                  // 4.5 mm (PCB bottom Z)

// ----------------------------------------------------------------------------
// 4. Render Mode Selection (CLI & Interactive Compatible)
// ----------------------------------------------------------------------------
// Unquoted CLI token compatibility constants:
base        = "base";
lid         = "lid";
lid_print   = "lid_print";
print       = "print";
assembled   = "assembled";
exploded    = "exploded";
ghost       = "ghost";

// Modes:
//   "exploded"  : Base & Lid separated vertically along Z
//   "assembled" : Base & Lid closed together in mated state
//   "base"      : Base only (flat on bed, ready to print)
//   "lid"       : Lid only (in assembled top orientation)
//   "lid_print" : Lid flipped 180 deg (flat on bed for support-free printing)
//   "print"     : Both Base and flipped Lid side-by-side on build plate
//   "ghost"     : Assembled transparent view with ghost PCBs for verification
mode = "exploded";

if (mode == "base") {
    base();
} else if (mode == "lid") {
    lid();
} else if (mode == "lid_print") {
    translate([0, case_l, lid_h]) rotate([180, 0, 0]) lid();
} else if (mode == "print") {
    base();
    translate([case_w + 10, case_l, lid_h]) rotate([180, 0, 0]) lid();
} else if (mode == "assembled") {
    base();
    translate([0, 0, base_h]) lid();
} else if (mode == "ghost") {
    %base();
    %translate([0, 0, base_h]) lid();
    ghost_hardware();
} else { // "exploded"
    base();
    translate([0, 0, base_h + 10]) lid();
}

// ============================================================================
// Helper Modules
// ============================================================================

// Rounded rectangular prism bounded strictly on [0..w, 0..l, 0..h]
module rounded_box(w, l, h, r = corner_r) {
    r_clamped = max(0.1, min(r, min(w, l) / 2));
    hull() {
        translate([r_clamped, r_clamped, 0])         cylinder(r = r_clamped, h = h);
        translate([w - r_clamped, r_clamped, 0])     cylinder(r = r_clamped, h = h);
        translate([r_clamped, l - r_clamped, 0])     cylinder(r = r_clamped, h = h);
        translate([w - r_clamped, l - r_clamped, 0]) cylinder(r = r_clamped, h = h);
    }
}

// Rounded 2D slot extruded in Z
module rounded_slot_z(w, l, h) {
    r = min(w, l) / 2;
    hull() {
        if (w >= l) {
            translate([r, r, 0])     cylinder(r = r, h = h);
            translate([w - r, r, 0]) cylinder(r = r, h = h);
        } else {
            translate([r, r, 0])     cylinder(r = r, h = h);
            translate([r, l - r, 0]) cylinder(r = r, h = h);
        }
    }
}

// ============================================================================
// Module: Base (Lower Enclosure Half)
// ============================================================================
module base() {
    union() {
        difference() {
            // 1. Outer rounded shell (0..case_w, 0..case_l, 0..base_h)
            rounded_box(case_w, case_l, base_h, corner_r);

            // 2. Main interior cavity (uniform 2.0 mm wall & floor)
            translate([wall, wall, wall])
                rounded_box(case_w - 2*wall, case_l - 2*wall, base_h, max(0.2, corner_r - wall));

            // 3. Upper perimeter counterbore step to receive Lid mating skirt
            //    Leaves a 1.4 mm outer wall rim and receives the lid lip with 0.6 mm clearance
            translate([wall - 0.6, wall - 0.6, base_h - lip_h])
                rounded_box(case_w - 2*(wall - 0.6), case_l - 2*(wall - 0.6), lip_h + 1.0, max(0.4, corner_r - wall + 0.6));

            // 4. Snap-fit undercut detent grooves on Front (-Y), Rear (+Y), and Left (-X) inner walls
            //    Front snap groove (centered at X = longan_x + longan_l/2)
            translate([longan_x + longan_l/2 - (snap_len + 2)/2, wall - 0.6 - snap_undercut, base_h - lip_h + 0.4])
                cube([snap_len + 2, snap_undercut + 1.0, 1.0]);
            //    Rear snap groove (matching Lid rear snap tab at X = 24..31 mm)
            translate([23.0, case_l - wall + 0.6 - 1.0, base_h - lip_h + 0.4])
                cube([9.0, snap_undercut + 1.0, 1.0]);
            //    Left (-X) wall snap groove (centered at Y = 40 mm)
            translate([wall - 0.6 - snap_undercut, 40 - 6, base_h - lip_h + 0.4])
                cube([snap_undercut + 1.0, 12, 1.0]);

            // 5. Fingernail pry notches at left and right seam lines
            translate([-0.5, case_l/2 - 4, base_h - 0.8])
                cube([1.3, 8.0, 1.6]);
            translate([case_w - 0.8, case_l - 15, base_h - 0.8])
                cube([1.3, 8.0, 1.6]);

            // 6. Longan Nano USB-C Wall Cutout (11.0 mm x 5.0 mm on Right +X wall)
            //    Protrudes 3.0 mm from right edge of Longan Nano PCB (Z = 9.0..14.0 mm)
            translate([case_w - wall - 1.5, longan_cy - usbc_cut_w/2, base_h - usbc_cut_h])
                cube([wall + 3.0, usbc_cut_w, usbc_cut_h + 1.0]);

            // 7. Longan Nano MicroSD Card Access Slot (13.0 mm x 3.0 mm)
            //    (a) Side card access slot on Left (-X) wall aligned with bottom SD plane (Z = 6.2..9.2 mm)
            translate([-1.0, longan_cy - sd_cut_w/2, longan_pcb_z - 2.8])
                cube([wall + 2.5, sd_cut_w, sd_cut_h]);
            //    (b) Bottom direct finger/tweezers card access slot directly under MicroSD socket
            translate([longan_x + 21.0, longan_cy - sd_cut_w/2, -0.5])
                cube([16.0, sd_cut_w, wall + 1.0]);

            // 8. Blue INA219 Terminal Block Wire Entry Window (11.0 mm x 9.5 mm on Top/Rear +Y wall)
            //    Aligned with 2-pin 3.5 mm green screw terminal block at X = ina_cx
            translate([ina_cx - term_cut_w/2, case_l - wall - 1.5, base_h - term_cut_h])
                cube([term_cut_w, wall + 3.0, term_cut_h + 1.0]);
        }

        // 9. Longan Nano Precision Mounting Cradle & Center-Notch Indexing Posts
        longan_cradle();

        // 10. Blue INA219 4x M2.5 Corner Mounting Standoffs (21.0 mm x 17.8 mm spacing)
        ina219_mounts();
    }
}

// Longan Nano PCB Cradle:
// Supports PCB at Z = 9.0 mm (7.0 mm bottom clearance for MicroSD socket & header pins),
// retains perimeter with 0.6 mm clearance, and indexes the 3.8 mm center side notches.
module longan_cradle() {
    tol = clearance / 2; // 0.3 mm per side (total 0.6 mm clearance)
    lip_z = 1.4;         // Retaining wall height above PCB bottom
    z_overlap = 0.5;     // Overlap into base floor for watertight CSG union

    // 4 Corner L-shaped PCB support pillars + integrated end retaining rails
    for (end_idx = [0, 1]) {
        cx = (end_idx == 0) ? longan_x : (longan_x + longan_l - 4.5);
        rx = (end_idx == 0) ? (longan_x - tol) : (longan_x + longan_l - 6.0);

        // Front corner support pad + retaining rail
        translate([cx, longan_y, wall - z_overlap])
            cube([4.5, 3.2, longan_post_h + z_overlap]);
        translate([rx, longan_y - tol - 1.4, wall - z_overlap])
            cube([6.0, 1.4 + tol + 0.4, longan_post_h + lip_z + z_overlap]);

        // Back corner support pad + retaining rail
        translate([cx, longan_y + longan_w - 3.2, wall - z_overlap])
            cube([4.5, 3.2, longan_post_h + z_overlap]);
        translate([rx, longan_y + longan_w - 0.4, wall - z_overlap])
            cube([6.0, 1.4 + tol + 0.4, longan_post_h + lip_z + z_overlap]);
    }

    // Left (-X) stop pillars at front-left and back-left corners (leaving center open for MicroSD)
    translate([longan_x - tol - 1.6, longan_y - tol - 1.4, wall - z_overlap])
        cube([1.8, 4.6 + tol, longan_post_h + lip_z + z_overlap]);
    translate([longan_x - tol - 1.6, longan_y + longan_w - 3.2, wall - z_overlap])
        cube([1.8, 4.6 + tol, longan_post_h + lip_z + z_overlap]);

    // Center semi-circular index pegs engaging Longan Nano's 3.8 mm side notches
    notch_peg_d = notch_d - clearance; // 3.2 mm diameter peg inside 3.8 mm notch
    // Front edge notch peg (Y = longan_y)
    translate([longan_x + notch_x_ofs, longan_y - tol, wall - z_overlap]) {
        cylinder(d = notch_peg_d, h = longan_post_h + lip_z + z_overlap);
        translate([-2.5, -1.6, 0]) cube([5.0, 1.6, longan_post_h + lip_z + z_overlap]);
    }
    // Back edge notch peg (Y = longan_y + longan_w)
    translate([longan_x + notch_x_ofs, longan_y + longan_w + tol, wall - z_overlap]) {
        cylinder(d = notch_peg_d, h = longan_post_h + lip_z + z_overlap);
        translate([-2.5, 0, 0]) cube([5.0, 1.6, longan_post_h + lip_z + z_overlap]);
    }
}

// Blue INA219 4x M2.5 Corner Mounting Bosses (17.8 mm X x 21.0 mm Y center spacing)
module ina219_mounts() {
    z_overlap = 0.5;
    for (dx = [-ina_hole_dx/2, ina_hole_dx/2]) {
        for (dy = [-ina_hole_dy/2, ina_hole_dy/2]) {
            translate([ina_cx + dx, ina_cy + dy, wall - z_overlap]) {
                difference() {
                    union() {
                        // Base support standoff (supports PCB at Z = ina_pcb_z)
                        cylinder(d = 5.0, h = ina_post_h + z_overlap);
                        // M2.5 alignment pin (2.3 mm base tapering to 2.0 mm tip)
                        cylinder(d1 = ina_hole_d - 0.2, d2 = ina_hole_d - 0.5, h = ina_post_h + ina_t + 1.2 + z_overlap);
                    }
                    // M2 pilot bore through boss
                    translate([0, 0, 0.2])
                        cylinder(d = 1.2, h = ina_post_h + ina_t + 2.0 + z_overlap);
                }
            }
        }
    }

    // Lateral cradle ribs keeping INA219 square with 0.6 mm clearance
    tol = clearance / 2;
    translate([ina_x - tol - 1.2, ina_y + 4.0, wall - z_overlap])
        cube([1.2, ina_l - 8.0, ina_post_h + ina_t + z_overlap]);
    translate([ina_x + ina_w + tol, ina_y + 4.0, wall - z_overlap])
        cube([1.2, ina_l - 8.0, ina_post_h + ina_t + z_overlap]);
}

// ============================================================================
// Module: Lid (Upper Enclosure Cover)
// ============================================================================
module lid() {
    // Computed feature coordinates in lid space (0..case_w, 0..case_l, 0..lid_h)
    lcd_glass_x   = longan_x + lcd_glass_x_ofs;              // 36.4 mm
    lcd_glass_y   = longan_cy - lcd_glass_w / 2;             // 8.25 mm
    lcd_win_x     = lcd_glass_x + (lcd_glass_l - lcd_win_l)/2; // 38.15 mm
    lcd_win_y     = longan_cy - lcd_win_w / 2;               // 8.5 mm

    // Terminal screw centers (2-pin 3.5 mm terminal block on top edge of INA219)
    // Spaced at 4.3 mm so dual Ø 4.0 mm holes cleanly expose the 3.5 mm pitch screw heads
    screw_y       = ina_y + ina_l - term_d / 2;              // 48.7 mm
    screw_dx      = 2.15;

    // Thermal slot center over R100 shunt resistor & INA219 sense path
    vent_center_y = ina_y + 11.2;                            // 38.4 mm
    z_overlap     = 0.5;

    union() {
        difference() {
            union() {
                // 1. Hollowed outer lid shell (0..case_w, 0..case_l, 0..lid_h)
                difference() {
                    rounded_box(case_w, case_l, lid_h, corner_r);
                    translate([wall, wall, -1.0])
                        rounded_box(case_w - 2*wall, case_l - 2*wall, lid_h - wall + 1.0, max(0.2, corner_r - wall));
                }

                // 2. LCD Display Support Shelf Rim (fused 0.5 mm into Lid ceiling)
                translate([lcd_glass_x - 1.5, lcd_glass_y - 1.5, lid_h - wall - lcd_pocket_d])
                    cube([lcd_glass_l + 3.0, lcd_glass_w + 3.0, lcd_pocket_d + z_overlap]);

                // 3. Button Guide Sleeves around BOOT0 & RESET (fused 0.5 mm into Lid ceiling)
                for (by = [longan_y + btn_reset_y, longan_y + btn_boot_y]) {
                    translate([longan_x + btn_x_ofs, by, lid_h - wall - lcd_pocket_d])
                        cylinder(d = btn_hole_d + 2.4, h = lcd_pocket_d + z_overlap);
                }

                // 4. Snap-Fit Interlocking Skirt & Cantilever Detent Tabs
                lid_snap_skirt();
            }

            // 5. LCD Viewing Window Cutout (24.5 mm x 14.0 mm through top ceiling)
            translate([lcd_win_x, lcd_win_y, lid_h - wall - lcd_pocket_d - 1.0])
                cube([lcd_win_l, lcd_win_w, wall + lcd_pocket_d + 2.0]);

            // 6. Inner Recessed Pocket for 28.0 x 14.5 mm Outer LCD Glass (+ 0.6 mm clearance)
            translate([lcd_glass_x - clearance/2, lcd_glass_y - clearance/2, lid_h - wall - lcd_pocket_d - 0.5])
                cube([lcd_glass_l + clearance, lcd_glass_w + clearance, lcd_pocket_d + 0.5]);

            // 7. CRITICAL: 4.0 x 6.0 x 1.8 mm Relief Pocket on Left (-X) Edge for Folded 8-Pin FPC Ribbon
            translate([lcd_glass_x - 2.5, longan_cy - fpc_relief_w/2, lid_h - wall - fpc_relief_d - 0.5])
                cube([fpc_relief_l, fpc_relief_w, fpc_relief_d + 0.5]);

            // 8. Tactile Button Access Holes: RESET & BOOT0 (2x Ø 4.0 mm near USB-C)
            for (by = [longan_y + btn_reset_y, longan_y + btn_boot_y]) {
                translate([longan_x + btn_x_ofs, by, lid_h - wall - lcd_pocket_d - 1.0])
                    cylinder(d = btn_hole_d, h = wall + lcd_pocket_d + 2.0);
            }

            // 9. Dual Ø 4.0 mm Screwdriver Access Holes over INA219 Terminal Block Screws
            translate([ina_cx - screw_dx, screw_y, lid_h - wall - 1.0])
                cylinder(d = term_screw_d, h = wall + 2.0);
            translate([ina_cx + screw_dx, screw_y, lid_h - wall - 1.0])
                cylinder(d = term_screw_d, h = wall + 2.0);

            // 10. 4x 3.5 mm Thermal Ventilation Slots over R100 Shunt Resistor (1W @ 3.2A)
            for (i = [0 : vent_count - 1]) {
                slot_x = ina_cx - ((vent_count - 1) * vent_pitch)/2 + i * vent_pitch - vent_slot_t/2;
                translate([slot_x, vent_center_y - 4.0, lid_h - wall - 1.0])
                    rounded_slot_z(vent_slot_t, 8.0, wall + 2.0);
            }

            //     Additional 4x 3.5 mm rear exhaust ventilation slots on Rear (+Y) Lid wall
            for (i = [0 : vent_count - 1]) {
                translate([32.0 + i * 7.5, case_l - wall - 1.0, 1.8])
                    cube([vent_slot_w, wall + 2.0, lid_h - wall - 2.8]);
            }
        }
    }
}

// Snap-fit skirt and cantilever tabs on Lid (anchored to both Lid wall and ceiling for zero overhang)
module lid_snap_skirt() {
    inset = wall - 0.6 + clearance / 2; // 1.7 mm inset from outer perimeter (0.3 mm per-side clearance)
    tab_total_h = lid_h - wall + lip_h + 0.5; // Extends from ceiling (6.5 mm) down to -lip_h (-2.2 mm)

    // Front (-Y) snap-fit tab with outward snap bead
    translate([longan_x + longan_l/2 - snap_len/2, inset, -lip_h]) {
        cube([snap_len, lip_thick, tab_total_h]);
        translate([0, -snap_undercut, 0.4])
            hull() {
                cube([snap_len, snap_undercut + 0.1, 0.5]);
                translate([0, snap_undercut, -0.4])
                    cube([snap_len, 0.1, 1.3]);
            }
    }

    // Rear-Right (+Y) snap-fit tab with outward snap bead
    translate([24.0, case_l - inset - lip_thick, -lip_h]) {
        cube([7.0, lip_thick, tab_total_h]);
        translate([0, lip_thick - 0.1, 0.4])
            hull() {
                cube([7.0, snap_undercut + 0.1, 0.5]);
                translate([0, 0, -0.4])
                    cube([7.0, 0.1, 1.3]);
            }
    }

    // Left (-X) snap-fit tab with outward snap bead
    translate([inset, 40 - 5, -lip_h]) {
        cube([lip_thick, 10.0, tab_total_h]);
        translate([-snap_undercut, 0, 0.4])
            hull() {
                cube([snap_undercut + 0.1, 10.0, 0.5]);
                translate([snap_undercut, 0, -0.4])
                    cube([0.1, 10.0, 1.3]);
            }
    }

    // Front-Left alignment lip guide
    translate([6.0, inset, -lip_h + 0.4])
        cube([16.0, lip_thick, tab_total_h - 0.4]);
}

// ============================================================================
// Ghost Hardware Visualization Module (for Fit & Alignment Verification)
// ============================================================================
module ghost_hardware() {
    // 1. Sipeed Longan Nano PCB
    translate([longan_x, longan_y, longan_pcb_z]) {
        color([0.85, 0.15, 0.15])
            cube([longan_l, longan_w, longan_t]);
        // 0.96" LCD Outer Glass
        color([0.1, 0.1, 0.15])
            translate([lcd_glass_x_ofs, (longan_w - lcd_glass_w)/2, longan_t])
                cube([lcd_glass_l, lcd_glass_w, 3.0]);
        // 8-pin FPC Ribbon (folded over left edge)
        color([0.85, 0.55, 0.1])
            translate([lcd_glass_x_ofs - 3.5, (longan_w - 5.5)/2, longan_t])
                cube([3.5, 5.5, 2.8]);
        // USB-C Receptacle (protrudes 3.0 mm past right edge)
        color([0.75, 0.75, 0.78])
            translate([longan_l - 4.5, (longan_w - 8.94)/2, longan_t])
                cube([4.5 + usbc_protrude, 8.94, 3.16]);
        // BOOT0 & RESET Buttons
        color([0.9, 0.9, 0.9]) {
            translate([btn_x_ofs, btn_reset_y, longan_t]) cylinder(d = 2.6, h = 2.5);
            translate([btn_x_ofs, btn_boot_y, longan_t])  cylinder(d = 2.6, h = 2.5);
        }
        // Bottom MicroSD Socket
        color([0.7, 0.7, 0.72])
            translate([27.5, (longan_w - 11.5)/2, -1.8])
                cube([11.5, 11.5, 1.8]);
    }

    // 2. Blue INA219 Breakout PCB
    translate([ina_x, ina_y, ina_pcb_z]) {
        color([0.1, 0.35, 0.65])
            difference() {
                cube([ina_w, ina_l, ina_t]);
                for (dx = [-ina_hole_dx/2, ina_hole_dx/2])
                    for (dy = [-ina_hole_dy/2, ina_hole_dy/2])
                        translate([ina_w/2 + dx, ina_l/2 + dy, -0.5])
                            cylinder(d = 2.8, h = ina_t + 1.0);
            }
        // 2-Pin 3.5 mm Green Screw Terminal Block on Top (+Y) Edge
        color([0.15, 0.7, 0.3])
            translate([(ina_w - term_w)/2, ina_l - term_d, ina_t])
                cube([term_w, term_d, term_h]);
        // R100 Shunt Resistor
        color([0.15, 0.15, 0.15])
            translate([(ina_w - 6.3)/2, 11.2 - 1.6, ina_t])
                cube([6.3, 3.2, 1.2]);
    }
}
