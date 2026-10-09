#!/usr/bin/env python3
"""
Longan Nano INA219 Live Monitor & Bench Suite
Features:
  - High-frequency USB HID receiver (10 Hz stream)
  - Live ANSI terminal dashboard with Min/Max/Avg, Energy, Charge & Load Resistance
  - Dynamic ASCII visual bar meters
  - Auto-reconnecting USB link resilience
  - CSV recording on host PC
  - Optional real-time GUI oscilloscope waveform plotting (--gui)
"""

import sys
import time
import struct
import argparse
from datetime import datetime

try:
    import hid
except ImportError:
    print("❌ Error: 'hidapi' library is required. Install with: pip install hidapi")
    sys.exit(1)

VID = 0x28E9
PID = 0x1234

class LiveStats:
    def __init__(self):
        self.samples = 0
        self.v_min = float('inf')
        self.v_max = float('-inf')
        self.v_sum = 0.0
        self.c_min = float('inf')
        self.c_max = float('-inf')
        self.c_sum = 0.0
        self.p_max = float('-inf')
        self.p_sum = 0.0
        self.mwh_accum = 0.0
        self.mah_accum = 0.0
        self.last_ts = time.time()
        self.start_time = time.time()

    def update(self, v_mv, c_ma, p_mw):
        now = time.time()
        dt_hr = (now - self.last_ts) / 3600.0
        self.last_ts = now

        self.samples += 1
        self.v_sum += v_mv
        self.c_sum += c_ma
        self.p_sum += p_mw

        if v_mv < self.v_min: self.v_min = v_mv
        if v_mv > self.v_max: self.v_max = v_mv
        if c_ma < self.c_min: self.c_min = c_ma
        if c_ma > self.c_max: self.c_max = c_ma
        if p_mw > self.p_max: self.p_max = p_mw

        if v_mv > 500:
            self.mwh_accum += p_mw * dt_hr
            self.mah_accum += abs(c_ma) * dt_hr

    def resistance_str(self, v_mv, c_ma):
        if v_mv > 500 and abs(c_ma) > 2.0:
            r = (v_mv / 1000.0) / (abs(c_ma) / 1000.0)
            if r < 1000.0:
                return f"{r:>6.1f} Ω"
            elif r < 100000.0:
                return f"{r/1000.0:>6.2f} kΩ"
            else:
                return f"{r/1000000.0:>6.2f} MΩ"
        return "    --- Ω"

def make_bar(val, max_val, length=24):
    ratio = max(0.0, min(1.0, val / max_val if max_val > 0 else 0.0))
    filled = int(ratio * length)
    return "█" * filled + "░" * (length - filled)

def run_cli_dashboard(csv_file=None):
    stats = LiveStats()
    csv_writer = None
    if csv_file:
        csv_writer = open(csv_file, 'a', buffering=1, encoding='utf-8')
        if csv_writer.tell() == 0:
            csv_writer.write("Timestamp,Uptime_s,Voltage_mV,Current_mA,Power_mW,Energy_mWh,Charge_mAh\n")

    print(f"\n🔍 Searching for Longan Nano INA219 (VID: 0x{VID:04X}, PID: 0x{PID:04X})...")

    while True:
        device = None
        try:
            device = hid.device()
            device.open(VID, PID)
            device.set_nonblocking(True)
            print(" Connected! Streaming telemetry (Press Ctrl+C to exit)...\n")

            while True:
                report = device.read(64)
                if report and report[0] == 0x01 and len(report) >= 7:
                    v_mv, c_ma, p_mw = struct.unpack('<HhH', bytes(report[1:7]))
                    stats.update(v_mv, c_ma, p_mw)

                    if csv_writer:
                        iso = datetime.now().strftime("%Y-%m-%d %H:%M:%S.%f")[:-3]
                        csv_writer.write(f"{iso},{time.time()-stats.start_time:.2f},{v_mv},{c_ma},{p_mw},{stats.mwh_accum:.2f},{stats.mah_accum:.2f}\n")

                    uptime_s = int(time.time() - stats.start_time)
                    h, m, s = uptime_s // 3600, (uptime_s % 3600) // 60, uptime_s % 60
                    r_str = stats.resistance_str(v_mv, c_ma)

                    v_bar = make_bar(v_mv, 6000, 20)
                    c_bar = make_bar(abs(c_ma), 1000, 20)

                    # Dynamic ANSI terminal dashboard
                    output = (
                        f"\033[H\033[J" # Clear screen
                        f"╔════════════════════════════════════════════════════════════════════════╗\n"
                        f"║             LONGAN NANO INA219 DC CURRENT & POWER MONITOR              ║\n"
                        f"╠════════════════════════════════════════════════════════════════════════╣\n"
                        f"║ Uptime: {h:02d}:{m:02d}:{s:02d} | Packets: {stats.samples:<6} | Load Impedance: {r_str:<12}       ║\n"
                        f"╠════════════════════════════════════════════════════════════════════════╣\n"
                        f"║ VOLTAGE : {v_mv/1000.0:>6.3f} V   [{v_bar}] Min: {stats.v_min/1000.0:>5.2f}V Max: {stats.v_max/1000.0:>5.2f}V ║\n"
                        f"║ CURRENT : {c_ma:>7.1f} mA  [{c_bar}] Min: {stats.c_min:>6.1f}  Max: {stats.c_max:>6.1f}mA ║\n"
                        f"║ POWER   : {p_mw:>7.1f} mW                            Peak: {stats.p_max:>7.1f} mW ║\n"
                        f"╠════════════════════════════════════════════════════════════════════════╣\n"
                        f"║ ENERGY ACCUMULATOR: {stats.mwh_accum:>8.2f} mWh   | CHARGE: {stats.mah_accum:>8.2f} mAh          ║\n"
                        f"╚════════════════════════════════════════════════════════════════════════╝\n"
                    )
                    sys.stdout.write(output)
                    sys.stdout.flush()

                time.sleep(0.01)

        except KeyboardInterrupt:
            print("\n⏹️ Monitor stopped by user.")
            break
        except Exception as e:
            sys.stdout.write(f"\r⚠️ Device disconnected or error: {e}. Retrying in 1s...\n")
            sys.stdout.flush()
            time.sleep(1.0)
        finally:
            if device:
                try: device.close()
                except: pass

    if csv_writer:
        csv_writer.close()
        print(f"📁 Host log saved to: {csv_file}")

def run_gui_plotter(csv_file=None):
    try:
        import matplotlib.pyplot as plt
        import matplotlib.animation as animation
        from collections import deque
    except ImportError:
        print("❌ Error: 'matplotlib' is required for GUI mode. Install with: pip install matplotlib")
        print("Falling back to terminal dashboard...")
        run_cli_dashboard(csv_file)
        return

    stats = LiveStats()
    csv_writer = None
    if csv_file:
        csv_writer = open(csv_file, 'a', buffering=1, encoding='utf-8')
        if csv_writer.tell() == 0:
            csv_writer.write("Timestamp,Uptime_s,Voltage_mV,Current_mA,Power_mW,Energy_mWh,Charge_mAh\n")

    history_len = 200
    times = deque(maxlen=history_len)
    voltages = deque(maxlen=history_len)
    currents = deque(maxlen=history_len)
    powers = deque(maxlen=history_len)

    device = hid.device()
    try:
        device.open(VID, PID)
        device.set_nonblocking(True)
    except Exception as e:
        print(f"❌ Failed to open HID device: {e}")
        return

    fig, (ax_i, ax_v, ax_p) = plt.subplots(3, 1, figsize=(10, 8), sharex=True)
    fig.patch.set_facecolor('#0d1117')
    for ax in (ax_i, ax_v, ax_p):
        ax.set_facecolor('#161b22')
        ax.grid(True, color='#30363d', linestyle='--', alpha=0.6)
        ax.tick_params(colors='#c9d1d9')

    line_i, = ax_i.plot([], [], color='#00f0ff', linewidth=1.8, label='Current (mA)')
    line_v, = ax_v.plot([], [], color='#2ea043', linewidth=1.8, label='Voltage (V)')
    line_p, = ax_p.plot([], [], color='#e3b341', linewidth=1.8, label='Power (mW)')

    ax_i.legend(loc='upper right', facecolor='#0d1117', edgecolor='#30363d', labelcolor='#c9d1d9')
    ax_v.legend(loc='upper right', facecolor='#0d1117', edgecolor='#30363d', labelcolor='#c9d1d9')
    ax_p.legend(loc='upper right', facecolor='#0d1117', edgecolor='#30363d', labelcolor='#c9d1d9')
    ax_p.set_xlabel('Time (s)', color='#c9d1d9')

    def update_plot(_):
        while True:
            report = device.read(64)
            if not report:
                break
            if report[0] == 0x01 and len(report) >= 7:
                v_mv, c_ma, p_mw = struct.unpack('<HhH', bytes(report[1:7]))
                stats.update(v_mv, c_ma, p_mw)
                t_rel = time.time() - stats.start_time

                times.append(t_rel)
                voltages.append(v_mv / 1000.0)
                currents.append(c_ma)
                powers.append(p_mw)

                if csv_writer:
                    iso = datetime.now().strftime("%Y-%m-%d %H:%M:%S.%f")[:-3]
                    csv_writer.write(f"{iso},{t_rel:.2f},{v_mv},{c_ma},{p_mw},{stats.mwh_accum:.2f},{stats.mah_accum:.2f}\n")

        if times:
            t_list = list(times)
            line_i.set_data(t_list, list(currents))
            line_v.set_data(t_list, list(voltages))
            line_p.set_data(t_list, list(powers))

            for ax, data in zip((ax_i, ax_v, ax_p), (currents, voltages, powers)):
                ax.set_xlim(t_list[0], max(t_list[0] + 5, t_list[-1]))
                d_min, d_max = min(data), max(data)
                span = max(1.0, d_max - d_min)
                ax.set_ylim(d_min - span * 0.1, d_max + span * 0.1)

            fig.suptitle(
                f"Longan Nano Live Monitor | E: {stats.mwh_accum:.2f} mWh | Q: {stats.mah_accum:.2f} mAh | {stats.resistance_str(voltages[-1]*1000, currents[-1])}",
                color='#58a6ff', fontsize=12, fontweight='bold'
            )
        return line_i, line_v, line_p

    ani = animation.FuncAnimation(fig, update_plot, interval=50, blit=False)
    plt.tight_layout()
    try:
        plt.show()
    finally:
        device.close()
        if csv_writer:
            csv_writer.close()
            print(f"📁 Host log saved to: {csv_file}")

def main():
    parser = argparse.ArgumentParser(description="Longan Nano INA219 Live Host Monitor")
    parser.add_argument("--csv", type=str, default="host_monitor_log.csv", help="Path to host CSV log file")
    parser.add_argument("--gui", action="store_true", help="Launch live graphical waveform plotter (requires matplotlib)")
    args = parser.parse_args()

    if args.gui:
        run_gui_plotter(args.csv)
    else:
        run_cli_dashboard(args.csv)

if __name__ == "__main__":
    main()
