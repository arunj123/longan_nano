import struct

import sys
pcap_path = sys.argv[1] if len(sys.argv) > 1 else 'build/prj_usb_msc/msc_fresh.pcap'
with open(pcap_path, 'rb') as f:
    hdr = f.read(24)
    magic, ver_maj, ver_min, thiszone, sigfigs, snaplen, linktype = struct.unpack('<IHHiIII', hdr)
    print(f'Magic: {magic:#x}, LinkType: {linktype}')
    
    pkt_idx = 0
    while True:
        pkt_hdr = f.read(16)
        if len(pkt_hdr) < 16:
            break
        sec, usec, caplen, origlen = struct.unpack('<IIII', pkt_hdr)
        data = f.read(caplen)
        pkt_idx += 1
        
        hdr_len = 64 if linktype == 220 else 48
        if caplen >= hdr_len:
            urb_id, ptype, xfer_type, epnum, devnum = struct.unpack('<QBBBB', data[:12])
            status, len_urb, len_cap = struct.unpack('<iii', data[28:40])
            payload = data[hdr_len:hdr_len+len_cap]
            ptype_char = chr(ptype)
            
            # Print any bulk endpoint transfers
            if (epnum & 0x7F) in (1, 2) and devnum != 0:
                desc = ""
                if b'USBC' in payload:
                    pos = payload.find(b'USBC')
                    cbw = payload[pos:pos+31]
                    sig, tag, xfer_len, flags, lun, cdb_len = struct.unpack('<IIIBBB', cbw[:15])
                    cdb = cbw[15:15+cdb_len]
                    desc = f"--> CBW Tag={tag:#x} XferLen={xfer_len} CDB={cdb.hex()}"
                elif b'USBS' in payload:
                    pos = payload.find(b'USBS')
                    csw = payload[pos:pos+13]
                    sig, tag, residue, status_b = struct.unpack('<IIIB', csw[:13])
                    desc = f"<-- CSW Tag={tag:#x} Residue={residue} Status={status_b}"
                elif ptype_char == 'C' and (epnum & 0x80):
                    desc = f"<-- DATA_IN ({len_urb} bytes, stat={status}): {payload[:16].hex()} ... {payload[-16:].hex() if len(payload)>=16 else ''}"
                elif ptype_char == 'S' and (epnum & 0x80):
                    desc = f"    SUBMIT BULK IN (req {len_urb} bytes)"
                elif ptype_char == 'S' and not (epnum & 0x80):
                    desc = f"    SUBMIT BULK OUT (len {len_urb} bytes)"
                
                print(f"[{pkt_idx:3d}] dev={devnum} ep={epnum:#04x} {ptype_char} urb_len={len_urb:4d} stat={status:4d} {desc}")
