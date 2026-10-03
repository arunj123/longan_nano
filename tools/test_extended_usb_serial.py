import paramiko

client = paramiko.SSHClient()
client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
client.connect("192.168.0.63", username="arun", password="arun")

remote_code = """
import serial, time
s = serial.Serial('/dev/ttyACM1', 115200, timeout=2)

test_strings = [
    b'Ping from Linux host!',
    b'The quick brown fox jumps over the lazy dog 1234567890',
    b'Embedded Rust on Longan Nano GD32VF103 RV32IMAC @ 96 MHz works flawlessly!'
]

for idx, msg in enumerate(test_strings):
    s.write(msg)
    time.sleep(0.05)
    echo = s.read(len(msg))
    status = 'PASS' if echo == msg else 'FAIL'
    print(f'Test {idx+1}: {status} (sent {len(msg)} B, got {len(echo)} B)')
    if echo != msg:
        print(f'  Expected: {msg}')
        print(f'  Got:      {echo}')
"""

stdin, stdout, stderr = client.exec_command(f'python3 -c "{remote_code}"')
print("STDOUT:\n", stdout.read().decode())
print("STDERR:\n", stderr.read().decode())
client.close()
