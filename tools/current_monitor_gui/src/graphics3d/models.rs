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

    // Board Dimensions in 3D scene units
    // Length: 50.0, Width: 20.0, Thickness: 1.6
    let pcb_col = Color32::from_rgb(18, 55, 32); // Dark matte green FR-4
    mesh.add_box(Vec3::new(0.0, 0.0, 0.0), Vec3::new(50.0, 1.6, 20.0), pcb_col);

    // Gold Header Pins (Top and Bottom edge)
    let gold_col = Color32::from_rgb(220, 180, 50);
    let pin_black = Color32::from_rgb(30, 30, 30);
    // Pin plastic bases
    mesh.add_box(Vec3::new(0.0, -1.2, -8.5), Vec3::new(42.0, 1.2, 2.5), pin_black);
    mesh.add_box(Vec3::new(0.0, -1.2, 8.5), Vec3::new(42.0, 1.2, 2.5), pin_black);

    // Gold header pin tips
    for i in 0..16 {
        let x = -19.5 + (i as f32 * 2.6);
        mesh.add_box(Vec3::new(x, -2.5, -8.5), Vec3::new(0.6, 1.8, 0.6), gold_col);
        mesh.add_box(Vec3::new(x, -2.5, 8.5), Vec3::new(0.6, 1.8, 0.6), gold_col);
    }

    // USB-C Connector (Metal Shell on Left side: X = -24.0)
    let metal_col = Color32::from_rgb(190, 195, 205);
    mesh.add_box(Vec3::new(-23.5, 1.8, 0.0), Vec3::new(7.0, 2.8, 8.5), metal_col);
    // USB-C inner tongue
    mesh.add_box(Vec3::new(-26.8, 1.8, 0.0), Vec3::new(0.5, 1.0, 6.0), Color32::from_rgb(20, 20, 20));

    // GD32VF103CBT6 MCU (LQFP48 matte black package)
    let mcu_col = Color32::from_rgb(25, 25, 28);
    mesh.add_box(Vec3::new(-10.0, 1.3, 0.0), Vec3::new(7.0, 0.9, 7.0), mcu_col);
    // MCU Pin 1 dot
    mesh.add_box(Vec3::new(-12.5, 1.8, -2.5), Vec3::new(0.6, 0.2, 0.6), Color32::from_rgb(180, 180, 180));

    // Reset and Boot Buttons
    let btn_metal = Color32::from_rgb(160, 160, 170);
    mesh.add_box(Vec3::new(-16.0, 1.5, -6.0), Vec3::new(3.0, 1.4, 2.5), btn_metal);
    mesh.add_box(Vec3::new(-16.0, 1.5, 6.0), Vec3::new(3.0, 1.4, 2.5), btn_metal);

    // On-board Status LEDs (SMD 0603)
    // 1. Green Heartbeat LED (PA1)
    let hb_pulse = (anim_time * 2.5).sin().powi(4) as f32; // Smooth periodic blink
    let green_led = if hb_pulse > 0.3 {
        Color32::from_rgb(40, 255, 80)
    } else {
        Color32::from_rgb(15, 60, 25)
    };
    mesh.add_box(Vec3::new(-4.0, 1.2, -6.0), Vec3::new(1.4, 0.7, 0.9), green_led);

    // 2. Red Alert LED (PC13)
    let red_flash = (anim_time * 8.0).sin() > 0.0;
    let red_led = if alert && red_flash {
        Color32::from_rgb(255, 30, 30) // Emissive bright scarlet
    } else if alert {
        Color32::from_rgb(100, 10, 10)
    } else {
        Color32::from_rgb(50, 10, 10)
    };
    mesh.add_box(Vec3::new(-4.0, 1.2, -3.5), Vec3::new(1.4, 0.7, 0.9), red_led);

    // 3. Blue/Cyan USB LED (PA2)
    let usb_led = if connected {
        Color32::from_rgb(0, 230, 255)
    } else {
        Color32::from_rgb(10, 40, 60)
    };
    mesh.add_box(Vec3::new(-4.0, 1.2, -1.0), Vec3::new(1.4, 0.7, 0.9), usb_led);

    // ==========================================
    // ST7735 160x80 Color SPI LCD Display
    // Mounted on right side of PCB: X = 12.0
    // ==========================================
    // LCD Module Board / Backing
    mesh.add_box(Vec3::new(12.0, 1.5, 0.0), Vec3::new(26.0, 1.2, 14.5), Color32::from_rgb(15, 20, 25));
    // Glass Display Bezel
    mesh.add_box(Vec3::new(12.0, 2.2, 0.0), Vec3::new(24.0, 0.5, 13.0), Color32::from_rgb(8, 12, 16));

    // Dynamic Live LCD Screen Surface
    let scr_center = Vec3::new(12.0, 2.5, 0.0);
    // Background surface
    mesh.add_quad(
        Vertex::new(scr_center + Vec3::new(-11.0, 0.0, -5.8), Color32::from_rgb(10, 14, 22), Vec3::UP),
        Vertex::new(scr_center + Vec3::new(11.0, 0.0, -5.8), Color32::from_rgb(10, 14, 22), Vec3::UP),
        Vertex::new(scr_center + Vec3::new(11.0, 0.0, 5.8), Color32::from_rgb(10, 14, 22), Vec3::UP),
        Vertex::new(scr_center + Vec3::new(-11.0, 0.0, 5.8), Color32::from_rgb(10, 14, 22), Vec3::UP),
        true, // Emissive
    );

    // LCD Header Bar (Mint badge)
    mesh.add_quad(
        Vertex::new(scr_center + Vec3::new(-10.5, 0.02, -5.2), Color32::from_rgb(0, 180, 130), Vec3::UP),
        Vertex::new(scr_center + Vec3::new(-2.0, 0.02, -5.2), Color32::from_rgb(0, 180, 130), Vec3::UP),
        Vertex::new(scr_center + Vec3::new(-2.0, 0.02, -3.8), Color32::from_rgb(0, 180, 130), Vec3::UP),
        Vertex::new(scr_center + Vec3::new(-10.5, 0.02, -3.8), Color32::from_rgb(0, 180, 130), Vec3::UP),
        true,
    );

    // Large Live Current metric representation (Bright Cyan glow)
    let cur_glow = if alert { Color32::from_rgb(255, 60, 60) } else { Color32::from_rgb(80, 240, 255) };
    mesh.add_quad(
        Vertex::new(scr_center + Vec3::new(-9.5, 0.03, -2.5), cur_glow, Vec3::UP),
        Vertex::new(scr_center + Vec3::new(8.5, 0.03, -2.5), cur_glow, Vec3::UP),
        Vertex::new(scr_center + Vec3::new(8.5, 0.03, 1.2), cur_glow, Vec3::UP),
        Vertex::new(scr_center + Vec3::new(-9.5, 0.03, 1.2), cur_glow, Vec3::UP),
        true,
    );

    // Voltage & Power Telemetry Indicators (Mint & Amber bars)
    mesh.add_quad(
        Vertex::new(scr_center + Vec3::new(-9.5, 0.03, 2.0), Color32::from_rgb(46, 160, 67), Vec3::UP),
        Vertex::new(scr_center + Vec3::new(-1.0, 0.03, 2.0), Color32::from_rgb(46, 160, 67), Vec3::UP),
        Vertex::new(scr_center + Vec3::new(-1.0, 0.03, 3.2), Color32::from_rgb(46, 160, 67), Vec3::UP),
        Vertex::new(scr_center + Vec3::new(-9.5, 0.03, 3.2), Color32::from_rgb(46, 160, 67), Vec3::UP),
        true,
    );
    mesh.add_quad(
        Vertex::new(scr_center + Vec3::new(0.0, 0.03, 2.0), Color32::from_rgb(227, 179, 65), Vec3::UP),
        Vertex::new(scr_center + Vec3::new(8.5, 0.03, 2.0), Color32::from_rgb(227, 179, 65), Vec3::UP),
        Vertex::new(scr_center + Vec3::new(8.5, 0.03, 3.2), Color32::from_rgb(227, 179, 65), Vec3::UP),
        Vertex::new(scr_center + Vec3::new(0.0, 0.03, 3.2), Color32::from_rgb(227, 179, 65), Vec3::UP),
        true,
    );

    // 25-Segment Bar Meter (Simulated live fill based on current)
    let fill_ratio = (c_ma.abs() as f32 / 1000.0).clamp(0.05, 1.0);
    let bar_w = 18.0 * fill_ratio;
    let bar_col = if fill_ratio > 0.8 {
        Color32::from_rgb(255, 60, 60)
    } else if fill_ratio > 0.5 {
        Color32::from_rgb(240, 180, 40)
    } else {
        Color32::from_rgb(50, 220, 120)
    };
    mesh.add_quad(
        Vertex::new(scr_center + Vec3::new(-9.5, 0.03, 4.0), bar_col, Vec3::UP),
        Vertex::new(scr_center + Vec3::new(-9.5 + bar_w, 0.03, 4.0), bar_col, Vec3::UP),
        Vertex::new(scr_center + Vec3::new(-9.5 + bar_w, 0.03, 5.0), bar_col, Vec3::UP),
        Vertex::new(scr_center + Vec3::new(-9.5, 0.03, 5.0), bar_col, Vec3::UP),
        true,
    );

    // ==========================================
    // INA219 Sensor Module 3D Model (Adjacent)
    // Positioned at Z = 22.0
    // ==========================================
    let ina_pos = Vec3::new(5.0, -0.5, 24.0);
    // Purple Breakout PCB
    mesh.add_box(ina_pos, Vec3::new(22.0, 1.4, 20.0), Color32::from_rgb(60, 30, 95));
    // Screw Terminal Block (Green)
    mesh.add_box(ina_pos + Vec3::new(-7.0, 4.0, 0.0), Vec3::new(6.0, 7.0, 12.0), Color32::from_rgb(20, 120, 50));
    // Screws (Silver)
    mesh.add_box(ina_pos + Vec3::new(-7.0, 7.6, -3.0), Vec3::new(2.5, 0.6, 2.5), metal_col);
    mesh.add_box(ina_pos + Vec3::new(-7.0, 7.6, 3.0), Vec3::new(2.5, 0.6, 2.5), metal_col);
    // High-precision R100 Shunt Resistor (Large ceramic block: 2512 package)
    mesh.add_box(ina_pos + Vec3::new(0.0, 1.3, 0.0), Vec3::new(6.5, 1.2, 3.5), Color32::from_rgb(20, 40, 30));
    // INA219 SOT-23-8 IC
    mesh.add_box(ina_pos + Vec3::new(6.0, 1.1, 0.0), Vec3::new(3.0, 0.8, 3.0), Color32::from_rgb(20, 20, 20));

    // Connecting jumper wires from Longan Nano (I2C0 PB6/PB7 + 3.3V/GND)
    mesh.add_line(Vec3::new(0.0, 0.0, 8.5), ina_pos + Vec3::new(-5.0, 0.0, -9.0), Color32::from_rgb(220, 40, 40));  // 3.3V (Red)
    mesh.add_line(Vec3::new(3.0, 0.0, 8.5), ina_pos + Vec3::new(-2.0, 0.0, -9.0), Color32::from_rgb(30, 30, 30));   // GND (Black)
    mesh.add_line(Vec3::new(6.0, 0.0, 8.5), ina_pos + Vec3::new(1.0, 0.0, -9.0), Color32::from_rgb(40, 120, 240)); // SCL (Blue)
    mesh.add_line(Vec3::new(9.0, 0.0, 8.5), ina_pos + Vec3::new(4.0, 0.0, -9.0), Color32::from_rgb(240, 200, 30)); // SDA (Yellow)

    mesh
}

pub fn build_phase_trajectory_ribbon(points: &[(f64, f64, f64)]) -> Mesh3D {
    let mut mesh = Mesh3D::new();

    if points.len() < 2 {
        return mesh;
    }

    // Grid floor at Y = -15.0
    let grid_size = 60.0;
    let grid_y = -15.0;
    let grid_col = Color32::from_rgba_unmultiplied(40, 60, 80, 80);
    let step = 10.0;
    let mut x = -grid_size;
    while x <= grid_size {
        mesh.add_line(Vec3::new(x, grid_y, -grid_size), Vec3::new(x, grid_y, grid_size), grid_col);
        mesh.add_line(Vec3::new(-grid_size, grid_y, x), Vec3::new(grid_size, grid_y, x), grid_col);
        x += step;
    }

    // Coordinate Axes (X: Time, Y: Current, Z: Voltage)
    mesh.add_line(Vec3::new(0.0, grid_y, 0.0), Vec3::new(40.0, grid_y, 0.0), Color32::from_rgb(240, 60, 60)); // X Red (Time)
    mesh.add_line(Vec3::new(0.0, grid_y, 0.0), Vec3::new(0.0, 30.0, 0.0), Color32::from_rgb(60, 240, 80));    // Y Green (Current)
    mesh.add_line(Vec3::new(0.0, grid_y, 0.0), Vec3::new(0.0, grid_y, 40.0), Color32::from_rgb(60, 140, 240)); // Z Blue (Voltage)

    // Normalize and generate 3D ribbon
    let n = points.len();
    let ribbon_half_w = 0.8;

    for i in 0..(n - 1) {
        let (_t0, v0, c0) = points[i];
        let (_t1, v1, c1) = points[i + 1];

        // Mapping:
        // X = time offset from newest sample
        let x0 = (i as f32 - n as f32 * 0.5) * 0.7;
        let x1 = ((i + 1) as f32 - n as f32 * 0.5) * 0.7;

        // Y = current in mA scaled
        let y0 = (c0 as f32 * 0.15 - 10.0).clamp(-14.0, 40.0);
        let y1 = (c1 as f32 * 0.15 - 10.0).clamp(-14.0, 40.0);

        // Z = voltage in V scaled
        let z0 = ((v0 as f32 - 3.3) * 15.0).clamp(-20.0, 20.0);
        let z1 = ((v1 as f32 - 3.3) * 15.0).clamp(-20.0, 20.0);

        let p0 = Vec3::new(x0, y0, z0);
        let p1 = Vec3::new(x1, y1, z1);

        // Dynamic vertex color based on instantaneous current
        let col = if c1 > 500.0 {
            Color32::from_rgb(255, 50, 50)
        } else if c1 > 150.0 {
            Color32::from_rgb(245, 160, 40)
        } else if c1 > 30.0 {
            Color32::from_rgb(40, 220, 140)
        } else {
            Color32::from_rgb(0, 180, 255)
        };

        // Ribbon quad
        let offset = Vec3::new(0.0, 0.0, ribbon_half_w);
        mesh.add_quad(
            Vertex::new(p0 - offset, col, Vec3::UP),
            Vertex::new(p0 + offset, col, Vec3::UP),
            Vertex::new(p1 + offset, col, Vec3::UP),
            Vertex::new(p1 - offset, col, Vec3::UP),
            true, // Emissive for neon glow
        );
    }

    mesh
}
