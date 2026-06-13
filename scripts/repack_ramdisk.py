#!/usr/bin/env python3
"""Repack ramdisk with init.cutf_cvm.rc fix."""
import os, gzip, stat, sys

def create_cpio(dirpath, output_path):
    entries = []
    for root, dirs, files in os.walk(dirpath):
        for name in dirs + files:
            fullpath = os.path.join(root, name)
            relpath = os.path.relpath(fullpath, dirpath).replace('\\', '/')
            if relpath == '.':
                continue

            st = os.lstat(fullpath)
            mode = st.st_mode
            size = st.st_size if stat.S_ISREG(mode) else 0
            nlink = st.st_nlink

            header = f'070701{int(mode):08X}{0:08X}{0:08X}{nlink:08X}0{0:08X}{size:08X}{0:08X}{0:08X}{0:08X}{0:08X}{len(relpath)+1:08X}{0:08X}'.encode()
            name_bytes = relpath.encode() + b'\x00'
            padding = (4 - (len(header) + len(name_bytes)) % 4) % 4

            entry = header + name_bytes + b'\x00' * padding
            entries.append((entry, fullpath if stat.S_ISREG(mode) else None, size))

    # TRAILER!!!
    trailer = b'07070100000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000b00000000TRAILER!!!\x00\x00\x00\x00'
    entries.append((trailer, None, 0))

    with open(output_path, 'wb') as f:
        for header, filepath, size in entries:
            f.write(header)
            if filepath and size > 0:
                try:
                    with open(filepath, 'rb') as fin:
                        data = fin.read()
                        f.write(data)
                        pad = (4 - size % 4) % 4
                        if pad:
                            f.write(b'\x00' * pad)
                except OSError:
                    # Skip symlinks and unreadable files
                    pass

def main():
    work = 'C:/workspace/bscp/bscp/out/dist/img/ramdisk_fix_work'

    # Verify files exist
    init_rc = os.path.join(work, 'init.cutf_cvm.rc')
    fix_sh = os.path.join(work, 'vendor_gpu_fix.sh')
    for f in [init_rc, fix_sh]:
        if os.path.exists(f):
            print(f'OK: {f} ({os.path.getsize(f)} bytes)')
        else:
            print(f'MISSING: {f}')
            sys.exit(1)

    cpio_path = os.path.join(work, 'ramdisk_new.cpio')
    create_cpio(work, cpio_path)
    print(f'Cpio: {os.path.getsize(cpio_path)} bytes')

    with open(cpio_path, 'rb') as f:
        data = f.read()

    out_path = 'C:/workspace/bscp/bscp/out/dist/img/ramdisk_gpufix3.gz'
    with gzip.open(out_path, 'wb', compresslevel=1) as f:
        f.write(data)
    print(f'Gzip: {os.path.getsize(out_path)} bytes')

    # Verify
    for sig in [b'init.cutf_cvm.rc', b'vendor_gpu_fix.sh']:
        idx = data.find(sig)
        print(f'{sig.decode()}: {"FOUND" if idx >= 0 else "MISSING"}')

if __name__ == '__main__':
    main()
