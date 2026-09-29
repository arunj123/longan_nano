import paramiko

client = paramiko.SSHClient()
client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
client.connect('192.168.0.63', username='arun', password='arun', allow_agent=False, look_for_keys=False, timeout=15)
cmd = 'openocd -f /home/arun/longan_nano_tools/openocd-sipeed-libusb.cfg -c "init; halt; dump_image /tmp/media_buf.bin 0x20000610 512; resume; shutdown"'
stdin, stdout, stderr = client.exec_command(cmd)
stdout.read()
sftp = client.open_sftp()
sftp.get('/tmp/media_buf.bin', 'build/prj_usb_msc/media_buf.bin')
sftp.close()
client.close()

data = open('build/prj_usb_msc/media_buf.bin', 'rb').read()
print('DUMP of media_buffer (512 bytes):')
print('Bytes 0..64:', data[:64].hex())
print('Bytes 384..448:', data[384:448].hex())
print('Bytes 448..512:', data[448:512].hex())
