import paramiko

HOST = '192.168.0.63'
client = paramiko.SSHClient()
client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
client.connect(HOST, username='arun', password='arun', timeout=10)

stdin, stdout, stderr = client.exec_command('ps aux | grep -E "(usbmon|sg_|python)" | grep -v grep')
print('=== Running processes ===\n' + stdout.read().decode())

stdin, stdout, stderr = client.exec_command('echo arun | sudo -S dmesg | tail -n 25')
print('=== dmesg tail ===\n' + stdout.read().decode())

stdin, stdout, stderr = client.exec_command('lsblk')
print('=== lsblk ===\n' + stdout.read().decode())

client.close()
