#!/usr/bin/env python3

import sys
import time

from elftools.elf.elffile import ELFFile

from crc import Calculator, Crc32

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("usage: python3 sendelf.py [executable]")
        sys.exit(1)
    
    with open(sys.argv[1], 'rb') as f:
        elf = ELFFile(f)
        segments = [segment for segment in elf.iter_segments(type='PT_LOAD')]

        header = elf['e_entry'].to_bytes(4, byteorder='little')
        header += len(segments).to_bytes(4, byteorder='little')
        for segment in segments:
            header += segment['p_paddr'].to_bytes(4, byteorder='little')
            header += segment['p_filesz'].to_bytes(4, byteorder='little')
        
        data = b''
        for segment in segments:
            data += segment.data()

        crc = Calculator(Crc32.CRC32)
        header += crc.checksum(data).to_bytes(4, byteorder='little')
        header += crc.checksum(header).to_bytes(4, byteorder='little')
        
        time.sleep(1) # so we can pipe into QEMU
        sys.stdout.buffer.write(header)
        sys.stdout.buffer.write(data)
