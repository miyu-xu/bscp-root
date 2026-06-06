#!/bin/bash
# Patch ChromeOS EFI grub.cfg on a disk image (WSL root).
set -euo pipefail
IMG="${1:?usage: fix_chromeos_grub_wsl.sh <image.bin>}"
EFI_OFFSET=228589568
MNT="${2:-/tmp/chromeos-efi-mnt}"
mkdir -p "$MNT"
mount -o loop,offset=$EFI_OFFSET,rw "$IMG" "$MNT"
GRUB="$MNT/efi/boot/grub.cfg"
cp "$GRUB" "${GRUB}.bak"

CONSOLE='console=ttyS0,115200n8 earlycon=uart8250,io,0x3f8,115200n8'

python3 - "$GRUB" "$CONSOLE" <<'PY'
import re, sys
path, console = sys.argv[1], sys.argv[2]
text = open(path, encoding='utf-8', errors='replace').read()
# Replace empty console= in linux lines
text2 = re.sub(r'console=\s+', f'{console} ', text)
if text2 == text:
    # fallback: insert after noinitrd if console already partially set
    text2 = re.sub(r'(noinitrd)\s+', rf'\1 {console} ', text, count=1)
open(path, 'w', encoding='utf-8', newline='\n').write(text2)
print('--- patched grub.cfg ---')
print(open(path, encoding='utf-8').read())
PY

sync
umount "$MNT"
echo "OK: patched $IMG"
