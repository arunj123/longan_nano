use egui::Color32;
use crate::graphics3d::math3d::Vec3;
use crate::graphics3d::mesh::{Mesh3D, Vertex};

pub fn build_longan_nano_board(
    _v_mv: u16,
    c_ma: f64,
    _p_mw: u16,
    alert: bool,
    connected: bool,
    anim_time: f64,
) -> Mesh3D {
    let mut mesh = Mesh3D::new();

    // ==========================================
    // LAYER 0: Studio Environment & Shadow
    // ==========================================
    // Shadow disc under the board
    let shadow_col = Color32::from_rgba_unmultiplied(5, 8, 14, 180);
    mesh.add_box_layered(Vec3::new(4.0, -4.5, 12.0), Vec3::new(66.0, 0.1, 48.0), shadow_col, 0);

    // Floor grid
    let grid_y = -4.6;
    let grid_col = Color32::from_rgba_unmultiplied(30, 45, 65, 70);
    let mut x = -60.0;
    while x <= 70.0 {
        mesh.add_line(Vec3::new(x, grid_y, -25.0), Vec3::new(x, grid_y, 45.0), grid_col);
        x += 10.0;
    }
    let mut z = -25.0;
    while z <= 45.0 {
        mesh.add_line(Vec3::new(-60.0, grid_y, z), Vec3::new(70.0, grid_y, z), grid_col);
        z += 10.0;
    }

    // ==========================================
    // LAYER 1: PCB Boards
    // ==========================================
    // Longan Nano Board: 48mm x 18mm x 1.6mm
    let nano_pcb_col = Color32::from_rgb(14, 52, 28); // Solder mask green
    mesh.add_box_layered(Vec3::new(0.0, 0.0, 0.0), Vec3::new(50.0, 1.6, 20.0), nano_pcb_col, 1);

    // Gold PCB Edge Bevels
    let gold_bevel = Color32::from_rgb(215, 175, 55);
    mesh.add_box_layered(Vec3::new(0.0, 0.0, -10.05), Vec3::new(50.0, 0.4, 0.1), gold_bevel, 1);
    mesh.add_box_layered(Vec3::new(0.0, 0.0, 10.05), Vec3::new(50.0, 0.4, 0.1), gold_bevel, 1);

    // INA219 Breakout Board: 22mm x 20mm x 1.6mm (Royal Purple/Blue PCB)
    let ina_pos = Vec3::new(6.0, -0.4, 25.0);
    let ina_pcb_col = Color32::from_rgb(45, 20, 85);
    mesh.add_box_layered(ina_pos, Vec3::new(22.0, 1.6, 20.0), ina_pcb_col, 1);

    // ==========================================
    // LAYER 2: Components & Connectors
    // ==========================================
    // Gold Header Pins (Top and Bottom edge of Longan Nano)
    let gold_col = Color32::from_rgb(230, 190, 60);
    let pin_base_black = Color32::from_rgb(25, 25, 28);
    mesh.add_box_layered(Vec3::new(0.0, -1.2, -8.6), Vec3::new(43.0, 1.2, 2.5), pin_base_black, 2);
    mesh.add_box_layered(Vec3::new(0.0, -1.2, 8.6), Vec3::new(43.0, 1.2, 2.5), pin_base_black, 2);

    for i in 0..16 {
        let x = -20.0 + (i as f32 * 2.66);
        mesh.add_box_layered(Vec3::new(x, -2.6, -8.6), Vec3::new(0.65, 2.2, 0.65), gold_col, 2);
        mesh.add_box_layered(Vec3::new(x, -2.6, 8.6), Vec3::new(0.65, 2.2, 0.65), gold_col, 2);
    }

    // USB-C Connector (Metal Shell on Left side: X = -24.0)
    let metal_col = Color32::from_rgb(205, 210, 220);
    let metal_dark = Color32::from_rgb(40, 45, 50);
    mesh.add_box_layered(Vec3::new(-23.8, 1.8, 0.0), Vec3::new(7.2, 3.0, 8.8), metal_col, 2);
    // Dark USB-C inner receptacle cavity
    mesh.add_box_layered(Vec3::new(-27.2, 1.8, 0.0), Vec3::new(0.5, 1.4, 6.4), metal_dark, 2);

    // GD32VF103CBT6 MCU (LQFP48 matte black package)
    let mcu_col = Color32::from_rgb(22, 24, 28);
    mesh.add_box_layered(Vec3::new(-10.0, 1.3, 0.0), Vec3::new(7.2, 0.9, 7.2), mcu_col, 2);
    // Metallic lead edges
    let lead_col = Color32::from_rgb(180, 185, 195);
    mesh.add_box_layered(Vec3::new(-10.0, 0.9, -3.9), Vec3::new(6.0, 0.3, 0.6), lead_col, 2);
    mesh.add_box_layered(Vec3::new(-10.0, 0.9, 3.9), Vec3::new(6.0, 0.3, 0.6), lead_col, 2);
    mesh.add_box_layered(Vec3::new(-13.9, 0.9, 0.0), Vec3::new(0.6, 0.3, 6.0), lead_col, 2);
    mesh.add_box_layered(Vec3::new(-6.1, 0.9, 0.0), Vec3::new(0.6, 0.3, 6.0), lead_col, 2);
    // MCU Pin 1 dot
    mesh.add_box_layered(Vec3::new(-12.4, 1.8, -2.4), Vec3::new(0.7, 0.2, 0.7), Color32::from_rgb(220, 220, 230), 2);

    // Reset and Boot0 tactile buttons
    let btn_col = Color32::from_rgb(170, 175, 185);
    mesh.add_box_layered(Vec3::new(-16.5, 1.4, -6.0), Vec3::new(3.0, 1.2, 2.6), btn_col, 2);
    mesh.add_box_layered(Vec3::new(-16.5, 1.4, 6.0), Vec3::new(3.0, 1.2, 2.6), btn_col, 2);

    // Crystal Oscillator (Metal can)
    mesh.add_box_layered(Vec3::new(-2.5, 1.3, 6.2), Vec3::new(4.5, 1.1, 2.5), metal_col, 2);

    // INA219 Terminal Block (Emerald Green block + 2 Silver Screws)
    let term_col = Color32::from_rgb(18, 125, 55);
    mesh.add_box_layered(ina_pos + Vec3::new(-7.0, 4.2, 0.0), Vec3::new(6.5, 7.2, 12.5), term_col, 2);
    mesh.add_box_layered(ina_pos + Vec3::new(-7.0, 7.9, -3.2), Vec3::new(2.6, 0.6, 2.6), metal_col, 2);
    mesh.add_box_layered(ina_pos + Vec3::new(-7.0, 7.9, 3.2), Vec3::new(2.6, 0.6, 2.6), metal_col, 2);
    // Wire entry sockets
    mesh.add_box_layered(ina_pos + Vec3::new(-10.2, 3.6, -3.2), Vec3::new(0.4, 2.2, 2.2), metal_dark, 2);
    mesh.add_box_layered(ina_pos + Vec3::new(-10.2, 3.6, 3.2), Vec3::new(0.4, 2.2, 2.2), metal_dark, 2);

    // High-Precision 2512 Ceramic Shunt Resistor (R100)
    let shunt_col = Color32::from_rgb(25, 45, 35);
    mesh.add_box_layered(ina_pos + Vec3::new(0.0, 1.3, 0.0), Vec3::new(6.8, 1.3, 3.6), shunt_col, 2);
    mesh.add_box_layered(ina_pos + Vec3::new(-3.1, 1.3, 0.0), Vec3::new(0.8, 1.35, 3.6), lead_col, 2);
    mesh.add_box_layered(ina_pos + Vec3::new(3.1, 1.3, 0.0), Vec3::new(0.8, 1.35, 3.6), lead_col, 2);

    // INA219 IC (SOT-23-8)
    mesh.add_box_layered(ina_pos + Vec3::new(6.2, 1.1, 0.0), Vec3::new(3.2, 0.8, 3.0), mcu_col, 2);

    // ==========================================
    // LAYER 3: ST7735 LCD Substrate & Bezel
    // ==========================================
    let lcd_pos = Vec3::new(12.5, 1.5, 0.0);
    // LCD PCB Backing
    mesh.add_box_layered(lcd_pos, Vec3::new(27.0, 1.2, 15.0), Color32::from_rgb(18, 22, 28), 3);
    // Outer Black Bezel Frame
    mesh.add_box_layered(lcd_pos + Vec3::new(0.0, 0.8, 0.0), Vec3::new(25.0, 0.6, 13.6), Color32::from_rgb(10, 12, 16), 3);

    // ==========================================
    // LAYER 4: ST7735 LCD Screen Display Face
    // ==========================================
    // Glass surface
    let scr_top_y = lcd_pos.y + 1.15;
    let scr_w = 23.4;
    let scr_h = 12.2;
    let scr_c = Vec3::new(lcd_pos.x, scr_top_y, lcd_pos.z);

    // LCD Dark Glass Base Surface
    let lcd_bg = Color32::from_rgb(11, 16, 25);
    let hw = scr_w * 0.5;
    let hh = scr_h * 0.5;
    mesh.add_quad_layered(
        Vertex::new(scr_c + Vec3::new(-hw, 0.0, -hh), lcd_bg, Vec3::UP),
        Vertex::new(scr_c + Vec3::new(hw, 0.0, -hh), lcd_bg, Vec3::UP),
        Vertex::new(scr_c + Vec3::new(hw, 0.0, hh), lcd_bg, Vec3::UP),
        Vertex::new(scr_c + Vec3::new(-hw, 0.0, hh), lcd_bg, Vec3::UP),
        true,
        4,
    );

    // 1. LCD Header Bar (Mint Banner)
    let header_col = Color32::from_rgb(0, 200, 140);
    mesh.add_quad_layered(
        Vertex::new(scr_c + Vec3::new(-hw + 0.8, 0.01, -hh + 0.6), header_col, Vec3::UP),
        Vertex::new(scr_c + Vec3::new(-hw + 8.5, 0.01, -hh + 0.6), header_col, Vec3::UP),
        Vertex::new(scr_c + Vec3::new(-hw + 8.5, 0.01, -hh + 2.0), header_col, Vec3::UP),
        Vertex::new(scr_c + Vec3::new(-hw + 0.8, 0.01, -hh + 2.0), header_col, Vec3::UP),
        true,
        4,
    );

    // USB / Clock indicator pill on LCD header
    let usb_pill_col = if connected { Color32::from_rgb(0, 240, 255) } else { Color32::from_rgb(45, 60, 80) };
    mesh.add_quad_layered(
        Vertex::new(scr_c + Vec3::new(hw - 6.5, 0.01, -hh + 0.6), usb_pill_col, Vec3::UP),
        Vertex::new(scr_c + Vec3::new(hw - 0.8, 0.01, -hh + 0.6), usb_pill_col, Vec3::UP),
        Vertex::new(scr_c + Vec3::new(hw - 0.8, 0.01, -hh + 2.0), usb_pill_col, Vec3::UP),
        Vertex::new(scr_c + Vec3::new(hw - 6.5, 0.01, -hh + 2.0), usb_pill_col, Vec3::UP),
        true,
        4,
    );

    // 2. Large Live Current Card (Hero Metric)
    let cur_glow = if alert {
        Color32::from_rgb(255, 50, 50)
    } else if c_ma.abs() > 500.0 {
        Color32::from_rgb(245, 160, 40)
    } else {
        Color32::from_rgb(60, 240, 255)
    };
    mesh.add_quad_layered(
        Vertex::new(scr_c + Vec3::new(-hw + 1.2, 0.02, -hh + 2.5), cur_glow, Vec3::UP),
        Vertex::new(scr_c + Vec3::new(hw - 1.2, 0.02, -hh + 2.5), cur_glow, Vec3::UP),
        Vertex::new(scr_c + Vec3::new(hw - 1.2, 0.02, -hh + 5.8), cur_glow, Vec3::UP),
        Vertex::new(scr_c + Vec3::new(-hw + 1.2, 0.02, -hh + 5.8), cur_glow, Vec3::UP),
        true,
        4,
    );

    // 3. Voltage (Mint) & Power (Amber) Telemetry Sub-Cards
    let volt_col = Color32::from_rgb(46, 180, 80);
    let pwr_col = Color32::from_rgb(240, 185, 45);
    mesh.add_quad_layered(
        Vertex::new(scr_c + Vec3::new(-hw + 1.2, 0.02, -hh + 6.3), volt_col, Vec3::UP),
        Vertex::new(scr_c + Vec3::new(-0.4, 0.02, -hh + 6.3), volt_col, Vec3::UP),
        Vertex::new(scr_c + Vec3::new(-0.4, 0.02, -hh + 8.4), volt_col, Vec3::UP),
        Vertex::new(scr_c + Vec3::new(-hw + 1.2, 0.02, -hh + 8.4), volt_col, Vec3::UP),
        true,
        4,
    );
    mesh.add_quad_layered(
        Vertex::new(scr_c + Vec3::new(0.4, 0.02, -hh + 6.3), pwr_col, Vec3::UP),
        Vertex::new(scr_c + Vec3::new(hw - 1.2, 0.02, -hh + 6.3), pwr_col, Vec3::UP),
        Vertex::new(scr_c + Vec3::new(hw - 1.2, 0.02, -hh + 8.4), pwr_col, Vec3::UP),
        Vertex::new(scr_c + Vec3::new(0.4, 0.02, -hh + 8.4), pwr_col, Vec3::UP),
        true,
        4,
    );

    // 4. 25-Segment Load Bar Meter on bottom of LCD
    let num_segs = 24;
    let seg_total_w = scr_w - 2.4;
    let seg_w = seg_total_w / num_segs as f32;
    let active_ratio = (c_ma.abs() as f32 / 1000.0).clamp(0.0, 1.0);
    let active_count = (active_ratio * num_segs as f32).round() as usize;

    for i in 0..num_segs {
        let seg_x0 = -hw + 1.2 + (i as f32 * seg_w);
        let seg_x1 = seg_x0 + (seg_w - 0.25);
        let seg_z0 = hh - 2.8;
        let seg_z1 = hh - 0.8;

        let col = if i < active_count {
            if i > 18 {
                Color32::from_rgb(255, 50, 50)
            } else if i > 12 {
                Color32::from_rgb(240, 180, 40)
            } else {
                Color32::from_rgb(40, 220, 120)
            }
        } else {
            Color32::from_rgb(20, 28, 40) // Unlit dark segment
        };

        mesh.add_quad_layered(
            Vertex::new(scr_c + Vec3::new(seg_x0, 0.02, seg_z0), col, Vec3::UP),
            Vertex::new(scr_c + Vec3::new(seg_x1, 0.02, seg_z0), col, Vec3::UP),
            Vertex::new(scr_c + Vec3::new(seg_x1, 0.02, seg_z1), col, Vec3::UP),
            Vertex::new(scr_c + Vec3::new(seg_x0, 0.02, seg_z1), col, Vec3::UP),
            true,
            4,
        );
    }

    // ==========================================
    // LAYER 5: Dynamic Emissive Status LEDs
    // ==========================================
    // 1. Green Heartbeat LED (PA1)
    let hb_cycle = (anim_time * 2.5).sin().powi(4) as f32;
    let green_led = if hb_cycle > 0.25 {
        Color32::from_rgb(50, 255, 90)
    } else {
        Color32::from_rgb(18, 65, 28)
    };
    mesh.add_box_layered(Vec3::new(-4.0, 1.2, -6.0), Vec3::new(1.5, 0.7, 1.0), green_led, 5);

    // 2. Red Alert LED (PC13)
    let red_flash = (anim_time * 9.0).sin() > 0.0;
    let red_led = if alert && red_flash {
        Color32::from_rgb(255, 30, 30)
    } else if alert {
        Color32::from_rgb(110, 15, 15)
    } else {
        Color32::from_rgb(50, 12, 12)
    };
    mesh.add_box_layered(Vec3::new(-4.0, 1.2, -3.5), Vec3::new(1.5, 0.7, 1.0), red_led, 5);

    // 3. Blue/Cyan USB LED (PA2)
    let usb_led = if connected {
        Color32::from_rgb(0, 235, 255)
    } else {
        Color32::from_rgb(12, 45, 65)
    };
    mesh.add_box_layered(Vec3::new(-4.0, 1.2, -1.0), Vec3::new(1.5, 0.7, 1.0), usb_led, 5);

    // ==========================================
    // LAYER 6: Arched Flexible 3D Jumper Wires
    // ==========================================
    // 4 curved jumper wires with 3 segments each for smooth curve
    let add_curved_wire = |mesh: &mut Mesh3D, start: Vec3, end: Vec3, color: Color32| {
        let mid1 = Vec3::new(
            start.x * 0.65 + end.x * 0.35,
            (start.y + end.y) * 0.5 + 4.5,
            start.z * 0.65 + end.z * 0.35,
        );
        let mid2 = Vec3::new(
            start.x * 0.35 + end.x * 0.65,
            (start.y + end.y) * 0.5 + 4.5,
            start.z * 0.35 + end.z * 0.65,
        );
        mesh.add_line(start, mid1, color);
        mesh.add_line(mid1, mid2, color);
        mesh.add_line(mid2, end, color);
    };

    let p0 = Vec3::new(0.0, 0.2, 8.6);
    let p1 = Vec3::new(2.7, 0.2, 8.6);
    let p2 = Vec3::new(5.4, 0.2, 8.6);
    let p3 = Vec3::new(8.1, 0.2, 8.6);

    let t0 = ina_pos + Vec3::new(-4.5, 0.2, -9.0);
    let t1 = ina_pos + Vec3::new(-1.8, 0.2, -9.0);
    let t2 = ina_pos + Vec3::new(0.9, 0.2, -9.0);
    let t3 = ina_pos + Vec3::new(3.6, 0.2, -9.0);

    add_curved_wire(&mut mesh, p0, t0, Color32::from_rgb(230, 45, 45));  // 3.3V (Red)
    add_curved_wire(&mut mesh, p1, t1, Color32::from_rgb(35, 35, 38));   // GND (Black)
    add_curved_wire(&mut mesh, p2, t2, Color32::from_rgb(45, 130, 245)); // SCL (Blue)
    add_curved_wire(&mut mesh, p3, t3, Color32::from_rgb(245, 200, 35)); // SDA (Yellow)

    mesh
}

pub fn build_phase_trajectory_ribbon(points: &[(f64, f64, f64)]) -> Mesh3D {
    let mut mesh = Mesh3D::new();

    if points.len() < 2 {
        return mesh;
    }

    // Grid floor at Y = -15.0
    let grid_size = 55.0;
    let grid_y = -15.0;
    let grid_col = Color32::from_rgba_unmultiplied(35, 55, 80, 75);
    let step = 10.0;
    let mut x = -grid_size;
    while x <= grid_size {
        mesh.add_line(Vec3::new(x, grid_y, -grid_size), Vec3::new(x, grid_y, grid_size), grid_col);
        mesh.add_line(Vec3::new(-grid_size, grid_y, x), Vec3::new(grid_size, grid_y, x), grid_col);
        x += step;
    }

    // Coordinate Axes with Axis Labels
    mesh.add_line(Vec3::new(0.0, grid_y, 0.0), Vec3::new(45.0, grid_y, 0.0), Color32::from_rgb(240, 70, 70)); // X Red (Time)
    mesh.add_line(Vec3::new(0.0, grid_y, 0.0), Vec3::new(0.0, 32.0, 0.0), Color32::from_rgb(50, 240, 100));   // Y Green (Current)
    mesh.add_line(Vec3::new(0.0, grid_y, 0.0), Vec3::new(0.0, grid_y, 45.0), Color32::from_rgb(60, 150, 255)); // Z Blue (Voltage)

    let n = points.len();
    let ribbon_half_w = 0.9;

    for i in 0..(n - 1) {
        let (_t0, v0, c0) = points[i];
        let (_t1, v1, c1) = points[i + 1];

        // Spatial Mapping
        let x0 = (i as f32 - n as f32 * 0.5) * 0.75;
        let x1 = ((i + 1) as f32 - n as f32 * 0.5) * 0.75;

        let y0 = (c0 as f32 * 0.16 - 12.0).clamp(-14.5, 42.0);
        let y1 = (c1 as f32 * 0.16 - 12.0).clamp(-14.5, 42.0);

        let z0 = ((v0 as f32 - 3.3) * 16.0).clamp(-22.0, 22.0);
        let z1 = ((v1 as f32 - 3.3) * 16.0).clamp(-22.0, 22.0);

        let p0 = Vec3::new(x0, y0, z0);
        let p1 = Vec3::new(x1, y1, z1);

        // Dynamic vertex color based on instantaneous load severity
        let col = if c1 > 500.0 {
            Color32::from_rgb(255, 45, 45) // Critical Red
        } else if c1 > 150.0 {
            Color32::from_rgb(245, 155, 35) // High Amber
        } else if c1 > 30.0 {
            Color32::from_rgb(35, 225, 130) // Medium Mint
        } else {
            Color32::from_rgb(0, 210, 255)  // Quiescent Cyan
        };

        // 3D Neon Ribbon quad
        let offset = Vec3::new(0.0, 0.0, ribbon_half_w);
        mesh.add_quad_layered(
            Vertex::new(p0 - offset, col, Vec3::UP),
            Vertex::new(p0 + offset, col, Vec3::UP),
            Vertex::new(p1 + offset, col, Vec3::UP),
            Vertex::new(p1 - offset, col, Vec3::UP),
            true, // Emissive
            2,
        );

        // Floor shadow projection ribbon for depth perception
        let shadow_col = Color32::from_rgba_unmultiplied(20, 35, 55, 90);
        let s0 = Vec3::new(x0, grid_y + 0.1, z0);
        let s1 = Vec3::new(x1, grid_y + 0.1, z1);
        mesh.add_quad_layered(
            Vertex::new(s0 - offset, shadow_col, Vec3::UP),
            Vertex::new(s0 + offset, shadow_col, Vec3::UP),
            Vertex::new(s1 + offset, shadow_col, Vec3::UP),
            Vertex::new(s1 - offset, shadow_col, Vec3::UP),
            false,
            1,
        );
    }

    mesh
}
