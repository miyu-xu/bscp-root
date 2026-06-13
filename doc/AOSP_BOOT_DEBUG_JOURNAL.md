# AOSP Android 15 Boot on WHPX/crosvm — Debug Journal

## Goal

Boot a **pure AOSP Android 15** image (built from source) on **crosvm + WHPX** (Windows Hypervisor Platform) with **working GPU rendering** (virtio-gpu via gfxstream/ANGLE).

## Environment

| Component | Detail |
|-----------|--------|
| Host | Windows 11, Intel i7-1165G7 |
| Hypervisor | WHPX (Windows Hypervisor Platform) |
| VMM | crosvm (built from `external/crosvm`) |
| GPU backend | gfxstream (ANGLE/Vulkan) |
| AOSP build | `aosp_cf_x86_64_phone-trunk_staging-userdebug` (Android 15 / VanillaIceCream) |
| AOSP source | `/opt/workspace/aosp` on Ubuntu VM (WSL2/VirtualBox) |
| Kernel (GPP) | 6.1.166-android14 (GPP prebuilt) |
| Kernel (AOSP) | 6.6.30-android15 (prebuilt, at `packages/modules/Virtualization/guest/kernel/android15-6.6/x86_64/`) |

## Architecture Overview

```
┌─────────────────────────────────────────────────┐
│ Windows Host                                     │
│  ┌───────────────────────────────────────────┐  │
│  │ crosvm (WHPX)                             │  │
│  │  ┌─────────────────────────────────────┐  │  │
│  │  │ Android VM                           │  │  │
│  │  │  Kernel → first_stage_init           │  │  │
│  │  │    ├─ Load modules (virtio_blk etc)  │  │  │
│  │  │    ├─ Mount super partition           │  │  │
│  │  │    ├─ Create logical partitions (dm)  │  │  │
│  │  │    ├─ Mount system/vendor/product     │  │  │
│  │  │    └─ switch_root → second_stage_init │  │  │
│  │  │                                       │  │  │
│  │  │  second_stage_init                    │  │  │
│  │  │    ├─ Load SELinux policy             │  │  │
│  │  │    ├─ Parse init .rc files            │  │  │
│  │  │    ├─ exec_start apexd-bootstrap      │  │  │ ← FAILS HERE
│  │  │    ├─ Start servicemanager            │  │  │
│  │  │    ├─ Start surfaceflinger            │  │  │
│  │  │    └─ bootanim → launcher             │  │  │
│  │  └─────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────┘  │
│  GPU: gfxstream → ANGLE → Vulkan → Host GPU      │
└─────────────────────────────────────────────────┘
```

## Disk Layout

The VM uses a GPT disk created from two sources:
1. **AOSP super partition** (`super.img`) — contains system, vendor, product, odm, etc.
2. **GPP boot_a** partition — contains kernel ramdisk (for module loading)

```
GPT Disk (aggregate_aosp.img):
┌──────────────────────────────────────────────┐
│ LBA 0:     Protective MBR                    │
│ LBA 1:     GPT Header (primary)              │
│ LBA 2-33:  GPT Partition Entries             │
│ LBA 2048:  super (AOSP, ~7GB)                │
│            ├─ product_a (ext4)               │
│            ├─ system_a (ext4)                │
│            ├─ system_ext_a (ext4)            │
│            ├─ odm_a (ext4)                   │
│            ├─ vendor_a (ext4)                │
│            └─ ... (dlkm partitions)          │
│ LBA X:     boot_a (GPP, 32MB)               │
│ Last-33:   GPT Partition Entries (backup)    │
│ Last:      GPT Header (backup)               │
└──────────────────────────────────────────────┘
```

## Complete Timeline of Issues & Fixes

### Phase 1: Filesystem Type (EROFS → ext4)

**Problem**: AOSP 15 builds super.img with EROFS by default. GPP kernel (6.1.166) does NOT support EROFS. Kernel panics with "Invalid ext4 superblock" when trying to mount.

**Fix**: Override BoardConfig.mk to use ext4:
```makefile
# device/google/cuttlefish/vsoc_x86_64/BoardConfig.mk
BOARD_SYSTEMIMAGE_FILE_SYSTEM_TYPE := ext4
BOARD_VENDORIMAGE_FILE_SYSTEM_TYPE := ext4
BOARD_PRODUCTIMAGE_FILE_SYSTEM_TYPE := ext4
BOARD_SYSTEM_EXTIMAGE_FILE_SYSTEM_TYPE := ext4
```

**Script**: `scripts/aosp_ext4_boardconfig.patch` — documents the BoardConfig changes.

---

### Phase 2: Android Sparse Image Conversion

**Problem**: AOSP outputs super.img in Android Sparse Image format (magic `0xED26FF3A`). crosvm requires raw images for GPT disks.

**Fix**: Created `scripts/simg2img.py` — Python converter that handles:
- Android sparse image v1.0 format
- Chunk types: RAW (0xCAC1), FILL (0xCAC2), DONTCARE (0xCAC3), CRC32 (0xCAC4)
- Critical detail: `total_size` in chunk header includes 12-byte header, so data_size = total_size - 12

---

### Phase 3: GPT Disk Assembly

**Problem**: crosvm's `--block` expects a partitioned disk. The AOSP super partition alone won't work — it needs a GPT partition table.

**Fix**: Created `scripts/create_aosp_disk.py` — creates GPT disk with:
1. Protective MBR at LBA 0
2. GPT header at LBA 1 (92 bytes + padding)
3. Partition entry array at LBA 2
4. super partition at LBA 2048 (1MB aligned)
5. boot_a partition for ramdisk data
6. Backup GPT at end of disk

Uses the GPP disk's partition type GUIDs for compatibility.

---

### Phase 4: Kernel Module Compatibility

**Problem 1**: GPP kernel modules compiled for 6.1.166 fail to load on AOSP 6.6.30 kernel ("Exec format error" — vermagic mismatch).

**Problem 2**: AOSP kernel prebuilt (6.6.30, 9MB bzImage) hangs after console init — lacks built-in virtio drivers. GPP ramdisk modules don't match.

**Resolution**: Continue using GPP kernel (6.1.166) with its matching ramdisk modules. The GPP kernel boots fully and mounts all partitions correctly.

---

### Phase 5: SELinux Policy — Empty Policy Fallback

**Problem**: Without `precompiled_sepolicy` in vendor, init tries to compile CIL files at boot:
```
secilc: Failed to resolve typeattributeset statement at /product/etc/selinux/product_sepolicy.cil:1
```
The CIL compilation fails because `product_sepolicy.cil` references type attributes defined in other partitions (cross-partition reference issue). With no policy loaded, init FATAL-reboots.

**Fix (3 patches to `system/core/init/selinux.cpp`)**:

```cpp
// Patch 1: ReadPolicy() — return early when permissive + no enforcing
void ReadPolicy(std::string* policy) {
    if (ALLOW_PERMISSIVE_SELINUX && !IsEnforcing()) {
        LOG(WARNING) << "Skipping SELinux policy (permissive mode)";
        return;  // <— policy stays empty, init continues
    }
    // ... original code
}

// Patch 2: Same logic for ReadFdToString() failure path
// Patch 3: LoadSelinuxPolicy() — skip load if policy is empty
if (policy.empty()) {
    LOG(WARNING) << "Empty policy, skipping load (permissive)";
    return;
}
```

**Script**: `scripts/fix_selinux.py` — applies these patches to selinux.cpp.

**Result**: init no longer reboots on SELinux policy failure. Boot proceeds to second stage init.

---

### Phase 6: Precompiled SELinux Policy (WORKING APPROACH)

**Problem**: With empty SELinux policy, `apexd-bootstrap` fails:
```
Could not start exec service: File /system/bin/apexd(labeled "u:object_r:apexd_exec:s0")
has incorrect label or no domain transition from kernel to another SELinux domain defined.
```
The kernel's SELinux LSM, even in permissive mode, cannot perform domain transitions without a loaded policy. `apexd` must transition from `kernel` domain to `apexd` domain — impossible with no policy.

**Solution**: Provide `precompiled_sepolicy` in vendor partition.

**Source**: `$OUT/odm/etc/selinux/precompiled_sepolicy` (769KB) + 3 SHA256 companion files.

**Method**: Use `debugfs -w` on vendor.img to add the files:
```bash
debugfs -w vendor.img
cd /etc/selinux
write /path/to/precompiled_sepolicy precompiled_sepolicy
```

**Result**: With precompiled_sepolicy in vendor, the policy loads successfully, and apexd-bootstrap starts (confirmed in v4+ builds).

---

### Phase 7: Duplicate Camera APEX

**Problem**: After loading SELinux policy and starting apexd-bootstrap, init reboots with reason `bootstrap-apexd-failed`. Root cause: duplicate camera APEX packages in vendor:
- `com.google.emulated.camera.provider.hal.apex`
- `com.google.emulated.camera.provider.hal.fastscenecycle.apex`

The `.fastscenecycle` variant is a duplicate of the base HAL — apexd detects a package name conflict and crashes.

**Fix**: Delete the fastscenecycle variant from vendor:
```bash
debugfs -w vendor.img
rm /apex/com.google.emulated.camera.provider.hal.fastscenecycle.apex
```

**Critical**: Must also delete `obj/PACKAGING/vendor_intermediates` before rebuilding super, otherwise the cached intermediate is used and the change is lost.

**Status**: super_v8 was built with this fix applied.

---

### Phase 8: GPP Kernel SELinux Class Gap

**Problem**: Even with valid precompiled_sepolicy, the GPP kernel (6.1.166) lacks the `property_service` SELinux class introduced in newer Android kernels. Every property set operation fails:
```
selinux: Unknown class property_service
init: Init cannot set 'ro.boot.hardware' to 'cutf_cvm': SELinux permission check failed
```
This causes cascading failures:
- `ro.hardware` not set → `/init.${ro.hardware}.rc` imports fail
- `ro.zygote` not set → `/init.${ro.zygote}.rc` imports fail
- All build properties fail to load

**Attempted fix**: `selinux=0` kernel parameter — no effect on GPP kernel (parameter might not be compiled in).

**Note**: Despite these failures, init continues past property loading. The fatal error is still apexd domain transition (see Phase 6).

---

## Current Boot Status (super_v8 + GPP kernel)

### What Works ✅
| Step | Status | Notes |
|------|--------|-------|
| Kernel boot | ✅ | GPP 6.1.166, all 8 virtio modules loaded |
| Super partition mount | ✅ | LP metadata parsed, 9 logical partitions created |
| System mount (ext4) | ✅ | dm-1 mounted at /system |
| Vendor mount (ext4) | ✅ | dm-6 mounted at /vendor |
| Product mount (ext4) | ✅ | dm-0 mounted at /product |
| switch_root | ✅ | Transitions to /system |
| Second stage init | ✅ | Parses all .rc files |
| SELinux CIL compilation | ❌ | `secilc` cross-partition reference failure |
| SELinux policy load | ❌ | No precompiled_sepolicy in v8 vendor (lost in clean rebuild) |
| Property loading | ⚠️ | Fails due to `property_service` class unknown (kernel 6.1) |
| apexd-bootstrap | ❌ | Domain transition failed (no policy loaded) |
| servicemanager | — | Not reached |
| surfaceflinger / GPU | — | Not reached |

### The Critical Blocker

```
┌────────────────────────────────────────────────────────────┐
│ apexd-bootstrap needs:                                     │
│                                                            │
│  1. Valid SELinux policy LOADED in kernel                  │
│     └─ Requires: precompiled_sepolicy in /vendor/etc/selinux│
│     └─ Status: ❌ Missing in super_v8 (clean rebuild lost it)│
│                                                            │
│  2. Domain transition: kernel → apexd domain               │
│     └─ Requires: policy that defines both domains          │
│     └─ Status: ❌ Can't happen with empty policy           │
└────────────────────────────────────────────────────────────┘
```

## AOSP Source Patches Required

These two patches MUST be applied to the AOSP source tree before building.

---

### Patch 1: SELinux Permissive Mode (system/core/init)

**File**: `system/core/init/selinux.cpp`
**Why**: Without this patch, init calls `LOG(FATAL)` and reboots when SELinux policy cannot be opened or compiled. This is the default on WHPX/crosvm where the kernel may lack `property_service` class or CIL cross-partition compilation fails.

**How to apply**:
```bash
cd /opt/workspace/aosp
python3 fix_selinux.py system/core/init/selinux.cpp
```

**fix_selinux.py**:
```python
#!/usr/bin/env python3
"""Fix selinux.cpp to skip SELinux policy loading in permissive mode."""
import sys

with open(sys.argv[1], 'r') as f:
    c = f.read()

# Patch 1: ReadPolicy() — return early when permissive
# Insert early-return at the top of ReadPolicy() body, right after the opening brace
old_rp = '''void ReadPolicy(std::string* policy) {
    PolicyFile policy_file;'''
new_rp = '''void ReadPolicy(std::string* policy) {
    if (ALLOW_PERMISSIVE_SELINUX && !IsEnforcing()) {
        LOG(WARNING) << "Skipping SELinux policy (permissive mode)";
        return;
    }
    PolicyFile policy_file;'''
c = c.replace(old_rp, new_rp, 1)

# Patch 2: ReadPolicy() — same for ReadFdToString failure path
# (Handled by Patch 1 since ReadFdToString failure also calls LOG(FATAL),
#  and the early return skips the entire function body)

# Patch 3: LoadSelinuxPolicy() — skip load if policy string is empty
old_lp = '''    if (security_load_policy(policy.data(), policy.size()) < 0) {
        PLOG(FATAL) << "SELinux:  Could not load policy";
    }'''
new_lp = '''    if (policy.empty()) {
        LOG(WARNING) << "Empty policy, skipping load (permissive)";
        return;
    }
    if (security_load_policy(policy.data(), policy.size()) < 0) {
        PLOG(FATAL) << "SELinux:  Could not load policy";
    }'''
c = c.replace(old_lp, new_lp, 1)

with open(sys.argv[1], 'w') as f:
    f.write(c)
print('PATCHED')
```

**Resulting source changes** (`git diff system/core/init/selinux.cpp`):

```diff
 void ReadPolicy(std::string* policy) {
+    if (ALLOW_PERMISSIVE_SELINUX && !IsEnforcing()) {
+        LOG(WARNING) << "Skipping SELinux policy (permissive mode)";
+        return;
+    }
     PolicyFile policy_file;
```

```diff
 static void LoadSelinuxPolicy(std::string& policy) {
     LOG(INFO) << "Loading SELinux policy";
     set_selinuxmnt("/sys/fs/selinux");
+    if (policy.empty()) {
+        LOG(WARNING) << "Empty policy, skipping load (permissive)";
+        return;
+    }
     if (security_load_policy(policy.data(), policy.size()) < 0) {
         PLOG(FATAL) << "SELinux:  Could not load policy";
     }
```

**After patching**: The `init` binary must be rebuilt and the system image re-generated:
```bash
cd /opt/workspace/aosp
source build/envsetup.sh
lunch aosp_cf_x86_64_phone-trunk_staging-userdebug

# Clean init intermediates to force rebuild
find out/ -name "init" -type f -delete 2>/dev/null
rm -rf out/target/product/vsoc_x86_64/obj/PACKAGING/system_intermediates

m
```

---

### Patch 2: Filesystem Type (device/google/cuttlefish)

**File**: `device/google/cuttlefish/vsoc_x86_64/BoardConfig.mk`
**Why**: Android 15 defaults to EROFS for read-only partitions. The GPP kernel (6.1.166) does not support EROFS. The AOSP 6.6 kernel does have EROFS support but lacks built-in virtio drivers (see Path A in Alternatives). Override to ext4 for GPP kernel compatibility.

**Changes** — append to end of file:
```makefile
# Force ext4 for WHPX compatibility (override EROFS default)
BOARD_SYSTEMIMAGE_FILE_SYSTEM_TYPE := ext4
BOARD_VENDORIMAGE_FILE_SYSTEM_TYPE := ext4
BOARD_PRODUCTIMAGE_FILE_SYSTEM_TYPE := ext4
BOARD_SYSTEM_EXTIMAGE_FILE_SYSTEM_TYPE := ext4
```

**How to apply**:
```bash
cat >> device/google/cuttlefish/vsoc_x86_64/BoardConfig.mk << 'EOF'

# Force ext4 for WHPX compatibility (override EROFS default)
BOARD_SYSTEMIMAGE_FILE_SYSTEM_TYPE := ext4
BOARD_VENDORIMAGE_FILE_SYSTEM_TYPE := ext4
BOARD_PRODUCTIMAGE_FILE_SYSTEM_TYPE := ext4
BOARD_SYSTEM_EXTIMAGE_FILE_SYSTEM_TYPE := ext4
EOF
```

---

### Full Rebuild Commands (after both patches)

On the AOSP build VM:
```bash
cd /opt/workspace/aosp

# Apply SELinux patch
python3 fix_selinux.py system/core/init/selinux.cpp

# Apply BoardConfig patch (if not already done)
cat >> device/google/cuttlefish/vsoc_x86_64/BoardConfig.mk << 'EOF'

BOARD_SYSTEMIMAGE_FILE_SYSTEM_TYPE := ext4
BOARD_VENDORIMAGE_FILE_SYSTEM_TYPE := ext4
BOARD_PRODUCTIMAGE_FILE_SYSTEM_TYPE := ext4
BOARD_SYSTEM_EXTIMAGE_FILE_SYSTEM_TYPE := ext4
EOF

# Clean and rebuild
rm -f out/.lock
source build/envsetup.sh
lunch aosp_cf_x86_64_phone-trunk_staging-userdebug

# Clean init to pick up SELinux fix
find out/ -name "init" -type f -delete 2>/dev/null
rm -rf out/target/product/vsoc_x86_64/obj/PACKAGING/system_intermediates

# Build everything
m

# Copy super to shared folder for Windows
cp out/target/product/vsoc_x86_64/super.img /media/sf_Desktop/super_vX.img
```

---

## Fix Required (Next Step)

### One-line summary
Add `precompiled_sepolicy` back to vendor.img and rebuild super.

### Detailed steps on VM:
```bash
# 1. Find precompiled_sepolicy in build output
find $OUT -name "precompiled_sepolicy" -type f

# 2. Add to vendor.img using debugfs
debugfs -w $OUT/vendor.img
cd /etc/selinux
write $SEPOLICY_PATH precompiled_sepolicy
# Also write companion sha256 files

# 3. Clean vendor intermediates (CRITICAL — otherwise cached copy used)
rm -rf $OUT/obj/PACKAGING/vendor_intermediates
rm -rf $OUT/obj/PACKAGING/super_intermediates

# 4. Rebuild super
source build/envsetup.sh
lunch aosp_cf_x86_64_phone-trunk_staging-userdebug
rm -f $OUT/super.img
m superimage

# 5. Copy to Desktop
cp $OUT/super.img /media/sf_Desktop/super_v9.img
```

### Then on Windows:
```batch
python simg2img.py super_v9.img super_aosp_ext4.img
python create_aosp_disk.py
run_aosp_ext4.bat
```

## Alternative Paths Considered

### Path A: Use AOSP 6.6 kernel
- **Status**: Tried, failed
- **Issue**: AOSP 6.6 kernel (9MB bzImage) hangs after console init. Doesn't have virtio drivers built-in. Needs matching initramfs with kernel modules (not available as prebuilt).
- **Verdict**: Requires building custom kernel with virtio built-in. Too complex for now.

### Path B: Disable SELinux at kernel level
- **Status**: Tried, failed
- **Issue**: `selinux=0` kernel parameter not recognized by GPP kernel (likely `CONFIG_SECURITY_SELINUX_BOOTPARAM=n`).
- **Verdict**: Kernel rebuild needed. Same complexity as Path A.

### Path C: Patch init to skip domain transition
- **Status**: Not attempted
- **Issue**: Need to modify `system/core/init/service.cpp` to skip SELinux domain transition checks when permissive. Requires another source patch + AOSP rebuild.
- **Verdict**: Worth trying if precompiled_sepolicy approach fails again.

### Path D: Compile CIL to binary policy on-device
- **Status**: Not attempted
- **Issue**: secilc fails due to cross-partition type references. Would need to merge all CIL files from system, product, vendor before compiling.
- **Verdict**: Complex. precompiled_sepolicy approach is simpler and proven.

## Key Files Reference

| File | Purpose |
|------|---------|
| `scripts/create_aosp_disk.py` | GPT disk creation from AOSP super partition |
| `scripts/simg2img.py` | Android sparse image → raw image converter |
| `scripts/fix_selinux.py` | Patch selinux.cpp for permissive mode |
| `scripts/extract_lp_partitions.py` | Extract logical partitions from super (WIP) |
| `scripts/run_aosp_ext4.bat` | Boot AOSP with GPP kernel + permissive SELinux |
| `scripts/run_aosp_v8.bat` | Boot AOSP with AOSP 6.6 kernel attempt |
| `scripts/vm_fix_sepolicy.sh` | VM-side script to add precompiled_sepolicy |
| `scripts/aosp_selinux_fix.patch` | Documents the 3 SELinux source patches |
| `scripts/aosp_ext4_boardconfig.patch` | Documents BoardConfig ext4 override |

## Boot Command Reference

```batch
crosvm run-mp --disable-sandbox --cid 100 --mem 4096 --cpus 4 --no-balloon --no-usb ^
  --serial "type=file,path=serial.txt,hardware=serial,num=1,earlycon=true" ^
  --serial "type=file,path=hvc.txt,hardware=virtio-console,num=1,console=true" ^
  --block "path=aggregate_aosp.img,ro=false,lock=false,sparse=false" ^
  --initrd "ramdisk_noavb.gz" ^
  --params "console=hvc0,ttyS0 loglevel=7 printk.devkmsg=on init=/init 
            androidboot.hardware=cutf_cvm 
            androidboot.selinux=permissive 
            androidboot.slot_suffix=_a" ^
  --gpu "backend=gfxstream,width=1280,height=720,angle=true,vulkan=true,wsi=vk" ^
  android_kernel
```

## Summary

After extensive debugging (~30+ boot attempts across 8 super image versions), the AOSP Android 15 boot on WHPX/crosvm is **one step away** from success:

1. ✅ AOSP builds as ext4 (BoardConfig override)
2. ✅ ext4 super boots on GPP kernel (all partitions mount)
3. ✅ SELinux permissive patch works (init doesn't reboot on policy failure)
4. ✅ precompiled_sepolicy loading works (proven in v4-v7 builds)
5. ✅ Camera APEX duplicate fixed (v8)
6. ❌ **precompiled_sepolicy missing from v8** (clean rebuild lost debugfs changes)
7. ❌ apexd-bootstrap domain transition fails without loaded policy

**Next action**: Add precompiled_sepolicy to vendor, rebuild super (v9), boot. This should reach the Android desktop with GPU rendering.

---

*Last updated: 2026-06-08*
