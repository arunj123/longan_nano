#!/usr/bin/env python3
"""
Remote UI Cycling & Command Console Test for Longan Nano Current Monitor (Build 00C8)
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

def send_and_expect(cmd_str, expected_substrs, timeout=2.5, char_delay=0.0):
    ser.reset_input_buffer()
    if char_delay > 0:
        for ch in cmd_str:
            ser.write(ch.encode())
            ser.flush()
            time.sleep(char_delay)
    else:
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
ok, lines = send_and_expect('?', ['=== REMOTE COMMAND CONSOLE ===', 'Set Screen', 'Cycle battery capacity profile'], timeout=2.0)
results['test_help'] = {'pass': ok, 'lines': lines}

# TEST 2: Screen Mode Switching via UART ('1', '2', '3', '4', '5', 'm')
modes_tested = {}
for key, expected_mode, expected_label in [
    ('1', '1: HERO', 'HERO'),
    ('2', '2: GRAPH', 'GRAPH'),
    ('3', '3: STATS', 'STATS'),
    ('4', '4: HISTOGRAM', 'HISTO'),
    ('5', '5: BIG DIGIT', 'BIG'),
    ('m', '1: HERO', 'HERO'),
]:
    ok, lines = send_and_expect(key, [f'[UI] Screen mode: {expected_mode}'], timeout=1.5)
    modes_tested[f"mode_{key}"] = {'pass': ok, 'lines': lines}
results['test_modes'] = modes_tested

# TEST 3: Battery Capacity Profile Cycling ('b')
ok, lines = send_and_expect('b', ['[BATT] Switched battery profile to:'], timeout=1.5)
results['test_battery'] = {'pass': ok, 'lines': lines}

# TEST 4: Telemetry Summary Dump ('s') with MCU Temperature & Battery SoC
ok, lines = send_and_expect('s', ['--- SESSION TELEMETRY SUMMARY ---', 'MCU Temp:', '[JSON]'], timeout=2.0)
json_data = None
for l in lines:
    if l.startswith('[JSON] '):
        try:
            json_data = json.loads(l[7:])
        except:
            pass
has_temp = False
if json_data and 'mcu_temp_c' in json_data and 'bat_soc' in json_data and 'epoch' in json_data and 'time' in json_data:
    has_temp = True
results['test_dump'] = {'pass': ok and (json_data is not None) and has_temp, 'json': json_data}

# TEST 5: Force SD Buffer Flush ('f')
ok, lines = send_and_expect('f', ['[SD] Flushed buffer to'], timeout=1.5)
results['test_sd_flush'] = {'pass': ok, 'lines': lines}

# TEST 6: MicroSD Sequential Session Rotation ('r')
ok, lines = send_and_expect('r', ['[SD] Rotated to new file: LOG_'], timeout=1.5)
new_file = None
for l in lines:
    if 'Rotated to new file:' in l:
        parts = l.split('Rotated to new file:')
        if len(parts) > 1:
            new_file = parts[1].strip()
results['test_sd_rotate'] = {'pass': ok, 'new_file': new_file}

# TEST 7: Zero-Tare & Accumulator Reset ('t')
ok, lines = send_and_expect('t', ['[SYS] Session Reset! Rotated to: LOG_'], timeout=2.0)
results['test_tare_reset'] = {'pass': ok, 'lines': lines}

# TEST 8: USB HID Telemetry Stream Validation (/dev/hidraw0 Input Report ID 1)
hid_in_ok = False
hid_reading = None
try:
    with open('/dev/hidraw0', 'rb') as f:
        data = f.read(9)
        if len(data) >= 9 and data[0] == 0x01:
            rid, v, i, p, flg, seq = struct.unpack('<BHhHBB', data)
            hid_reading = {'v_mv': v, 'i_tenth_ma': i, 'p_mw': p, 'flags': flg, 'seq': seq}
            hid_in_ok = True
except Exception as e:
    results['hid_in_error'] = str(e)
results['test_hid_in'] = {'pass': hid_in_ok, 'reading': hid_reading}

# TEST 9: USB HID Bidirectional OUT Command (Report ID 2 -> Set Mode: 4 [BIG DIGIT])
hid_out_ok = False
try:
    # Packet layout: [0x02 Report ID, 0x01 SetMode Cmd, 0x04 BigDigit Mode, 0, 0, 0, 0, 0, 0]
    out_pkt = struct.pack('<BBBBBBBBB', 0x02, 0x01, 0x04, 0, 0, 0, 0, 0, 0)
    ser.reset_input_buffer()
    with open('/dev/hidraw0', 'wb') as f:
        f.write(out_pkt)
        f.flush()
    t0 = time.time()
    while time.time() - t0 < 2.0:
        line = ser.readline().decode('utf-8', errors='replace').strip()
        if '[UI] Screen mode: 5: BIG DIGIT' in line:
            hid_out_ok = True
            break
except Exception as e:
    results['hid_out_error'] = str(e)
results['test_hid_out'] = {'pass': hid_out_ok}

# TEST 10: USB HID OUT Command (Report ID 2 -> SetBatteryProfile: 2 [1200mAh])
hid_batt_ok = False
try:
    out_pkt = struct.pack('<BBBBBBBBB', 0x02, 0x06, 0x02, 0, 0, 0, 0, 0, 0)
    ser.reset_input_buffer()
    with open('/dev/hidraw0', 'wb') as f:
        f.write(out_pkt)
        f.flush()
    t0 = time.time()
    while time.time() - t0 < 2.0:
        line = ser.readline().decode('utf-8', errors='replace').strip()
        if '[HID-CMD] Battery profile set to: 1200mAh' in line:
            hid_batt_ok = True
            break
except Exception as e:
    results['hid_batt_error'] = str(e)
results['test_hid_batt'] = {'pass': hid_batt_ok}

# TEST 11: RTC Clock Read via UART Console ('c')
ok, lines = send_and_expect('c', ['[RTC] Current Time:', 'Epoch:'], timeout=1.5)
results['test_rtc_check'] = {'pass': ok, 'lines': lines}

# TEST 12: USB HID OUT Command (Report ID 2 -> SetEpoch: Opcode 0x07, 1760091240)
hid_rtc_ok = False
try:
    sync_epoch = 1760091240
    out_pkt = struct.pack('<BBI3s', 0x02, 0x07, sync_epoch, b'\x00\x00\x00')
    ser.reset_input_buffer()
    with open('/dev/hidraw0', 'wb') as f:
        f.write(out_pkt)
        f.flush()
    t0 = time.time()
    while time.time() - t0 < 2.0:
        line = ser.readline().decode('utf-8', errors='replace').strip()
        if '[HID-CMD] Synchronized RTC epoch to: 1760091240' in line:
            hid_rtc_ok = True
            break
except Exception as e:
    results['hid_rtc_error'] = str(e)
results['test_hid_epoch'] = {'pass': hid_rtc_ok}

# TEST 13: Over-Current Alert Threshold via UART Console ('l' and 'L 1500\n')
ok_l1, lines_l1 = send_and_expect('l', ['[LIMIT] Over-Current Alert Limit:'], timeout=1.5)
ok_l2, lines_l2 = send_and_expect('L 1500\n', ['[LIMIT] Set Over-Current Alert Limit to: 1500 mA'], timeout=2.0, char_delay=0.01)
results['test_limit_uart'] = {'pass': ok_l1 and ok_l2, 'lines': lines_l1 + lines_l2}

# TEST 14: USB HID OUT SetCurrentLimit (Opcode 0x08 -> 2500 mA)
hid_lim_ok = False
try:
    lim_ma = 2500
    out_pkt = struct.pack('<BBH5s', 0x02, 0x08, lim_ma, b'\x00\x00\x00\x00\x00')
    ser.reset_input_buffer()
    with open('/dev/hidraw0', 'wb') as f:
        f.write(out_pkt)
        f.flush()
    t0 = time.time()
    while time.time() - t0 < 2.0:
        line = ser.readline().decode('utf-8', errors='replace').strip()
        if '[HID-CMD] Set Over-Current Alert Limit to: 2500 mA' in line:
            hid_lim_ok = True
            break
except Exception as e:
    results['hid_lim_error'] = str(e)
results['test_hid_limit'] = {'pass': hid_lim_ok}

# TEST 15: Telemetry Summary JSON Verification for limit_ma and alert
ok_s, lines_s = send_and_expect('s', ['--- SESSION TELEMETRY SUMMARY ---', '[JSON]'], timeout=2.5)
json_has_limit = False
limit_val = None
for l in lines_s:
    if '[JSON]' in l:
        idx = l.find('[JSON] ')
        if idx != -1:
            try:
                d = json.loads(l[idx+7:])
                if 'limit_ma' in d and 'alert' in d:
                    json_has_limit = True
                    limit_val = d['limit_ma']
            except:
                pass
results['test_json_limit'] = {'pass': ok_s and json_has_limit and (limit_val == 2500), 'limit_ma': limit_val, 'lines': lines_s}

ser.close()
print("FINAL_RESULTS:" + json.dumps(results))
"""

def main():
    print("=" * 70)
    print(" Running Build 00C8 Remote Command & Telemetry Validation Test")
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
    print(" EMPIRICAL TEST RESULTS (BUILD 00C8)")
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

    # Test 3: Battery Capacity Profile Cycling
    bat_res = res.get('test_battery', {})
    bp = bat_res.get('pass', False)
    all_passed &= bp
    print(f" [TEST 3] Battery Profile Cycling ('b'): {'PASS' if bp else 'FAIL'}")

    # Test 4: Telemetry Dump
    dump = res.get('test_dump', {})
    dp = dump.get('pass', False)
    all_passed &= dp
    print(f" [TEST 4] Session Dump ('s') & JSON parse: {'PASS' if dp else 'FAIL'}")
    if dp and dump.get('json'):
        print(f"          JSON Payload: {json.dumps(dump['json'])}")

    # Test 5: Force SD Flush
    f_res = res.get('test_sd_flush', {})
    fp = f_res.get('pass', False)
    all_passed &= fp
    print(f" [TEST 5] Force SD Flush ('f'): {'PASS' if fp else 'FAIL'}")

    # Test 6: SD Rotation
    r_res = res.get('test_sd_rotate', {})
    rp = r_res.get('pass', False)
    all_passed &= rp
    print(f" [TEST 6] SD File Rotation ('r'): {'PASS' if rp else 'FAIL'} (Active: {r_res.get('new_file')})")

    # Test 7: Tare Reset
    t_res = res.get('test_tare_reset', {})
    tp = t_res.get('pass', False)
    all_passed &= tp
    print(f" [TEST 7] Zero-Tare & Reset ('t'): {'PASS' if tp else 'FAIL'}")

    # Test 8: HID Input Report
    hid_in = res.get('test_hid_in', {})
    hip = hid_in.get('pass', False)
    all_passed &= hip
    print(f" [TEST 8] USB HID Telemetry (Report ID 1): {'PASS' if hip else 'FAIL'}")
    if hip and hid_in.get('reading'):
        print(f"          Reading: {hid_in['reading']}")

    # Test 9: HID Output Report (SetMode 4: BigDigit)
    hid_out = res.get('test_hid_out', {})
    hop = hid_out.get('pass', False)
    all_passed &= hop
    print(f" [TEST 9] USB HID OUT Command (SetMode: 4 [BIG DIGIT]): {'PASS' if hop else 'FAIL'}")

    # Test 10: HID Output Report (SetBatteryProfile 2: 1200mAh)
    hid_batt = res.get('test_hid_batt', {})
    hbp = hid_batt.get('pass', False)
    all_passed &= hbp
    print(f" [TEST 10] USB HID OUT Command (SetBatteryProfile: 2 [1200mAh]): {'PASS' if hbp else 'FAIL'}")

    # Test 11: RTC Status Check ('c')
    rtc_res = res.get('test_rtc_check', {})
    rcp = rtc_res.get('pass', False)
    all_passed &= rcp
    print(f" [TEST 11] RTC Clock Check via UART ('c'): {'PASS' if rcp else 'FAIL'}")
    if rcp and rtc_res.get('lines'):
        for l in rtc_res['lines']:
            if '[RTC]' in l:
                print(f"          {l}")

    # Test 12: HID SetEpoch (Opcode 0x07)
    epoch_res = res.get('test_hid_epoch', {})
    ep = epoch_res.get('pass', False)
    all_passed &= ep
    print(f" [TEST 12] USB HID OUT SetEpoch (Opcode 0x07 -> 1760091240): {'PASS' if ep else 'FAIL'}")

    # Test 13: Over-Current Alert Threshold via UART Console ('l', 'L 1500')
    lim_u = res.get('test_limit_uart', {})
    lup = lim_u.get('pass', False)
    all_passed &= lup
    print(f" [TEST 13] Over-Current Alert Threshold via UART ('l', 'L 1500'): {'PASS' if lup else 'FAIL'}")

    # Test 14: USB HID OUT SetCurrentLimit (Opcode 0x08 -> 2500 mA)
    lim_h = res.get('test_hid_limit', {})
    lhp = lim_h.get('pass', False)
    all_passed &= lhp
    print(f" [TEST 14] USB HID OUT SetCurrentLimit (Opcode 0x08 -> 2500 mA): {'PASS' if lhp else 'FAIL'}")

    # Test 15: Telemetry Summary JSON Validation for limit_ma and alert
    lim_j = res.get('test_json_limit', {})
    ljp = lim_j.get('pass', False)
    all_passed &= ljp
    print(f" [TEST 15] JSON Telemetry Alert & Limit Field Validation: {'PASS' if ljp else 'FAIL'} (Limit: {lim_j.get('limit_ma')} mA)")

    print("=" * 70)
    if all_passed:
        print(" >>> ALL 15 TEST SUITES PASSED CLEANLY ON TARGET HARDWARE <<<")
        print("=" * 70)
        sys.exit(0)
    else:
        print(" >>> SOME TESTS FAILED - DIAGNOSTICS REQUIRED <<<")
        print(f" Raw Results: {json.dumps(res, indent=2)}")
        print("=" * 70)
        sys.exit(1)

if __name__ == "__main__":
    main()
