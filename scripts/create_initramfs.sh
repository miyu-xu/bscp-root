#!/bin/bash
set -e

rm -rf /tmp/initramfs
mkdir -p /tmp/initramfs/{bin,proc,sys,dev,newroot}

BUSYBOX=/mnt/c/Users/developer/AppData/Local/Temp/busybox
MOUNT_ROOT=/mnt/c/Users/developer/AppData/Local/Temp/mount_root

# Copy busybox and create copies for each command
cp "$BUSYBOX" /tmp/initramfs/bin/busybox
chmod 755 /tmp/initramfs/bin/busybox
# Add static mount_root binary
cp "$MOUNT_ROOT" /tmp/initramfs/bin/mount_root
chmod 755 /tmp/initramfs/bin/mount_root

cd /tmp/initramfs/bin
for cmd in sh mount umount sed grep cat echo ls switch_root sleep head cp mkdir chroot chmod; do
    cp busybox "$cmd"
done
echo "bin files: $(ls | wc -l)"

cat > /tmp/initramfs/init << 'INITEOF'
#!/bin/sh
echo "initramfs: start"
/bin/mount -t devtmpfs none /dev
/bin/mount -t proc none /proc
/bin/mount -t sysfs none /sys
echo "initramfs: waiting for vda3..."
i=0
while [ $i -lt 30 ]; do
    [ -b /dev/vda3 ] && { echo "initramfs: vda3 ready ${i}s"; break; }
    i=$((i+1))
    /bin/sleep 1
done
[ ! -b /dev/vda3 ] && { echo "initramfs: vda3 not found!"; while true; do /bin/sleep 60; done; }

# Mount root RO (kernel can't mount RW due to ext4 metadata_csum features)
# We'll use bind mounts to overlay modified config files on the RO rootfs
echo "initramfs: mounting root RO..."
/bin/mount -t ext4 -o ro /dev/vda3 /newroot
rc=$?
echo "initramfs: mount rc=$rc"
[ $rc -ne 0 ] && { echo "mount failed!"; while true; do /bin/sleep 60; done; }

# Initramfs root is already writable (it's tmpfs). Create modified configs directly.
echo "initramfs: preparing patched configs..."

# Create modified ui.conf (remove lockbox-cache dependency)
/bin/sed 's/and started lockbox-cache//' /newroot/etc/init/ui.conf > /ui.conf
echo "initramfs: ui.conf rc=$?"

# Create modified lockbox-cache.conf
if [ -f /newroot/etc/init/lockbox-cache.conf ]; then
    /bin/sed 's/pre-start exec lockbox-cache-manager/pre-start exec lockbox-cache-manager || true/' /newroot/etc/init/lockbox-cache.conf > /lockbox-cache.conf
    echo "initramfs: lc.conf rc=$?"
fi

# Fix cros_configfs to always succeed (original fails with exit 32 in VMs)
cat > /cros_configfs.conf << 'CFSEOF'
description "Abstract job to control the /run/chromeos-config mount"
author "crosvm VM fix"
start on starting udev
task
script
  logger -t cros_configfs "crosvm VM: using no-op"
end script
CFSEOF

# Create boot-splash that starts frecon to probe DRM connectors
# (Frecon sets mode on DRM → connectors become "connected" → Chrome renders)
cat > /boot-splash.conf << 'BSEOF'
description "Displays splash/animation while booting"
author "crosvm VM fix"
oom score -100
export fork
start on stopped udev-trigger-early and started cros_configfs
task
script
  logger -t boot-splash "crosvm: starting frecon in assetless mode"
  /sbin/frecon --daemon --clear 0xfffefefe --enable-osc --enable-vts --pre-create-vts --dev-mode
  logger -t boot-splash "crosvm: frecon done rc=$?"
end script
BSEOF
# Create a simplified ui.conf that runs session_manager directly (no minijail)
cat > /copy/ui.conf << 'UICONFEOF'
description "Chrome OS user interface - crosvm simplified"
author "chromium-os-dev@chromium.org"
start on started boot-services and stopped cgroups and started lockbox-cache
stop on starting pre-shutdown
oom score -100
limit nice 40 40
limit rtprio 10 10
kill timeout 20
expect fork
env UI_LOG_DIR=/var/log/ui
env UI_LOG_FILE=ui.LATEST
env CHROME_COMMAND_FLAG
env UI_FREEZER_CGROUP_DIR=/sys/fs/cgroup/freezer/ui
env CHROME_FREEZER_CGROUP_DIR=/sys/fs/cgroup/freezer/ui/chrome_renderers
env UI_CPU_CGROUP_DIR=/sys/fs/cgroup/cpu/ui
tmpfiles /usr/lib/tmpfiles.d/chromeos.conf
pre-start exec /usr/share/cros/init/ui-pre-start
script
  # Run modetest to set display mode (vhost-user GPU is now ready)
  echo "===UI: running modetest to set mode ===" > /dev/kmsg
  /usr/bin/modetest -s 34:1024x768 2>&1 | while read line; do echo "modetest: $line" > /dev/kmsg; done &
  MODE_PID=$!
  sleep 3
  echo "===UI: modetest PID=$MODE_PID, launching session_manager ===" > /dev/kmsg
  exec /sbin/session_manager ${CHROME_COMMAND_FLAG}
end script
post-start script
  echo $(status | cut -f 4 -d ' ') > "${UI_CPU_CGROUP_DIR}/tasks" 2>/dev/null || true
  echo $(status | cut -f 4 -d ' ') > "${UI_FREEZER_CGROUP_DIR}/cgroup.procs" 2>/dev/null || true
end script
post-stop exec /usr/share/cros/init/ui-post-stop
UICONFEOF
echo "initramfs: ui.conf created rc=$?"
# Note: /copy/ui.conf is already cp'd below in the generic cp section

# Replace trunksd with a no-op (TPM not fully supported yet)
cat > /trunksd.conf << 'TRUNKSEOF'
# No-op trunksd: TPM not fully supported by crosvm minimal backend
start on started boot-services and stopped cr50-result and started dbus
stop on hwsec-stop-low-level-tpm-daemon-signal
task
script
  logger -t trunksd "crosvm minimal TPM - trunks disabled"
end script
TRUNKSEOF
echo "initramfs: trunksd.conf replaced"

# Create a simplified ui-pre-start that skips problematic checks
cat > /ui-pre-start << 'UIPREOF'
#!/bin/sh
# Simplified ui-pre-start for crosvm VM
JOB=$(basename "$0")
UI_LOG_DIR=/var/log/ui
UI_LOG_FILE=ui.LATEST
UI_CPU_CGROUP_DIR=/sys/fs/cgroup/cpu/ui
UI_FREEZER_CGROUP_DIR=/sys/fs/cgroup/freezer/ui
CHROME_FREEZER_CGROUP_DIR=/sys/fs/cgroup/freezer/ui/chrome_renderers
mkdir -p "${UI_LOG_DIR:-}"
ln -sf ui."$(date --utc +%Y%m%d-%H%M%S)" "${UI_LOG_DIR}/${UI_LOG_FILE:-}"
SUPPORT_DIR="/var/spool/support"
mkdir -p "${SUPPORT_DIR}"
chown -R chronos "${SUPPORT_DIR}" 2>/dev/null || true
if [ ! -d "${UI_CPU_CGROUP_DIR:-}" ]; then
  mkdir -p "${UI_CPU_CGROUP_DIR}"
  chmod -R g+w "${UI_CPU_CGROUP_DIR}" 2>/dev/null || true
fi
if [ ! -d "${UI_FREEZER_CGROUP_DIR:-}" ]; then
  mkdir -p "${UI_FREEZER_CGROUP_DIR}"
fi
if [ ! -d "${CHROME_FREEZER_CGROUP_DIR:-}" ]; then
  mkdir -p "${CHROME_FREEZER_CGROUP_DIR}"
  mkdir -p "${CHROME_FREEZER_CGROUP_DIR}/to_be_frozen"
fi
logger -t "${JOB}" "crosvm simplified ui-pre-start completed"
exit 0
UIPREOF
chmod 755 /ui-pre-start
echo "initramfs: ui-pre-start created"

# Bind mount modified files over RO rootfs.
# Use old_root=/initramfs so switch_root preserves our tmpfs mounts.
/bin/mkdir -p /newroot/initramfs 2>/dev/null
mkdir -p /copy
cp /ui.conf /copy/ui.conf
cp /lockbox-cache.conf /copy/lockbox-cache.conf 2>/dev/null
cp /cros_configfs.conf /copy/cros_configfs.conf
cp /boot-splash.conf /copy/boot-splash.conf
cp /trunksd.conf /copy/trunksd.conf
cp /ui-pre-start /copy/ui-pre-start
# Ensure all copied files are executable (for scripts)
chmod 755 /copy/ui-pre-start /copy/ui.conf /copy/boot-splash.conf /copy/trunksd.conf /copy/cros_configfs.conf 2>/dev/null
echo "initramfs: copied to /copy rc=$?"

# Bind mount from /copy (initramfs) - survives chroot
mount_root --bind /copy/ui.conf /newroot/etc/init/ui.conf
mount_root --bind /copy/lockbox-cache.conf /newroot/etc/init/lockbox-cache.conf 2>/dev/null
mount_root --bind /copy/cros_configfs.conf /newroot/etc/init/cros_configfs.conf
mount_root --bind /copy/boot-splash.conf /newroot/etc/init/boot-splash.conf
mount_root --bind /copy/trunksd.conf /newroot/etc/init/trunksd.conf
mount_root --bind /copy/ui-pre-start /newroot/usr/share/cros/init/ui-pre-start
echo "initramfs: bind mounts done"

# Verify the ui-pre-start file before chrooting
echo "initramfs: target perms:"
ls -la /newroot/usr/share/cros/init/ui-pre-start 2>&1
echo "initramfs: target head:"
head -1 /newroot/usr/share/cros/init/ui-pre-start 2>&1

# Debug: check DRM in /dev/dri and sysfs
echo "initramfs: /dev/dri:"
ls -la /dev/dri/ 2>&1
echo "initramfs: card0 nodes:"
ls -la /dev/dri/card* /dev/dri/render* 2>&1
# TEST: Check all DRM connectors
echo "initramfs: DRM test..."
/bin/chroot /newroot /bin/sh -c '
echo "=== DRM sysfs ==="
ls -la /sys/class/drm/ 2>&1
echo "=== DRM connector names ==="
for d in /sys/class/drm/card0-*; do
    [ -d "$d" ] && echo "Found: $(basename $d)"
    [ -f "$d/status" ] && echo "  status: $(cat $d/status 2>&1)"
    [ -f "$d/modes" ] && echo "  modes: $(cat $d/modes 2>&1)"
    [ -f "$d/enabled" ] && echo "  enabled: $(cat $d/enabled 2>&1)"
done
echo "=== /dev/dri ==="
ls -la /dev/dri/ 2>&1
echo "=== DONE ==="
' 2>&1

# CRITICAL: Mount /dev, /sys, /proc into /newroot so chroot'd processes see DRM and sysfs
echo "initramfs: bind-mounting /dev /sys /proc..."
mount_root --bind /dev /newroot/dev
mount_root --bind /sys /newroot/sys
mount_root --bind /proc /newroot/proc
echo "initramfs: /newroot/dev/dri:"
ls -la /newroot/dev/dri/ 2>&1
echo "initramfs: /newroot/sys/class/drm:"
ls -la /newroot/sys/class/drm/ 2>&1
echo "initramfs: sys/class content:"
ls /newroot/sys/class/ 2>&1 | head -20
echo "initramfs: modetest will run after UI starts (vhost-user gpu not ready yet)"
echo "initramfs: DRM connector status:"
for d in /newroot/sys/class/drm/card0-Virtual-*; do
    s=$(cat $d/status 2>&1)
    m=$(cat $d/modes 2>&1)
    e=$(cat $d/enabled 2>&1)
    echo "$(basename $d): status=$s modes=$m enabled=$e"
done

echo "initramfs: chroot..."
exec /bin/chroot /newroot /sbin/init
echo "initramfs: FAILED"
exec /bin/sh
INITEOF
chmod 755 /tmp/initramfs/init

# Create cpio archive
cd /tmp/initramfs
find . -print0 | cpio --null -o -H newc --quiet > /tmp/initrd 2>/dev/null
echo "cpio: $(stat -c%s /tmp/initrd) bytes"

# Verify
rm -rf /tmp/verify
mkdir -p /tmp/verify
cd /tmp/verify
cpio -i --quiet < /tmp/initrd 2>/dev/null
echo "verify: init exists: $([ -f ./init ] && echo yes || echo no)"
echo "verify: bin/sh exists: $([ -f ./bin/sh ] && echo yes || echo no)"
file ./bin/sh 2>/dev/null || echo "bin/sh not a file"
rm -rf /tmp/verify

# Compress
gzip -9 -c /tmp/initrd > /mnt/c/workspace/bscp/bscp/out/dist/img/initramfs.cpio.gz
echo "Done: $(stat -c%s /mnt/c/workspace/bscp/bscp/out/dist/img/initramfs.cpio.gz) bytes"
