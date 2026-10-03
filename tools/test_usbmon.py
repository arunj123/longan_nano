import paramiko
import time
import threading

client = paramiko.SSHClient()
client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
client.connect("192.168.0.63", username="arun", password="arun")

stop_event = threading.Event()

def monitor_usbmon():
    # Read usbmon 1u
    stdin, stdout, stderr = client.exec_command("echo arun | sudo -S cat /sys/kernel/debug/usb/usbmon/1u")
    while not stop_event.is_set():
        line = stdout.readline()
        if not line:
            break
        # Filter for control transfers on EP 0 (e.g. "Ci:" or "Co:" or "s 80 06" etc.)
        if " 1:" in line or " 00000000" in line or "Ci:" in line or "Co:" in line:
            print("[USBMON]", line.strip())

mon_thread = threading.Thread(target=monitor_usbmon, daemon=True)
mon_thread.start()

# Now unstick / reset port 1-1 to trigger re-enumeration
print("[HOST] Resetting port 1-1...")
client.exec_command("echo 1 | sudo -S tee /sys/bus/usb/devices/usb1/1-0:1.0/usb1-port1/disable; sleep 0.5; echo 0 | sudo -S tee /sys/bus/usb/devices/usb1/1-0:1.0/usb1-port1/disable")

time.sleep(5)
stop_event.set()
client.close()
