import subprocess, os

os.chdir('/tmp')
os.system('rm -rf comment_rd && mkdir comment_rd')
os.chdir('/tmp/comment_rd')
subprocess.run('gunzip -c /mnt/c/workspace/bscp/bscp/out/dist/img/android_ramdisk.gz 2>/dev/null | cpio -idm', shell=True)

old = '/dev/block/by-name/metadata /metadata ext4 nodev,noatime,nosuid,errors=panic,data=journal,commit=1 wait,formattable,first_stage_mount,check,metadata_csum'
new = '#' + ' ' * (len(old) - 1)

print(f"Old ({len(old)}): |{old}|")
print(f"New ({len(new)}): |{new}|")

for f in ['fstab.cutf_cvm', 'first_stage_ramdisk/fstab.cutf_cvm']:
    with open(f, 'r') as fh:
        content = fh.read()
    assert old in content, f"OLD not found in {f}"
    content = content.replace(old, new)
    with open(f, 'w') as fh:
        fh.write(content)

# Verify
with open('fstab.cutf_cvm') as f:
    for i, line in enumerate(f):
        if '/metadata' in line and '#' not in line:
            print(f"WARNING: metadata still present at line {i}: {line.strip()}")
    print(f"File size: {os.path.getsize('fstab.cutf_cvm')} bytes")

# Repack
subprocess.run('find . -print0 | cpio -o0 -H newc --owner=root:root 2>/dev/null | gzip -1 > /mnt/c/workspace/bscp/bscp/out/dist/img/android_ramdisk_comment.gz', shell=True)
print(f"Done: {os.path.getsize('/mnt/c/workspace/bscp/bscp/out/dist/img/android_ramdisk_comment.gz')} bytes")
