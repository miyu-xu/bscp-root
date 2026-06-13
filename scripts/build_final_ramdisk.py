#!/usr/bin/env python3
"""Build final ramdisk: metadata->meta_bak, first_stage_mount->nofail+padded."""
import subprocess, os, shutil

WORK = '/tmp/build_rd2'
if os.path.exists(WORK):
    shutil.rmtree(WORK)
os.makedirs(WORK)
os.chdir(WORK)

subprocess.run(
    'gunzip -c /mnt/c/workspace/bscp/bscp/out/dist/img/android_ramdisk.gz 2>/dev/null | cpio -idm',
    shell=True, check=True)

old_suffix = 'first_stage_mount,check,metadata_csum'
new_suffix = 'nofail,nodiratime,noexec,nodev,nosuid'
print(f"Suffix: {len(old_suffix)} -> {len(new_suffix)} chars (match={len(old_suffix)==len(new_suffix)})")

old = ('/dev/block/by-name/metadata /metadata ext4 nodev,noatime,nosuid,'
       'errors=panic,data=journal,commit=1 wait,formattable,'
       + old_suffix)
new = ('/dev/block/by-name/meta_bak /metadata ext4 nodev,noatime,nosuid,'
       'errors=panic,data=journal,commit=1 wait,formattable,'
       + new_suffix)

print(f"Old ({len(old)}): {old}")
print(f"New ({len(new)}): {new}")
assert len(old) == len(new), f"MISMATCH: {len(old)} vs {len(new)}"

for fname in ['fstab.cutf_cvm', 'first_stage_ramdisk/fstab.cutf_cvm']:
    with open(fname, 'r') as f:
        content = f.read()
    assert old in content, f"Pattern not found in {fname}"
    content = content.replace(old, new)
    with open(fname, 'w') as f:
        f.write(content)

with open('fstab.cutf_cvm') as f:
    for i, line in enumerate(f):
        if 'meta_bak' in line:
            print(f"Line {i+1}: {line.rstrip()}")
    sz = os.path.getsize('fstab.cutf_cvm')
    print(f"Size: {sz} bytes (expected 1355)")

out = '/mnt/c/workspace/bscp/bscp/out/dist/img/android_ramdisk_final2.gz'
subprocess.run(
    'find . -print0 | cpio -o0 -H newc --owner=root:root 2>/dev/null | gzip -1 > ' + out,
    shell=True, check=True)
print(f"Ramdisk: {os.path.getsize(out)} bytes")
print("DONE!")
