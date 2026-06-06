#!/bin/bash
# Fix ChromeOS SYSLINUX configs on EFI partition (WSL root required).
set -euo pipefail
REPO="/mnt/c/workspace/bscp/bscp"
IMG="${1:-$REPO/out/dist/img/amd64-generic_test_image_boot.bin}"
EFI_OFFSET=228589568
MNT="${2:-/tmp/chromeos-efi-mnt}"
CFG_DIR="$REPO/out/dist/logs/efi-mount"

mkdir -p "$MNT"
mount -o loop,offset=$EFI_OFFSET,rw "$IMG" "$MNT"

cp "$CFG_DIR/default.cfg" "$MNT/syslinux/default.cfg"
cp "$CFG_DIR/usb.A.cfg" "$MNT/syslinux/usb.A.cfg"
cp "$CFG_DIR/root.A.cfg" "$MNT/syslinux/root.A.cfg"

# Remove cached config if present (BIOS path); EFI reads .cfg directly.
rm -f "$MNT/syslinux/ldlinux.bss" 2>/dev/null || true

sync
umount "$MNT"
echo "Patched syslinux on $IMG"
echo "  DEFAULT=$(cat "$CFG_DIR/default.cfg")"
