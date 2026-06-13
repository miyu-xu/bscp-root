#!/bin/bash
# Run on VM as root to add precompiled_sepolicy to vendor and rebuild super
set -e

AOSP=/opt/workspace/aosp
OUT=$AOSP/out/target/product/vsoc_x86_64

echo "=== Step 1: Find precompiled_sepolicy ==="
# Find the precompiled sepolicy in build output
if [ -f "$OUT/obj/ETC/precompiled_sepolicy_intermediates/precompiled_sepolicy" ]; then
    SEPOLICY="$OUT/obj/ETC/precompiled_sepolicy_intermediates/precompiled_sepolicy"
    echo "Found at: $SEPOLICY"
elif [ -f "$OUT/odm/etc/selinux/precompiled_sepolicy" ]; then
    SEPOLICY="$OUT/odm/etc/selinux/precompiled_sepolicy"
    echo "Found at: $SEPOLICY"
elif [ -f "$OUT/obj/ETC/vendor_sepolicy.cil_intermediates/vendor_sepolicy.cil" ]; then
    # Need to compile from CIL - skip for now, find prebuilt
    echo "Need to compile from CIL..."
    find $OUT -name "precompiled_sepolicy" -type f 2>/dev/null
    exit 1
else
    echo "Searching for precompiled_sepolicy..."
    find $OUT -name "precompiled_sepolicy" -type f 2>/dev/null
    exit 1
fi

echo "=== Step 2: Mount vendor.img and add precompiled_sepolicy ==="
# Clean intermediates so rebuild picks up changes
rm -rf $OUT/obj/PACKAGING/vendor_intermediates
rm -rf $OUT/obj/PACKAGING/super_intermediates

# Mount vendor
mkdir -p /tmp/vendor_mnt
mount -t ext4 -o loop,rw $OUT/vendor.img /tmp/vendor_mnt 2>/dev/null || {
    echo "Mount failed, using debugfs..."
    # Use debugfs to add file
    cd /tmp
    rm -f vendor_cmds.txt
    echo "cd /etc/selinux" > vendor_cmds.txt
    echo "write $SEPOLICY precompiled_sepolicy" >> vendor_cmds.txt
    debugfs -w -f vendor_cmds.txt $OUT/vendor.img 2>&1
    echo "Added precompiled_sepolicy via debugfs"
    # Add companion files if they exist
    for f in $(dirname $SEPOLICY)/precompiled_sepolicy*; do
        if [ -f "$f" ] && [ "$(basename $f)" != "precompiled_sepolicy" ]; then
            echo "cd /etc/selinux" > vendor_cmds.txt
            echo "write $f $(basename $f)" >> vendor_cmds.txt
            debugfs -w -f vendor_cmds.txt $OUT/vendor.img 2>&1
        fi
    done
}

echo "=== Step 3: Rebuild super.img ==="
cd $AOSP
source build/envsetup.sh 2>/dev/null
lunch aosp_cf_x86_64_phone-trunk_staging-userdebug 2>/dev/null
rm -f $OUT/super.img $OUT/super_empty.img
m superimage 2>&1

echo "=== Step 4: Verify ==="
if [ -f "$OUT/super.img" ]; then
    echo "SUCCESS: super.img rebuilt at $OUT/super.img"
    echo "Size: $(ls -lh $OUT/super.img | awk '{print $5}')"
    echo "Copying to Desktop..."
    cp $OUT/super.img /media/sf_Desktop/super_v9.img
    echo "DONE - super_v9.img copied to Desktop"
else
    echo "FAILED: super.img not found"
    exit 1
fi
