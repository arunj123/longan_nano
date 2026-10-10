#!/usr/bin/env python3
"""
Remote UI Cycling & Command Console Test for Longan Nano Current Monitor (Build 00C4)
Connects to Linux testbed (192.168.0.63) and exercises:
1. UART0 Remote Command Console ('?', '1'..'4', 'm', 's', 'f', 'r', 't')
2. MicroSD Sequential File Rotation (LOG_XXXX.CSV)
3. Bidirectional USB HID OUT Endpoint (/dev/hidraw0 Report ID 0x02)
4. Telemetry Stream Validation (/dev/hidraw0 Report ID 0x01)
"""

import sys
import time
import json
import struct
import paramiko

HOST = "192.168.0.63"
USER = "arun"
PASS = "arun"

REMOTE_TEST_HARNESS = r"""
import sys
import time
import json
import struct
import serial
import os

results = {}

# 1. Open Serial Port to UART0 (/dev/ttyUSB1 @ 115200)
try:
    ser = serial.Serial('/dev/ttyUSB1', 115200, timeout=0.5)
    ser.reset_input_buffer()
    results['serial_open'] = True
except Exception as e:
    results['serial_open'] = False
    results['error'] = f"Failed to open /dev/ttyUSB1: {e}"
    print(json.dumps(results))
    sys.exit(1)

def send_and_expect(cmd_str, expected_substrs, timeout=2.5):
    ser.reset_input_buffer()
    ser.write(cmd_str.encode())
    ser.flush()
    t_start = time.time()
    captured = []
    found_all = False
    while time.time() - t_start < timeout:
        line = ser.readline().decode('utf-8', errors='replace').strip()
        if line:
            captured.append(line)
            # check if all expected substrings are in any captured lines
            full_text = "\n".join(captured)
            if all(sub in full_text for sub in expected_substrs):
                found_all = True
                break
    return found_all, captured

# Wait for a baseline heartbeat
time.sleep(1.2)
ser.reset_input_buffer()

# TEST 1: Help Menu
ok, lines = send_and_expect('?', ['=== REMOTE COMMAND CONSOLE ===', 'Set Screen'], timeout=2.0)
results['test_help'] = {'pass': ok, 'lines': lines}

# TEST 2: Screen Mode Switching via UART ('1', '2', '3', '4', 'm')
modes_tested = {}
for key, expected_mode, expected_label in [
    ('1', '1: HERO', 'HERO'),
    ('2', '2: GRAPH', 'GRAPH'),
    ('3', '3: STATS', 'STATS'),
    ('4', '4: HISTOGRAM', 'HISTO'),
    ('m', '1: HERO', 'HERO'),
]:
    ok, lines = send_and_expect(key, [f'[UI] Screen mode: {expected_mode}'], timeout=1.5)
    modes_tested[f"mode_{key}"] = {'pass': ok, 'lines': lines}
results['test_modes'] = modes_tested

# TEST 3: Telemetry Summary Dump ('s')
ok, lines = send_and_expect('s', ['--- SESSION TELEMETRY SUMMARY ---', '[JSON]'], timeout=2.0)
json_data = None
for l in lines:
    if l.startswith('[JSON] '):
        try:
            json_data = json.loads(l[7:])
        except:
            pass
results['test_dump'] = {'pass': ok and (json_data is not None), 'json': json_data}

# TEST 4: Force SD Buffer Flush ('f')
ok, lines = send_and_expect('f', ['[SD] Flushed buffer to'], timeout=1.5)
results['test_sd_flush'] = {'pass': ok, 'lines': lines}

# TEST 5: MicroSD Sequential Session Rotation ('r')
ok, lines = send_and_expect('r', ['[SD] Rotated to new file: LOG_'], timeout=1.5)
new_file = None
for l in lines:
    if 'Rotated to new file:' in l:
        parts = l.split('Rotated to new file:')
        if len(parts) > 1:
            new_file = parts[1].strip()
results['test_sd_rotate'] = {'pass': ok, 'new_file': new_file}

# TEST 6: Zero-Tare & Accumulator Reset ('t')
ok, lines = send_and_expect('t', ['[SYS] Session Reset! Rotated to: LOG_'], timeout=2.0)
results['test_tare_reset'] = {'pass': ok, 'lines': lines}

# TEST 7: USB HID Telemetry Stream Validation (/dev/hidraw0 Input Report ID 1)
hid_in_ok = False
hid_reading = None
try:
    with open('/dev/hidraw0', 'rb') as f:
        # Read 9-byte input report
        data = f.read(9)
        if len(data) >= 9 and data[0] == 0x01:
            rid, v, i, p, flg, seq = struct.unpack('<BHhHBB', data)
            hid_reading = {'v_mv': v, 'i_tenth_ma': i, 'p_mw': p, 'flags': flg, 'seq': seq}
            hid_in_ok = True
except Exception as e:
    results['hid_in_error'] = str(e)
results['test_hid_in'] = {'pass': hid_in_ok, 'reading': hid_reading}

# TEST 8: USB HID Bidirectional OUT Command (Report ID 2 -> Set Mode: 2 [STATS])
hid_out_ok = False
try:
    # Packet layout: [0x02 Report ID, 0x01 SetMode Cmd, 0x02 Stats Mode, 0, 0, 0, 0, 0, 0]
    out_pkt = struct.pack('<BBBBBBBBB', 0x02, 0x01, 0x02, 0, 0, 0, 0, 0, 0)
    ser.reset_input_buffer()
    with open('/dev/hidraw0', 'wb') as f:
        f.write(out_pkt)
        f.flush()
    # Verify screen mode changed to Stats on serial log
    t0 = time.time()
    while time.time() - t0 < 2.0:
        line = ser.readline().decode('utf-8', errors='replace').strip()
        if '[UI] Screen mode: 3: STATS' in line:
            hid_out_ok = True
            break
except Exception as e:
    results['hid_out_error'] = str(e)
results['test_hid_out'] = {'pass': hid_out_ok}

ser.close()
print("FINAL_RESULTS:" + json.dumps(results))
"""

def main():
    print("=" * 70)
    print(" Running Build 00C4 Remote Command & Telemetry Validation Test")
    print(f" Target Testbed: {USER}@{HOST}")
    print("=" * 70)

    client = paramiko.SSHClient()
    client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
    try:
        client.connect(HOST, username=USER, password=PASS, timeout=10)
    except Exception as e:
        print(f"[FAIL] SSH Connection Failed: {e}")
        sys.exit(1)

    print("[1/2] Deploying test harness to remote testbed...")
    sftp = client.open_sftp()
    with sftp.file("/home/arun/longan_nano_tools/test_harness.py", "w") as f:
        f.write(REMOTE_TEST_HARNESS)
    sftp.close()

    print("[2/2] Executing remote test harness against GD32VF103 hardware...")
    stdin, stdout, stderr = client.exec_command(
        "python3 -u /home/arun/longan_nano_tools/test_harness.py",
        get_pty=True
    )

    output_lines = []
    raw_results = None
    for line in iter(stdout.readline, ""):
        line = line.strip()
        if not line:
            continue
        if line.startswith("FINAL_RESULTS:"):
            raw_results = line[len("FINAL_RESULTS:"):]
        else:
            output_lines.append(line)

    err = stderr.read().decode().strip()
    client.close()

    if not raw_results:
        print("[FAIL] Test harness did not return final results JSON!")
        if err:
            print(f"Stderr: {err}")
        if output_lines:
            print(f"Stdout:\n" + "\n".join(output_lines))
        sys.exit(1)

    res = json.loads(raw_results)

    print("\n" + "=" * 70)
    print(" EMPIRICAL TEST RESULTS (BUILD 00C4)")
    print("=" * 70)

    all_passed = True

    # Test 1: Help Menu
    p = res.get('test_help', {}).get('pass', False)
    all_passed &= p
    print(f" [TEST 1] Help Menu ('?'): {'PASS' if p else 'FAIL'}")

    # Test 2: Mode switching
    modes = res.get('test_modes', {})
    for k, v in modes.items():
        mp = v.get('pass', False)
        all_passed &= mp
        print(f" [TEST 2] Screen Switch '{k}': {'PASS' if mp else 'FAIL'}")

    # Test 3: Telemetry Dump
    dump = res.get('test_dump', {})
    dp = dump.get('pass', False)
    all_passed &= dp
    print(f" [TEST 3] Session Dump ('s') & JSON parse: {'PASS' if dp else 'FAIL'}")
    if dp and dump.get('json'):
        print(f"          JSON Payload: {json.dumps(dump['json'])}")

    # Test 4: Force SD Flush
    f_res = res.get('test_sd_flush', {})
    fp = f_res.get('pass', False)
    all_passed &= fp
    print(f" [TEST 4] Force SD Flush ('f'): {'PASS' if fp else 'FAIL'}")

    # Test 5: SD Rotation
    r_res = res.get('test_sd_rotate', {})
    rp = r_res.get('pass', False)
    all_passed &= rp
    print(f" [TEST 5] SD File Rotation ('r'): {'PASS' if rp else 'FAIL'} (Active: {r_res.get('new_file')})")

    # Test 6: Tare Reset
    t_res = res.get('test_tare_reset', {})
    tp = t_res.get('pass', False)
    all_passed &= tp
    print(f" [TEST 6] Zero-Tare & Reset ('t'): {'PASS' if tp else 'FAIL'}")

    # Test 7: HID Input Report
    hid_in = res.get('test_hid_in', {})
    hip = hid_in.get('pass', False)
    all_passed &= hip
    print(f" [TEST 7] USB HID Telemetry (Report ID 1): {'PASS' if hip else 'FAIL'}")
    if hip and hid_in.get('reading'):
        print(f"          Reading: {hid_in['reading']}")

    # Test 8: HID Output Report
    hid_out = res.get('test_hid_out', {})
    hop = hid_out.get('pass', False)
    all_passed &= hop
    print(f" [TEST 8] USB HID OUT Command (Report ID 2 -> SetMode): {'PASS' if hop else 'FAIL'}")

    print("=" * 70)
    if all_passed:
        print(" >>> ALL 8 TEST SUITES PASSED CLEANLY ON TARGET HARDWARE <<<")
        print("=" * 70)
        sys.exit(0)
    else:
        print(" >>> SOME TESTS FAILED - DIAGNOSTICS REQUIRED <<<")
        print(f" Raw Results: {json.dumps(res, indent=2)}")
        print("=" * 70)
        sys.exit(1)

if __name__ == "__main__":
    main()
