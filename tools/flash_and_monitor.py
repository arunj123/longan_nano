import os
import sys
import time
import threading
import paramiko

HOST = "192.168.0.63"
USER = "arun"
PASS = "arun"
REMOTE_DIR = "/home/arun/longan_nano_tools"
REMOTE_CFG = f"{REMOTE_DIR}/openocd-sipeed-libusb.cfg"

def run(hex_path, monitor_duration=12):
    print(f"=== FLASH AND MONITOR TEST ===")
    hex_path = os.path.abspath(hex_path)
    if not os.path.exists(hex_path):
        print(f"Error: {hex_path} does not exist.")
        sys.exit(1)

    # 1. Connect SSH for monitor
    print(f"[1/4] Connecting to {USER}@{HOST} for UART monitor...")
    mon_client = paramiko.SSHClient()
    mon_client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
    mon_client.connect(HOST, username=USER, password=PASS, timeout=15)

    _, mon_out, _ = mon_client.exec_command(
        "python3 -u /home/arun/longan_nano_tools/uart_monitor.py",
        get_pty=True
    )

    stop_event = threading.Event()

    def reader():
        while not stop_event.is_set():
            line = mon_out.readline()
            if not line:
                break
            print(f"[UART] {line.rstrip()}")

    t = threading.Thread(target=reader, daemon=True)
    t.start()

    time.sleep(1.5) # Allow monitor to attach to serial device

    # 2. Flash via second SSH connection
    print(f"[2/4] Uploading {os.path.basename(hex_path)}...")
    flash_client = paramiko.SSHClient()
    flash_client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
    flash_client.connect(HOST, username=USER, password=PASS, timeout=15)

    sftp = flash_client.open_sftp()
    remote_hex = f"{REMOTE_DIR}/{os.path.basename(hex_path)}"
    sftp.put(hex_path, remote_hex)
    sftp.close()

    print(f"[3/4] Programming target via OpenOCD...")
    program_cmd = f'program "{remote_hex}" verify; halt; reg pc 0x08000000; resume; shutdown'
    full_cmd = f"openocd -f {REMOTE_CFG} -c '{program_cmd}'"

    _, f_out, f_err = flash_client.exec_command(full_cmd)
    exit_code = f_out.channel.recv_exit_status()
    output = f_out.read().decode() + f_err.read().decode()
    if exit_code == 0 and "Verified OK" in output:
        print("[FLASH OK] GD32VF103 programmed and resumed at 0x08000000 successfully!")
    else:
        print("[FLASH FAIL]:", output)

    flash_client.close()

    # 3. Stream serial logs
    print(f"[4/4] Streaming UART0 logs for {monitor_duration} seconds...")
    time.sleep(monitor_duration)

    stop_event.set()
    mon_client.close()
    print("=== TEST COMPLETE ===")

if __name__ == "__main__":
    hex_file = sys.argv[1] if len(sys.argv) > 1 else "build/prj_sdcard_test/rust/firmware_rust.hex"
    run(hex_file)
