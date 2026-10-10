use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::model::{HidCommand, TelemetryPacket};

pub const VID: u16 = 0x28E9;
pub const PID: u16 = 0x1234;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportKind {
    LocalHid,
    TcpBridge(String),
    Simulation,
}

pub struct TransportManager {
    pub kind: TransportKind,
    packet_rx: Receiver<TelemetryPacket>,
    command_tx: Sender<HidCommand>,
    is_connected: Arc<Mutex<bool>>,
    status_msg: Arc<Mutex<String>>,
}

impl TransportManager {
    pub fn new(kind: TransportKind) -> Self {
        let (packet_tx, packet_rx) = channel();
        let (command_tx, command_rx) = channel();
        let is_connected = Arc::new(Mutex::new(false));
        let status_msg = Arc::new(Mutex::new("Initializing...".to_string()));

        let conn_flag = Arc::clone(&is_connected);
        let status_clone = Arc::clone(&status_msg);
        let kind_clone = kind.clone();

        thread::spawn(move || {
            worker_loop(kind_clone, packet_tx, command_rx, conn_flag, status_clone);
        });

        Self {
            kind,
            packet_rx,
            command_tx,
            is_connected,
            status_msg,
        }
    }

    pub fn poll_packet(&self) -> Option<TelemetryPacket> {
        self.packet_rx.try_recv().ok()
    }

    pub fn poll_all_packets(&self) -> Vec<TelemetryPacket> {
        let mut packets = Vec::new();
        while let Ok(pkt) = self.packet_rx.try_recv() {
            packets.push(pkt);
        }
        packets
    }

    pub fn send_command(&self, cmd: HidCommand) {
        let _ = self.command_tx.send(cmd);
    }

    pub fn is_connected(&self) -> bool {
        *self.is_connected.lock().unwrap()
    }

    pub fn status_text(&self) -> String {
        self.status_msg.lock().unwrap().clone()
    }
}

fn parse_report_bytes(buf: &[u8], start_time: Instant) -> Option<TelemetryPacket> {
    if buf.is_empty() {
        return None;
    }
    // Find Report ID 0x01
    let pos = buf.iter().position(|&b| b == 0x01)?;
    if buf.len() < pos + 9 {
        return None;
    }
    let p = &buf[pos..pos + 9];
    let v_mv = u16::from_le_bytes([p[1], p[2]]);
    let c_raw = i16::from_le_bytes([p[3], p[4]]);
    let p_mw = u16::from_le_bytes([p[5], p[6]]);
    let flags = p[7];
    let seq = p[8];

    let now = Instant::now();
    Some(TelemetryPacket {
        timestamp: now,
        relative_secs: now.duration_since(start_time).as_secs_f64(),
        voltage_mv: v_mv,
        current_tenth_ma: c_raw,
        power_mw: p_mw,
        flags,
        seq,
    })
}

fn worker_loop(
    kind: TransportKind,
    packet_tx: Sender<TelemetryPacket>,
    command_rx: Receiver<HidCommand>,
    is_connected: Arc<Mutex<bool>>,
    status_msg: Arc<Mutex<String>>,
) {
    let start_time = Instant::now();

    match kind {
        TransportKind::Simulation => {
            *is_connected.lock().unwrap() = true;
            *status_msg.lock().unwrap() = "Simulation Mode Active".to_string();

            let mut seq = 0u8;
            let mut tick = 0u64;
            let mut tare_offset = 0.0;
            let mut current_limit = 2000.0;

            loop {
                // Check commands
                while let Ok(cmd) = command_rx.try_recv() {
                    match cmd {
                        HidCommand::TareZero => {
                            tare_offset += 25.0; // Simulate zero tare
                        }
                        HidCommand::SetCurrentLimit(lim) => {
                            current_limit = lim as f64;
                        }
                        _ => {}
                    }
                }

                let t = tick as f64 * 0.1;
                // Realistic dynamic current waveform:
                // Base 35mA + 50mA pulsing every 3s + 150mA RF pulse every 7s + minor noise
                let base = 35.0;
                let pulse_mcu = if (t % 3.0) < 0.6 { 65.0 } else { 0.0 };
                let pulse_rf = if (t % 7.0) < 0.15 { 180.0 } else { 0.0 };
                let noise = ((tick % 7) as f64 - 3.0) * 0.4;
                let c_ma = (base + pulse_mcu + pulse_rf + noise - tare_offset).max(0.0);

                let v_mv = (3300.0 - (c_ma * 0.4)).round() as u16; // Slight drop under load
                let p_mw = ((v_mv as f64 / 1000.0) * c_ma).round() as u16;
                let mut flags = 0x01 | 0x08; // Online + SD logging
                if c_ma > current_limit {
                    flags |= 0x10; // Alert
                }

                let now = Instant::now();
                let pkt = TelemetryPacket {
                    timestamp: now,
                    relative_secs: now.duration_since(start_time).as_secs_f64(),
                    voltage_mv: v_mv,
                    current_tenth_ma: (c_ma * 10.0).round() as i16,
                    power_mw: p_mw,
                    flags,
                    seq,
                };

                let _ = packet_tx.send(pkt);
                seq = seq.wrapping_add(1);
                tick += 1;

                thread::sleep(Duration::from_millis(100)); // 10 Hz
            }
        }

        TransportKind::TcpBridge(addr_str) => {
            loop {
                *is_connected.lock().unwrap() = false;
                *status_msg.lock().unwrap() = format!("Connecting to TCP bridge at {}...", addr_str);

                match TcpStream::connect(&addr_str) {
                    Ok(mut stream) => {
                        *is_connected.lock().unwrap() = true;
                        *status_msg.lock().unwrap() = format!("Connected: TCP Bridge {}", addr_str);
                        stream.set_nonblocking(true).ok();

                        let mut buf = [0u8; 256];
                        let mut stream_open = true;

                        while stream_open {
                            // Forward commands to TCP
                            while let Ok(cmd) = command_rx.try_recv() {
                                let bytes = cmd.to_bytes();
                                if let Err(e) = stream.write_all(&bytes) {
                                    *status_msg.lock().unwrap() = format!("Command send error: {}", e);
                                    stream_open = false;
                                    break;
                                }
                            }

                            // Read incoming packets
                            match stream.read(&mut buf) {
                                Ok(0) => {
                                    // Remote disconnected
                                    stream_open = false;
                                }
                                Ok(n) => {
                                    if let Some(pkt) = parse_report_bytes(&buf[..n], start_time) {
                                        let _ = packet_tx.send(pkt);
                                    }
                                }
                                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                    // No data ready
                                }
                                Err(_) => {
                                    stream_open = false;
                                }
                            }

                            thread::sleep(Duration::from_millis(10));
                        }
                    }
                    Err(e) => {
                        *status_msg.lock().unwrap() = format!("Bridge connection failed: {}. Retrying in 2s...", e);
                        thread::sleep(Duration::from_secs(2));
                    }
                }
            }
        }

        TransportKind::LocalHid => {
            loop {
                *is_connected.lock().unwrap() = false;
                *status_msg.lock().unwrap() = format!("Scanning for USB HID (VID: 0x{:04X}, PID: 0x{:04X})...", VID, PID);

                match hidapi::HidApi::new() {
                    Ok(api) => {
                        match api.open(VID, PID) {
                            Ok(dev) => {
                                *is_connected.lock().unwrap() = true;
                                *status_msg.lock().unwrap() = "Connected: Direct USB HID".to_string();
                                dev.set_blocking_mode(false).ok();

                                let mut buf = [0u8; 64];
                                let mut dev_open = true;

                                while dev_open {
                                    // Send queued commands
                                    while let Ok(cmd) = command_rx.try_recv() {
                                        let bytes = cmd.to_bytes();
                                        if let Err(e) = dev.write(&bytes) {
                                            *status_msg.lock().unwrap() = format!("HID write error: {}", e);
                                            dev_open = false;
                                            break;
                                        }
                                    }

                                    // Read telemetry reports
                                    match dev.read(&mut buf) {
                                        Ok(n) if n >= 7 => {
                                            if let Some(pkt) = parse_report_bytes(&buf[..n], start_time) {
                                                let _ = packet_tx.send(pkt);
                                            }
                                        }
                                        Ok(_) => {}
                                        Err(_) => {
                                            dev_open = false;
                                        }
                                    }

                                    thread::sleep(Duration::from_millis(10));
                                }
                            }
                            Err(_) => {
                                thread::sleep(Duration::from_secs(1));
                            }
                        }
                    }
                    Err(e) => {
                        *status_msg.lock().unwrap() = format!("HID API initialization error: {}", e);
                        thread::sleep(Duration::from_secs(2));
                    }
                }
            }
        }
    }
}
