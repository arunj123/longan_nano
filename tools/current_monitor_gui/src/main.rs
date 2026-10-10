#![allow(dead_code, unused_imports)]

mod app;
mod graphics3d;
mod model;
mod transport;
mod ui;

use std::env;
use eframe::egui::ViewportBuilder;
use app::CurrentMonitorApp;
use transport::TransportKind;

fn main() -> eframe::Result<()> {
    let args: Vec<String> = env::args().collect();

    let initial_transport = if args.iter().any(|a| a == "--sim" || a == "--simulation") {
        TransportKind::Simulation
    } else if let Some(pos) = args.iter().position(|a| a == "--tcp") {
        let addr = args.get(pos + 1).cloned().unwrap_or_else(|| "127.0.0.1:5055".to_string());
        TransportKind::TcpBridge(addr)
    } else if args.iter().any(|a| a == "--remote") {
        TransportKind::TcpBridge("192.168.0.63:5055".to_string())
    } else if args.iter().any(|a| a == "--hid") {
        TransportKind::LocalHid
    } else {
        // Default: connect to local TCP bridge (or user can switch to HID or remote in header)
        TransportKind::TcpBridge("127.0.0.1:5055".to_string())
    };

    let native_options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title("⚡ Longan Nano INA219 Current & Power Monitor - High-Performance 3D Suite")
            .with_inner_size([1360.0, 840.0])
            .with_min_inner_size([960.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Longan Nano Current Monitor",
        native_options,
        Box::new(move |_cc| Box::new(CurrentMonitorApp::new(initial_transport))),
    )
}
