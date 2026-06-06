#!/usr/bin/env bash
# Mount ChromeOS EFI-SYSTEM (FAT) from a raw disk image in WSL and optionally edit syslinux cfg.
set -euo pipefail
REPO="/mnt/c/workspace/bscp/bscp"
IMG="${1:-$REPO/out/dist/img/amd64-generic_test_image_vhd.bin}"
MNT="${2:-$REPO/out/dist/logs/efi-mount/mnt}"
EFI_LBA=446464   # partition 12 start (512-byte sectors)
OFFSET=$((EFI_LBA * 512))

if [[ ! -f "$IMG" ]]; then
  echo "Image not found: $IMG" >&2
  exit 1
fi

mkdir -p "$MNT"
if mountpoint -q "$MNT"; then
  echo "Already mounted at $MNT"
else
  echo "Mounting EFI at offset $OFFSET from $IMG"
  sudo mount -o loop,offset="$OFFSET",rw "$IMG" "$MNT"
fi

echo "EFI contents:"
ls -la "$MNT"
echo ""
if [[ -d "$MNT/syslinux" ]]; then
  echo "syslinux/:"
  ls -la "$MNT/syslinux"
  echo ""
  for f in "$MNT/syslinux"/*.cfg; do
    echo "=== $(basename "$f") ==="
    head -40 "$f"
    echo ""
  done
fi
