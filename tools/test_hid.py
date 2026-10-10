import paramiko

ssh = paramiko.SSHClient()
ssh.set_missing_host_key_policy(paramiko.AutoAddPolicy())
ssh.connect('192.168.0.63', username='arun', password='arun', timeout=10)

cmd = "python3 -c \"import struct; f=open('/dev/hidraw0', 'rb'); d=f.read(9); f.close(); rid, v, i, p, flg, seq = struct.unpack('<BHhHBB', d); print(f'ReportID: {rid}, V: {v} mV, I: {i/10:.1f} mA, P: {p} mW, Flags: 0x{flg:02X}, Seq: {seq}')\""
stdin, stdout, stderr = ssh.exec_command(cmd)
out = stdout.read().decode().strip()
err = stderr.read().decode().strip()
print('HID REPORT:', out)
if err:
    print('STDERR:', err)
ssh.close()
