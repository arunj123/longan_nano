// ============================================================================
// Parametric Two-Piece Snap-Fit Enclosure for DC Power Monitor (Rev 3)
// Hardware Modules:
//   1. Sipeed Longan Nano (GD32VF103CBT6 RISC-V) with 0.96" 160x80 IPS LCD
//   2. Blue INA219 High-Side DC Current Sensor Breakout
//
// Authoritative Mechanical Reference & Blueprints: docs/HARDWARE_CASE_DESIGN_SPECS.md
//
// Empirical Fit Refinements (Verified on Physical Rev 3 Prints case_rev3_*.jpg):
//   - Full 360-degree perimeter interlocking lip + 4-wall snap detents (zero X/Y slide)
//   - Removed MicroSD card slots (USB MSC Drive Mode is used instead)
//   - Ultra-low-profile flat Lid (reduced lid_h from 8.0 mm to 2.4 mm; total height 16.6 mm)
//     bringing the LCD glass within 1.4 mm of the top surface and buttons within fingertip reach
//   - Calibrated LCD window (22.0 x 11.4 mm) & button holes (X = 35.75 mm, Y = 6.3/13.7 mm)
//     to the exact 2.54 mm pin header grid of the Longan Nano PCB
//   - Widened/lengthened Longan Nano cradle (47.2 x 21.0 mm) & removed pinching notch pegs
//   - Corrected Blue INA219 orientation & 2.54 mm grid mounting holes:
//     25.5 mm (X width) x 22.3 mm (Y length), hole spacing 20.4 mm (X) x 17.2 mm (Y),
//     tapered Ø 1.9->1.5 mm poles, and 4.0 mm under-board solder clearance
// ============================================================================

$fn = 64; // Smooth cylinder tessellation

// ----------------------------------------------------------------------------
// 1. Global Enclosure Constraints & Print Tolerances
// ----------------------------------------------------------------------------
case_w          = 75.0;  // Overall enclosure width (X-axis, mm)
case_l          = 55.0;  // Overall enclosure length/depth (Y-axis, mm)
base_h          = 14.2;  // Lower Base shell height (mm)
lid_h           = 2.4;   // Ultra-low-profile flat Upper Lid thickness (mm)
case_h          = base_h + lid_h; // Overall assembled height (16.6 mm)

wall            = 2.0;   // Outer wall and floor thickness (mm)
clearance       = 0.8;   // PCB cradle fit clearance (0.4 mm per side, mm)
corner_r        = 2.0;   // Outer vertical corner fillet radius (mm)

// 360-degree interlocking perimeter lip & 4-side snap-fit parameters
lip_h           = 2.0;   // Height of lid mating skirt extending into base (mm)
lip_thick       = 1.0;   // Thickness of mating skirt wall (mm)
lip_clearance   = 0.25;  // Per-side radial clearance between base step and lid skirt (mm)
snap_undercut   = 0.40;  // Radial engagement of snap-fit ridge (mm)
snap_len        = 12.0;  // Width of front/back snap-fit tabs (mm)

// ----------------------------------------------------------------------------
// 2. Sipeed Longan Nano Hardware Specifications (Calibrated to 2.54 mm Grid)
// ----------------------------------------------------------------------------
longan_l        = 46.2;  // Nominal PCB length including routing tolerance (X-axis, mm)
longan_w        = 20.0;  // Nominal PCB width (Y-axis, mm)
longan_t        = 1.0;   // FR4 PCB thickness (mm)

// LCD Module (0.96" 160x80 IPS, 13.5 mm breadth x 28.0 mm length)
// Physical print verification (case_rev3_assembled_top.jpg & case_rev3_internal_layout.jpg):
//   - Outer LCD module body: 28.0 mm length (X_pcb = 7.53..35.53 mm) x 13.5 mm breadth
//   - Top viewport cutout: 27.2 mm inner / 28.0 mm beveled length x 11.4 mm inner / 12.6 mm beveled breadth
//   - Underside pocket: 28.9 mm length (X_pcb = 6.80..35.70 mm) x 15.2 mm breadth x 1.4 mm depth
lcd_glass_x_ofs = 7.4;   // Left (-X) edge of 28.0 mm LCD module body from PCB left edge (mm)
lcd_glass_l     = 28.0;  // Full black LCD module length (7.4..35.4 mm, empirically measured 28.0 mm)
lcd_glass_w     = 14.2;  // Outer LCD module width (13.5 mm) + foam tape margin (mm)
lcd_win_cx_ofs  = 21.10; // Center of LCD top window from PCB left edge (spans X_pcb = 7.50..34.70 mm)
lcd_win_l       = 27.2;  // Top viewport inner length (27.2 mm inner, 28.0 mm with top bevel)
lcd_win_w       = 11.4;  // Top viewport inner breadth (11.4 mm inner, 12.6 mm with top bevel; fits breadth)
lcd_pocket_d    = 1.4;   // Underside recess depth in lid ceiling for 13.5x28.0 mm LCD module (mm)

// 8-pin FPC Display Flex Cable Relief Pocket (folded over left edge of LCD, X_pcb = 3.0..8.0 mm)
fpc_relief_l    = 5.0;   // Relief pocket X length (mm)
fpc_relief_w    = 8.5;   // Relief pocket Y width (mm)
fpc_relief_d    = 1.4;   // Relief pocket Z depth into lid underside (mm)

// USB Type-C Port (protrudes ~3.0 mm from right edge of Longan Nano)
usbc_protrude   = 3.0;   // Protrusion past right PCB edge (mm)
usbc_cut_w      = 11.5;  // Wall cutout width (Y-axis, mm)
usbc_cut_h      = 4.8;   // Wall cutout height in Base (Z = 9.4..14.2 mm)

// Tactile Pushbuttons (BOOT0 and RESET, calibrated to case_rev3_assembled_active.jpg & 2.54 mm grid)
btn_hole_d      = 4.0;   // Button access hole diameter (mm)
btn_x_ofs       = 37.50; // Calibrated button center X offset from Longan left edge (+1.75 mm toward +X USB-C)
btn_reset_y     = 6.45;  // RESET button Y offset from Longan front (-Y) edge (mm)
btn_boot_y      = 13.30; // BOOT0 button Y offset from Longan front (-Y) edge (mm)

// Placement of Longan Nano in Right/Front Bay (0.5 mm standoff from +X Right inner wall)
longan_right_gap = 0.5;
longan_x        = case_w - wall - longan_right_gap - longan_l; // 26.3 mm (right edge at 72.5 mm)
longan_y        = 5.5;                      // Front bay Y origin (spans 5.5..25.5 mm)
longan_cy       = longan_y + longan_w / 2;  // 15.5 mm centerline
longan_post_h   = 7.5;                      // Standoff height above base floor (mm)
longan_pcb_z    = wall + longan_post_h;     // 9.5 mm (PCB bottom Z; PCB top = 10.5 mm)

// ----------------------------------------------------------------------------
// 3. Blue INA219 Current Sensor Breakout Specifications (Calibrated to case_rev3_internal_layout.jpg)
// ----------------------------------------------------------------------------
// Orientation in case (matching IMG_8183 & IMG_8184):
//   - VIN- / VIN+ sense pads at Rear (+Y) wall
//   - 6-pin header (VCC, GND, SCL, SDA, VIN-, VIN+) at Front (-Y) side
//   - Width along X-axis = 25.5 mm; Length along Y-axis = 22.3 mm
ina_w           = 25.5;  // PCB width along X-axis (mm)
ina_l           = 22.3;  // PCB length along Y-axis (mm)
ina_t           = 1.6;   // PCB thickness (mm)

// 4x Corner Mounting Holes (on 2.54 mm header grid: 8 x 2.54 = 20.32 mm X, 17.2 mm Y)
ina_hole_dx     = 20.4;  // Center-to-center spacing along X-axis (mm)
ina_hole_dy     = 17.2;  // Center-to-center spacing along Y-axis (mm)
ina_pin_d1      = 1.9;   // Alignment pole base diameter (fits 2.5-2.8 mm holes easily)
ina_pin_d2      = 1.5;   // Alignment pole tapered tip diameter (mm)

// 2-Pin 3.5 mm Terminal / Sense Wire Cutout & Access Holes
term_w          = 10.0;  // Terminal block width (X-axis, mm)
term_d          = 8.0;   // Terminal block depth (Y-axis, mm)
term_cut_w      = 11.5;  // Rear wall wire entry window width (mm)
term_cut_h      = 8.2;   // Rear wall wire entry window height in Base (Z = 6.0..14.2 mm)
term_screw_d    = 4.0;   // Dual top access hole diameter on lid (mm)

// Thermal Ventilation Slots over R100 Shunt Resistor (1W dissipation at 3.2A)
vent_count      = 4;     // Number of ventilation slots
vent_slot_w     = 3.5;   // Characteristic slot width/length (mm)
vent_slot_t     = 1.6;   // Slot narrow opening width (mm)
vent_pitch      = 3.2;   // Center-to-center spacing of thermal slots (mm)

// Placement of Blue INA219 in Left/Rear Bay (VIN+/VIN- flush with +Y Rear wall)
ina_x           = wall + 2.5;                  // 4.5 mm (spans X = 4.5..30.0 mm)
ina_y           = case_l - wall - 0.9 - ina_l; // 29.8 mm (spans Y = 29.8..52.1 mm)
ina_cx          = ina_x + ina_w / 2;           // 17.25 mm X centerline
ina_cy          = ina_y + ina_l / 2;           // 40.95 mm Y centerline
ina_post_h      = 4.0;                         // 4.0 mm standoff for bottom solder/wire clearance
ina_pcb_z       = wall + ina_post_h;           // 6.0 mm (PCB bottom Z; PCB top = 7.6 mm)

// ----------------------------------------------------------------------------
// 4. Render Mode Selection (CLI & Interactive Compatible)
// ----------------------------------------------------------------------------
base        = "base";
lid         = "lid";
lid_print   = "lid_print";
print       = "print";
assembled   = "assembled";
exploded    = "exploded";
ghost       = "ghost";

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
    step_wall = 1.2; // Outer rim thickness at top counterbore step (mm)

    union() {
        difference() {
            // 1. Outer rounded shell (0..case_w, 0..case_l, 0..base_h)
            rounded_box(case_w, case_l, base_h, corner_r);

            // 2. Main interior cavity (uniform 2.0 mm wall & floor)
            translate([wall, wall, wall])
                rounded_box(case_w - 2*wall, case_l - 2*wall, base_h, max(0.2, corner_r - wall));

            // 3. Full 360-degree upper perimeter counterbore step to lock Lid skirt in X & Y
            translate([step_wall, step_wall, base_h - lip_h])
                rounded_box(
                    case_w - 2*step_wall,
                    case_l - 2*step_wall,
                    lip_h + 1.0,
                    max(0.4, corner_r - step_wall)
                );

            // 4. Symmetric 4-Wall Snap-Fit Detent Grooves (Front -Y, Rear +Y, Left -X, Right +X)
            //    Front (-Y) snap groove
            translate([longan_x + longan_l/2 - (snap_len + 2)/2, step_wall - snap_undercut, base_h - lip_h + 0.35])
                cube([snap_len + 2, snap_undercut + 1.0, 0.9]);
            //    Rear (+Y) snap groove (X = 41..55 mm)
            translate([41.0, case_l - step_wall - 1.0, base_h - lip_h + 0.35])
                cube([snap_len + 2, snap_undercut + 1.0, 0.9]);
            //    Left (-X) wall snap groove (Y = 20..34 mm)
            translate([step_wall - snap_undercut, 20.0, base_h - lip_h + 0.35])
                cube([snap_undercut + 1.0, 14.0, 0.9]);
            //    Right (+X) wall snap groove (Y = 30..44 mm, preventing any X/Y slide)
            translate([case_w - step_wall - 1.0, 30.0, base_h - lip_h + 0.35])
                cube([snap_undercut + 1.0, 14.0, 0.9]);

            // 5. Fingernail pry notches at left and right seam lines
            translate([-0.5, case_l/2 - 4, base_h - 0.7])
                cube([1.1, 8.0, 1.4]);
            translate([case_w - 0.6, case_l - 12, base_h - 0.7])
                cube([1.1, 8.0, 1.4]);

            // 6. Longan Nano USB-C Wall Cutout (11.5 mm x 4.8 mm on Right +X wall)
            //    Aligned with PCB top at Z = 10.5 mm (cutout spans Z = 9.4..14.2 mm)
            translate([case_w - wall - 1.5, longan_cy - usbc_cut_w/2, base_h - usbc_cut_h])
                cube([wall + 3.0, usbc_cut_w, usbc_cut_h + 1.0]);

            // (Note: MicroSD card slots intentionally omitted — USB MSC Drive Mode is used)

            // 7. Blue INA219 Sense Wire / Terminal Window (11.5 mm x 8.2 mm on Rear +Y wall)
            //    Aligned with VIN- / VIN+ pads at X = ina_cx (cutout spans Z = 6.0..14.2 mm)
            translate([ina_cx - term_cut_w/2, case_l - wall - 1.5, base_h - term_cut_h])
                cube([term_cut_w, wall + 3.0, term_cut_h + 1.0]);
        }

        // 8. Longan Nano Precision Corner Cradle (47.2 x 21.0 mm pocket, no pinching notch pegs)
        longan_cradle();

        // 9. Blue INA219 4x Corner Mounting Standoffs (20.4 mm X x 17.2 mm Y spacing)
        ina219_mounts();
    }
}

// Longan Nano PCB Cradle:
// Supports PCB at Z = 9.5 mm (7.5 mm bottom clearance for MicroSD socket & solder joints).
// Provides generous 0.5 mm per-side clearance (47.2 x 21.0 mm interior pocket),
// leaves left center wide open for the 4-pin JTAG header, and omits center-notch pegs
// that previously pinched the 17.8 mm notch root.
module longan_cradle() {
    tol_x = 0.5;         // 0.5 mm X clearance on left and right (total 47.2 mm pocket)
    tol_y = 0.5;         // 0.5 mm Y clearance on front and back (total 21.0 mm pocket)
    lip_z = 1.6;         // Retaining wall height above PCB bottom (reaches Z = 11.1 mm)
    z_overlap = 0.5;     // Overlap into base floor for watertight CSG union

    // 4 Corner L-shaped PCB support pads + front/back retaining rails
    for (end_idx = [0, 1]) {
        pad_x  = (end_idx == 0) ? longan_x : (longan_x + longan_l - 4.5);
        rail_x = (end_idx == 0) ? (longan_x - tol_x) : (longan_x + longan_l + tol_x - 6.5);

        // Front (-Y) corner support pad + outer retaining wall
        translate([pad_x, longan_y - tol_y, wall - z_overlap])
            cube([4.5, 2.8 + tol_y, longan_post_h + z_overlap]);
        translate([rail_x, longan_y - tol_y - 1.4, wall - z_overlap])
            cube([6.5, 1.4, longan_post_h + lip_z + z_overlap]);

        // Back (+Y) corner support pad + outer retaining wall
        translate([pad_x, longan_y + longan_w - 2.8, wall - z_overlap])
            cube([4.5, 2.8 + tol_y, longan_post_h + z_overlap]);
        translate([rail_x, longan_y + longan_w + tol_y, wall - z_overlap])
            cube([6.5, 1.4, longan_post_h + lip_z + z_overlap]);
    }

    // Center support pillars under PCB edges (flat top at Z = longan_pcb_z, no pinching pegs)
    translate([longan_x + longan_l/2 - 3.0, longan_y - tol_y - 1.4, wall - z_overlap]) {
        cube([6.0, 2.6 + tol_y, longan_post_h + z_overlap]);
        cube([6.0, 1.4, longan_post_h + lip_z + z_overlap]);
    }
    translate([longan_x + longan_l/2 - 3.0, longan_y + longan_w - 1.2, wall - z_overlap]) {
        cube([6.0, 2.6 + tol_y, longan_post_h + z_overlap]);
        translate([0, 1.2 + tol_y, 0])
            cube([6.0, 1.4, longan_post_h + lip_z + z_overlap]);
    }

    // Left (-X) end stop pillars at front-left & back-left corners
    // (Leaves a 12.6 mm center gap at Y = 9.2..21.8 mm for the 4-pin JTAG header!)
    translate([longan_x - tol_x - 1.5, longan_y - tol_y - 1.4, wall - z_overlap])
        cube([1.5, 3.7 + tol_y, longan_post_h + lip_z + z_overlap]);
    translate([longan_x - tol_x - 1.5, longan_y + longan_w - 2.3, wall - z_overlap])
        cube([1.5, 3.7 + tol_y, longan_post_h + lip_z + z_overlap]);

    // Right (+X) end stop ribs next to inner right wall (keeps PCB at exact X reference)
    translate([longan_x + longan_l + tol_x, longan_y - tol_y - 1.4, wall - z_overlap])
        cube([case_w - wall - (longan_x + longan_l + tol_x) + 0.2, 4.0 + tol_y, longan_post_h + lip_z + z_overlap]);
    translate([longan_x + longan_l + tol_x, longan_y + longan_w - 2.6, wall - z_overlap])
        cube([case_w - wall - (longan_x + longan_l + tol_x) + 0.2, 4.0 + tol_y, longan_post_h + lip_z + z_overlap]);
}

// Blue INA219 4x Corner Mounting Standoffs (20.4 mm X x 17.2 mm Y center spacing)
// + 26.7 x 23.5 mm lateral guide ribs (0.6 mm clearance per side around 25.5 x 22.3 mm PCB)
module ina219_mounts() {
    z_overlap = 0.5;
    tol = 0.6; // 0.6 mm per-side clearance (1.2 mm total) so PCB never touches walls

    for (dx = [-ina_hole_dx/2, ina_hole_dx/2]) {
        for (dy = [-ina_hole_dy/2, ina_hole_dy/2]) {
            translate([ina_cx + dx, ina_cy + dy, wall - z_overlap]) {
                // Base support standoff (Ø 4.6 mm flat shoulder at Z = ina_pcb_z)
                cylinder(d = 4.6, h = ina_post_h + z_overlap);
                // Tapered alignment pole (Ø 1.9 mm base -> Ø 1.5 mm tip, passes smoothly through Ø 2.6 mm holes)
                cylinder(d1 = ina_pin_d1, d2 = ina_pin_d2, h = ina_post_h + ina_t + 1.6 + z_overlap);
            }
        }
    }

    // Left (-X) and Right (+X) lateral guide ribs (spaced 25.5 + 1.2 = 26.7 mm apart)
    translate([ina_x - tol - 1.2, ina_y + 4.0, wall - z_overlap])
        cube([1.2, ina_l - 8.0, ina_post_h + ina_t + z_overlap]);
    translate([ina_x + ina_w + tol, ina_y + 4.0, wall - z_overlap])
        cube([1.2, ina_l - 8.0, ina_post_h + ina_t + z_overlap]);
}

// ============================================================================
// Module: Lid (Ultra-Low-Profile Flat Upper Cover, 2.4 mm Plate + 2.0 mm Skirt)
// ============================================================================
module lid() {
    // Computed feature coordinates in lid space (0..case_w, 0..case_l, 0..lid_h)
    lcd_glass_x   = longan_x + lcd_glass_x_ofs;              // 33.7 mm (X_pcb = 7.4 mm)
    lcd_glass_y   = longan_cy - lcd_glass_w / 2;             // 8.4 mm
    lcd_win_x     = longan_x + lcd_win_cx_ofs - lcd_win_l/2; // 33.8 mm (X_pcb = 7.5 mm)
    lcd_win_y     = longan_cy - lcd_win_w / 2;               // 9.8 mm

    // Terminal / sense wire access centers near Rear (+Y) edge of INA219
    screw_y       = ina_y + ina_l - 3.8;                     // 48.3 mm
    screw_dx      = 2.15;

    // Thermal slot center over R100 shunt resistor & INA219 sense path
    vent_center_y = ina_y + 11.5;                            // 41.3 mm

    union() {
        difference() {
            union() {
                // 1. Solid low-profile top plate (0..case_w, 0..case_l, 0..lid_h = 2.4 mm)
                rounded_box(case_w, case_l, lid_h, corner_r);

                // 2. Full 360-Degree Interlocking Perimeter Skirt & 4-Wall Snap Detents
                lid_snap_skirt();
            }

            // 3. LCD Viewing Window Cutout (27.2 mm inner / 28.0 mm beveled x 11.4 mm inner)
            translate([lcd_win_x, lcd_win_y, -1.0])
                cube([lcd_win_l, lcd_win_w, lid_h + 2.0]);

            //    Top exterior viewing bevel around LCD window (28.0 mm x 12.6 mm outer opening,
            //    preserving a solid bridge on the +X side next to the buttons)
            hull() {
                translate([lcd_win_x, lcd_win_y, lid_h - 0.5])
                    cube([lcd_win_l, lcd_win_w, 0.1]);
                translate([lcd_win_x - 0.5, lcd_win_y - 0.6, lid_h - 0.05])
                    cube([lcd_win_l + 0.8, lcd_win_w + 1.2, 0.1]);
            }

            // 4. Unified Underside Recessed Pocket for 13.5 x 28.0 mm LCD Module (28.9 x 15.2 x 1.4 mm)
            translate([lcd_glass_x - 0.6, lcd_glass_y - 0.5, -0.5])
                cube([lcd_glass_l + 0.9, lcd_glass_w + 1.0, lcd_pocket_d + 0.5]);

            // 5. CRITICAL: Relief Pocket on Left (-X) Edge for Folded 8-Pin FPC Ribbon (1.4 mm deep)
            translate([longan_x + 3.0, longan_cy - fpc_relief_w/2, -0.5])
                cube([fpc_relief_l, fpc_relief_w, fpc_relief_d + 0.5]);

            // 6. Tactile Button Access Holes: RESET & BOOT0 (2x Ø 4.0 mm at X_pcb = 37.50 mm, Y = 6.45/13.30 mm)
            for (by = [longan_y + btn_reset_y, longan_y + btn_boot_y]) {
                translate([longan_x + btn_x_ofs, by, -1.0])
                    cylinder(d = btn_hole_d, h = lid_h + 2.0);
                // Top ergonomic finger chamfer
                translate([longan_x + btn_x_ofs, by, lid_h - 0.5])
                    cylinder(d1 = btn_hole_d, d2 = btn_hole_d + 0.9, h = 0.55);
            }

            // 7. Dual Ø 4.0 mm Access Holes over INA219 VIN-/VIN+ Pads / Terminal Screws
            translate([ina_cx - screw_dx, screw_y, -1.0])
                cylinder(d = term_screw_d, h = lid_h + 2.0);
            translate([ina_cx + screw_dx, screw_y, -1.0])
                cylinder(d = term_screw_d, h = lid_h + 2.0);

            // 8. 4x Thermal Ventilation Slots over R100 Shunt Resistor (1W @ 3.2A)
            for (i = [0 : vent_count - 1]) {
                slot_x = ina_cx - ((vent_count - 1) * vent_pitch)/2 + i * vent_pitch - vent_slot_t/2;
                translate([slot_x, vent_center_y - 4.0, -1.0])
                    rounded_slot_z(vent_slot_t, 8.0, lid_h + 2.0);
            }
        }
    }
}

// Full 360-Degree Interlocking Skirt + 4-Wall Snap Detents on Lid
// Eliminates sliding in BOTH X and Y axes by engaging all 4 walls of the Base counterbore step.
module lid_snap_skirt() {
    step_wall   = 1.2;
    inset       = step_wall + lip_clearance; // 1.45 mm inset from outer perimeter
    skirt_w     = case_w - 2 * inset;
    skirt_l     = case_l - 2 * inset;
    skirt_r     = max(0.3, corner_r - inset);
    skirt_tot_h = lip_h + 0.5;               // Extends from Z = +0.5 mm (inside lid plate) down to -lip_h

    difference() {
        // 1. Continuous 360-degree outer interlocking skirt ring
        translate([inset, inset, -lip_h])
            difference() {
                rounded_box(skirt_w, skirt_l, skirt_tot_h, skirt_r);
                translate([lip_thick, lip_thick, -0.5])
                    rounded_box(
                        skirt_w - 2*lip_thick,
                        skirt_l - 2*lip_thick,
                        skirt_tot_h + 1.0,
                        max(0.2, skirt_r - lip_thick)
                    );
            }

        // 2. Clear skirt at Right (+X) USB-C port & pry notch (only below Z = 0)
        translate([case_w - wall - 1.0, longan_cy - usbc_cut_w/2 - 1.0, -lip_h - 0.5])
            cube([wall + 2.0, usbc_cut_w + 2.0, lip_h + 0.5]);
        translate([case_w - wall - 1.0, case_l - 13.0, -lip_h - 0.5])
            cube([wall + 2.0, 10.0, lip_h + 0.5]);

        // 3. Clear skirt at Left (-X) pry notch (only below Z = 0)
        translate([0, case_l/2 - 5.0, -lip_h - 0.5])
            cube([wall + 2.0, 10.0, lip_h + 0.5]);

        // 4. Clear skirt at Rear (+Y) INA219 wire window (only below Z = 0)
        translate([ina_cx - term_cut_w/2 - 1.0, case_l - wall - 1.0, -lip_h - 0.5])
            cube([term_cut_w + 2.0, wall + 2.0, lip_h + 0.5]);
    }

    // 5. Outward Snap-Fit Detent Beads on ALL 4 Walls (Front -Y, Rear +Y, Left -X, Right +X)
    //    Front (-Y) snap bead
    translate([longan_x + longan_l/2 - snap_len/2, inset, -lip_h])
        translate([0, -snap_undercut, 0.35])
            hull() {
                cube([snap_len, snap_undercut + 0.1, 0.45]);
                translate([0, snap_undercut, -0.35])
                    cube([snap_len, 0.1, 1.1]);
            }

    //    Rear (+Y) snap bead (X = 42..54 mm)
    translate([42.0, case_l - inset - lip_thick, -lip_h])
        translate([0, lip_thick - 0.1, 0.35])
            hull() {
                cube([snap_len, snap_undercut + 0.1, 0.45]);
                translate([0, 0, -0.35])
                    cube([snap_len, 0.1, 1.1]);
            }

    //    Left (-X) snap bead (Y = 21..33 mm)
    translate([inset, 21.0, -lip_h])
        translate([-snap_undercut, 0, 0.35])
            hull() {
                cube([snap_undercut + 0.1, 12.0, 0.45]);
                translate([snap_undercut, 0, -0.35])
                    cube([0.1, 12.0, 1.1]);
            }

    //    Right (+X) snap bead (Y = 31..43 mm)
    translate([case_w - inset - lip_thick, 31.0, -lip_h])
        translate([lip_thick - 0.1, 0, 0.35])
            hull() {
                cube([snap_undercut + 0.1, 12.0, 0.45]);
                translate([0, 0, -0.35])
                    cube([0.1, 12.0, 1.1]);
            }
}

// ============================================================================
// Ghost Hardware Visualization Module (for Fit & Alignment Verification)
// ============================================================================
module ghost_hardware() {
    // 1. Sipeed Longan Nano PCB
    translate([longan_x, longan_y, longan_pcb_z]) {
        color([0.85, 0.15, 0.15])
            cube([longan_l, longan_w, longan_t]);
        // 0.96" LCD Module (13.5 mm breadth x 28.0 mm length at X_pcb = 7.4..35.4 mm)
        color([0.1, 0.1, 0.15])
            translate([lcd_glass_x_ofs, (longan_w - 13.5)/2, longan_t])
                cube([lcd_glass_l, 13.5, 2.8]);
        // 8-pin FPC Ribbon (folded over left edge at X_pcb = 3.6..7.4 mm)
        color([0.85, 0.55, 0.1])
            translate([3.6, (longan_w - 5.5)/2, longan_t])
                cube([3.8, 5.5, 2.5]);
        // 4-pin JTAG Male Header on Left Edge (protrudes 8.5 mm to -X)
        color([0.75, 0.75, 0.2])
            translate([-8.5, (longan_w - 10.0)/2, longan_t])
                cube([8.5, 10.0, 2.5]);
        // USB-C Receptacle (protrudes 3.0 mm past right edge)
        color([0.75, 0.75, 0.78])
            translate([longan_l - 4.5, (longan_w - 8.94)/2, longan_t])
                cube([4.5 + usbc_protrude, 8.94, 3.16]);
        // BOOT0 & RESET Buttons (at X_pcb = 37.50 mm, Y_pcb = 6.45 / 13.30 mm)
        color([0.95, 0.95, 0.95]) {
            translate([btn_x_ofs, btn_reset_y, longan_t]) cylinder(d = 2.4, h = 2.5);
            translate([btn_x_ofs, btn_boot_y, longan_t])  cylinder(d = 2.4, h = 2.5);
        }
        // Bottom MicroSD Socket
        color([0.7, 0.7, 0.72])
            translate([27.5, (longan_w - 11.5)/2, -1.8])
                cube([11.5, 11.5, 1.8]);
    }

    // 2. Blue INA219 Breakout PCB (25.5 mm X width x 22.3 mm Y length)
    translate([ina_x, ina_y, ina_pcb_z]) {
        color([0.1, 0.35, 0.65])
            difference() {
                cube([ina_w, ina_l, ina_t]);
                for (dx = [-ina_hole_dx/2, ina_hole_dx/2])
                    for (dy = [-ina_hole_dy/2, ina_hole_dy/2])
                        translate([ina_w/2 + dx, ina_l/2 + dy, -0.5])
                            cylinder(d = 2.6, h = ina_t + 1.0);
            }
        // R100 Shunt Resistor
        color([0.15, 0.15, 0.15])
            translate([(ina_w - 6.3)/2, 11.5 - 1.6, ina_t])
                cube([6.3, 3.2, 1.2]);
    }
}
