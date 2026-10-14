// ==========================================
// 3D Printable Case for Longan Nano + INA219
// ==========================================
$fn = 60; // Smooth curves

// --- Board & Component Dimensions (True spec) ---
longan_l = 46.1;
longan_w = 20.0;
longan_h = 3.5;

ina_l = 26.0;
ina_w = 22.0;
ina_h = 2.0;

// LCD viewport & delicate ribbon bracket
lcd_view_w = 24.5;
lcd_view_l = 14.0;
lcd_pocket_depth = 1.8;

// Terminals / Connections
term_w = 10.0;
term_l = 8.0;
term_h = 10.0;

// --- Enclosure Dimensions ---
wall = 2.0;
clearance = 0.8; // Tolerance gap
case_w = 75.0;   // X-axis
case_l = 55.0;   // Y-axis
case_h = 24.0;   // Z-axis

lid_h = 10.0;
base_h = case_h - lid_h;

// Mode identifier constants (allows both quoted string and unquoted CLI token matching)
base        = "base";
lid         = "lid";
lid_print   = "lid_print";
print       = "print";
assembled   = "assembled";
exploded    = "exploded";

// Assembly Mode:
// "exploded"   : Base & lid separated vertically
// "assembled"  : Base & lid closed together
// "base"       : Base only (oriented for printing)
// "lid"        : Lid only (as modeled)
// "lid_print"  : Lid flipped 180 deg (flat on bed for support-free printing)
// "print"      : Both base and flipped lid side-by-side on build plate
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
} else { // "exploded"
    base();
    translate([0, 0, 22]) lid();
}

// Rounded outer shell helper (hull of corner cylinders)
// Fast, robust, and correctly bounded from [0,0,0] to [w, l, h]
module rounded_box(w, l, h, r = 2) {
    hull() {
        translate([r, r, 0]) cylinder(r = r, h = h);
        translate([w - r, r, 0]) cylinder(r = r, h = h);
        translate([r, l - r, 0]) cylinder(r = r, h = h);
        translate([w - r, l - r, 0]) cylinder(r = r, h = h);
    }
}

// ==========================================
// Base (Bottom Case)
// ==========================================
module base() {
    difference() {
        // Outer box (precisely 0..case_w, 0..case_l, 0..base_h with r=2 rounded corners)
        rounded_box(case_w, case_l, base_h, 2);

        // Inner cavity (uniform 2mm wall on all sides)
        translate([wall, wall, wall])
            cube([case_w - wall*2, case_l - wall*2, base_h + 1]);

        // USB-C port cutout (Left side)
        translate([-1, 9, wall + 1.5])
            cube([wall + 3, 11, 4.5]);
            
        // microSD Slot (Left side)
        translate([-1, 26, wall + 1])
            cube([wall + 3, 13, 3]);

        // External Power terminal ports (Front-center-right)
        translate([case_w - term_w - 12, -1, wall])
            cube([term_w, wall + 3, term_h]);
    }

    // --- PCB Alignment Pegs / Pillars ---
    // Longan Nano alignment posts (Interior Left)
    translate([wall + 3, wall + 3, wall]) {
        alignment_post(0, 0, 3);
        alignment_post(longan_l - 4, 0, 3);
        alignment_post(0, longan_w - 4, 3);
        alignment_post(longan_l - 4, longan_w - 4, 3);
    }
    
    // INA219 mounting posts (Interior Right)
    translate([case_w - ina_l - wall - 3, wall + 3, wall]) {
        alignment_post(0, 0, 4);
        alignment_post(ina_l - 3, 0, 4);
        alignment_post(0, ina_w - 3, 4);
        alignment_post(ina_l - 3, ina_w - 3, 4);
    }
}

// ==========================================
// Top Lid (Upper Cover)
// ==========================================
module lid() {
    difference() {
        // Outer lid shell
        rounded_box(case_w, case_l, lid_h, 2);

        // Shell out the inside
        translate([wall, wall, -1])
            cube([case_w - wall*2, case_l - wall*2, lid_h - wall + 1]);

        // Viewport window for LCD (Top Face)
        translate([wall + 4, wall + 8, lid_h - wall - 1])
            cube([lcd_view_w, lcd_view_l, wall + 3]);
            
        // Button Holes: Reset
        translate([wall + 32, wall + 7, lid_h - wall - 1])
            cylinder(d=4.5, h=wall+3);
            
        // Button Holes: Boot
        translate([wall + 32, wall + 17, lid_h - wall - 1])
            cylinder(d=4.5, h=wall+3);
            
        // Ventilation Slots (Back side)
        for (i = [0 : 3]) {
            translate([case_w - 20 - (i*6), case_l - wall - 1, lid_h/2 - 1.5])
                cube([3.5, wall + 3, 5]);
        }
    }

    // --- LCD Display Support Shelf & Ribbon Relief ---
    // Protects fragile flex FPC connector by clamping natural board stack
    translate([wall + 2.5, wall + 6.5, lid_h - wall - lcd_pocket_depth]) {
        difference() {
            // Inner supporting brim for the display backing (retaining glass bezel)
            cube([lcd_view_w + 3, lcd_view_l + 3, lcd_pocket_depth]);
            // Hollowed center window (light clearance)
            translate([1.5, 1.5, -1])
                cube([lcd_view_w, lcd_view_l, lcd_pocket_depth + 2]);
            // Tiny gap on the side for the folded ribbon cable passage
            translate([-1, 4, -1])
                cube([4, 6, lcd_pocket_depth + 2]);
        }
    }
}

// Pin support helper
module alignment_post(x, y, z) {
    translate([x, y, 0]) {
        cylinder(d=2.8, h=z); // Base post
        cylinder(d=1.4, h=z+2); // Tight mounting nipple
    }
}
