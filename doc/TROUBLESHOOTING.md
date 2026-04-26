# Troubleshooting Guide

> Common issues and fixes for the cross-platform AVF host runtime.

## Build Issues

### CMake fails with "ninja: command not found"

**Solution**: Install Ninja:
- macOS: `brew install ninja`
- Linux: `sudo apt-get install ninja-build` (or `ninja` on Fedora/RHEL)
- **Important**: On macOS with depot_tools, an incompatible stub `ninja` may be on PATH. Either uninstall depot_tools or run `brew link --overwrite ninja` after `brew install ninja`.

### macOS: "missing com.apple.security.hypervisor entitlement"

The crosvm binary must be codesigned with the Hypervisor entitlement.

**Fix**:
```bash
codesign --force --sign - --entitlements scripts/macos_crosvm.entitlements --timestamp=none out/dist/macos/bin/crosvm
```

Or rebuild with `build_all.sh` which does this automatically.

### macOS: "HVF hv_vm_create failed"

Hypervisor.framework may not be enabled or the entitlement is invalid.

**Check**:
```bash
sysctl kern.hv_support   # Should print "kern.hv_support: 1"
```

If not enabled, run:
```bash
sudo nvram boot-args="-arm64e_preview_abi"
```
Then reboot.

### macOS: "error: could not compile `crosvm`" — missing `cc` crate or linker error

**Fix**: Ensure the `cc` build dependency is present. The `net_util` crate now uses `cc` to compile `vmnet_shim.c`. Run:
```bash
cargo install cc
# Or ensure Cargo.toml has: [build-dependencies] cc = "1"
```

### Cargo build fails on Windows with MinGW

The `binder` crate may trigger const-eval errors around `ParcelFileDescriptor` with the MinGW toolchain.

**Workaround**: Switch to the MSVC toolchain or pin to a known-working Rust nightly:
```bash
rustup toolchain install nightly-2024-10-01 --target x86_64-pc-windows-gnu
```

## Runtime Issues

### "Failed to load payload metadata"

The APEX tree is incomplete or the payload binary is missing.

**Fix**:
- Ensure `$VIRTMGR_APEX_ROOT/com.android.virt` contains the payload.
- For Microdroid: check `$APEX_TREE_ROOT/apex/com.android.virt/app/com.android.virt/`.
- Use `prepare_host_apex_tree.sh` with a complete AOSP build output.

### Frame dropping / "PFIFO timed out" on VM console

If the guest framebuffer shows dropped frames, the console may be overwhelmed.

**Check**: Ensure TMPDIR has enough disk space:
```bash
df -h $TMPDIR
```

### "Address already in use" when assigning CID

The CID is already taken by a previous (possibly stale) virtmgr instance.

**Fix**:
```bash
# Clean up all virtmgr/crosvm processes
scripts/vm_macos.sh -Command cleanup

# Or manually:
pkill -f virtmgr 2>/dev/null || true
pkill -f crosvm 2>/dev/null || true
```

### "get_virtualization_service is not supported"

This API is only available on the Android platform. Desktop host builds use a stub implementation. This is expected behavior.

### ADB connection fails

**Check**:
1. Guest adbd is running: Look for `adbd listening on vsock:5555` in guest logs.
2. Port is available: `lsof -i :8035` (or your configured ADB port).
3. The TCP bridge is active: Check crosvm/virtmgr logs for `startHostVsockTcpBridge`.

### macOS: "failed to set CID for guest" on retry

A stale `/tmp/binder_rpc_vsock_*.sock` or CID file remains from a previous run.

**Fix**:
```bash
rm -f /tmp/binder_rpc_vsock_*.sock
```
Or use the `cleanup` command: `scripts/vm_macos.sh -Command cleanup`

### Linux: "crosvm: Permission denied" for `/dev/kvm`

The user does not have read/write access to the KVM device.

**Fix**:
```bash
sudo usermod -aG kvm $USER
# Then log out and log back in.
```

### Windows: "vm console" shows empty output

The Windows console is file-backed (not TTY interactive). Read the console file directly:
```powershell
Get-Content out/dist/logs/windows-*/work/vm-console.txt -Tail 20
```

### Mock permission/SELinux not working as expected

The unified `MockPermissionProvider` / `MockSelinuxProvider` support three modes:

| Mode | Env Config | Behavior |
|------|-----------|----------|
| **bypass** (default) | No env vars set | All permissions/SELinux labels pass with a one-time warning |
| **allowlist** | `VIRTMGR_MOCK_PERMISSION_JSON` or `VIRTMGR_MOCK_SELINUX_JSON` | JSON file with `{"mode": "allowlist", "allowlist": ["perm1"]}`; unknown entries rejected |
| **strict** | `VIRTMGR_STRICT_PARITY=1` | All checks rejected (no silent bypass) |

Backward-compatible CSV env vars are also supported:
- `VIRTMGR_MOCK_PERMISSION_ALLOWLIST` (comma-separated) or `VIRTMGR_MOCK_PERMISSION_ALLOWLIST_FILE` (one per line)
- `VIRTMGR_MOCK_SELINUX_LABEL_ALLOWLIST` (comma-separated) or `VIRTMGR_MOCK_SELINUX_LABEL_ALLOWLIST_FILE` (one per line)

**Check**: If permissions are unexpectedly passing or failing, verify which mode is active by checking `VIRTMGR_STRICT_PARITY` and the mock JSON/env vars.

### All platforms: Guest log is empty

The `guest-log.txt` file may be at a different location depending on whether persistent mode is used.

**For persistent mode**: Check the work directory:
```
ls <service-root>/run-<cid>/guest-log.txt
```

**For non-persistent mode**: It's at `<log-dir>/guest-log.txt`.

## Platform-Specific Issues

### macOS

| Symptom | Likely Cause | Fix |
|---------|------------|-----|
| `HVF vcpu run failed` | Invalid memory mapping | Rebuild with clean APEX tree |
| `guest kernel: x86_64` | Wrong kernel arch | Run `scripts/fetch_arm64_guest_artifacts.sh` |
| `codesign: invalid entitlement` | Entitlements file out of date | Update `scripts/macos_crosvm.entitlements` |
| crosvm crashes on startup | Missing HVF entitlement | Confirm `codesign -d --entitlements :- crosvm` shows HVF |

### Linux

| Symptom | Likely Cause | Fix |
|---------|------------|-----|
| `Permission denied` for `/dev/kvm` | Not in kvm group | `sudo usermod -aG kvm $USER` |
| `Could not connect to vsock` | Kernel module not loaded | `sudo modprobe vhost_vsock` |
| crosvm: sandbox_init failure | seccomp-bpf not supported | Use `default-no-sandbox` feature |

### Windows

| Symptom | Likely Cause | Fix |
|---------|------------|-----|
| `WHPX: access denied` | Hypervisor platform not enabled | Enable Windows Hyper-V features |
| `Named pipe not found` | crosvm not started | Check crosvm.exe in task manager |
| console is empty | File-backed console | Use `Get-Content <console-path>` |
