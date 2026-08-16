#!/usr/bin/env python3
"""Convert Android sparse image to raw."""
import struct, os, sys

def write_fill(out, value, count):
    if value == 0:
        out.seek(count, os.SEEK_CUR)
        return
    pattern = struct.pack('<I', value)
    chunk = pattern * (1024 * 1024 // len(pattern))
    while count:
        size = min(count, len(chunk))
        out.write(chunk[:size])
        count -= size


def simg2img(src, dst):
    with open(src, 'rb') as f:
        magic = struct.unpack('<I', f.read(4))[0]
        if magic != 0xED26FF3A:
            print(f'Not sparse image: magic=0x{magic:08X}')
            return False

        ver_major = struct.unpack('<H', f.read(2))[0]
        ver_minor = struct.unpack('<H', f.read(2))[0]
        header_size = struct.unpack('<H', f.read(2))[0]
        chunk_hdr_size = struct.unpack('<H', f.read(2))[0]
        block_size = struct.unpack('<I', f.read(4))[0]
        total_blocks = struct.unpack('<I', f.read(4))[0]
        total_chunks = struct.unpack('<I', f.read(4))[0]
        crc32 = struct.unpack('<I', f.read(4))[0]

        print(f'Sparse: v{ver_major}.{ver_minor}, blocks={total_blocks}, block_size={block_size}, chunks={total_chunks}')
        print(f'Output: {total_blocks * block_size / 1024 / 1024:.1f} MB')

        with open(dst, 'wb') as out:
            out_blocks = 0
            for i in range(total_chunks):
                chunk_type = struct.unpack('<H', f.read(2))[0]
                reserved = struct.unpack('<H', f.read(2))[0]
                chunk_sectors = struct.unpack('<I', f.read(4))[0]
                total_size = struct.unpack('<I', f.read(4))[0]

                # total_size includes the 12-byte chunk header
                data_size = total_size - 12
                if chunk_type == 0xCAC1:  # RAW
                    data = f.read(data_size)
                    out.write(data)
                    out_blocks += chunk_sectors
                elif chunk_type == 0xCAC2:  # FILL
                    fill_val = struct.unpack('<I', f.read(data_size))[0]
                    write_fill(out, fill_val, block_size * chunk_sectors)
                    out_blocks += chunk_sectors
                elif chunk_type == 0xCAC3:  # DONTCARE
                    out.seek(chunk_sectors * block_size, os.SEEK_CUR)
                    out_blocks += chunk_sectors
                elif chunk_type == 0xCAC4:  # CRC32
                    f.read(data_size)

                if i % 20 == 0:
                    pct = out_blocks * 100 // total_blocks
                    print(f'\r  {pct}%', end='', flush=True)

            out.truncate(total_blocks * block_size)
            print(f'\r  100%')

        actual = os.path.getsize(dst)
        expected = total_blocks * block_size
        print(f'Done: {actual} bytes')
        return actual == expected

if __name__ == '__main__':
    src = sys.argv[1] if len(sys.argv) > 1 else 'C:/Users/developer/Desktop/super_sparse.img'
    dst = sys.argv[2] if len(sys.argv) > 2 else 'C:/Users/developer/Desktop/super_ext4_raw.img'
    simg2img(src, dst)
