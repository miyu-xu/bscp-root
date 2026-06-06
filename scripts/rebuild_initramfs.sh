#!/bin/bash
set -e
cd /tmp
rm -rf ifs && mkdir ifs && cd ifs
gzip -dc /mnt/c/workspace/bscp/bscp/out/dist/img/initramfs.cpio.gz | cpio -idum 2>/dev/null

# Fix 1: mkdir -p /copy before file creation (line 62)
sed -i '62i mkdir -p /copy' init

# Fix 2: disable cp overwrite
sed -i 's/^cp \/ui.conf \/copy\/ui.conf/#DISABLED: cp \/ui.conf \/copy\/ui.conf/' init

# Fix 3: Replace the modetest PID approach with killall
sed -i 's/MODE_PID=$!/MODE_PID=$!\n  sleep 3\n  echo "===UI: releasing DRM master (killall)..." > \/dev\/kmsg\n  killall modetest 2>\/dev\/null || true\n  sleep 1\n  echo "===UI: DRM released, launching session_manager ===" > \/dev\/kmsg/' init

# Remove the old kill line
sed -i '/kill.*MODE_PID.*release DRM master/d' init

# Fix 4: Add Chrome auto-login config before chroot
python3 << 'PYEOF'
c = open("/tmp/ifs/init").read()
insert = """# Create Chrome dev config for auto-login
echo "initramfs: creating Chrome auto-login config..."
cat > /chrome_dev.conf << "CHEOF"
--enable-autologin
--login-manager
--login-user=testuser
--login-profile=user
--disable-boot-animation
--no-sandbox
--disable-gpu-sandbox
CHEOF
mount_root --bind /chrome_dev.conf /newroot/etc/chrome_dev.conf 2>/dev/null || true
echo "initramfs: chrome_dev.conf bind-mounted"
"""
c = c.replace("exec /bin/chroot /newroot /sbin/init", insert + "\nexec /bin/chroot /newroot /sbin/init", 1)
open("/tmp/ifs/init", "w").write(c)
print("Auto-login added")
PYEOF

# Verify
echo "=== Verify ==="
grep -n "chrome_dev\|auto-login\|CHEOF\|killall\|DISABLED.*cp\|mkdir.*copy" init | head -15

# Repack
find . -print0 | cpio --null -o -H newc --quiet > /tmp/initrd.img 2>/dev/null
echo "cpio: $(ls -la /tmp/initrd.img | awk '{print $5}') bytes"
gzip -9 -c /tmp/initrd.img > /mnt/c/workspace/bscp/bscp/out/dist/img/initramfs.cpio.gz
ls -la /mnt/c/workspace/bscp/bscp/out/dist/img/initramfs.cpio.gz
echo "DONE"
