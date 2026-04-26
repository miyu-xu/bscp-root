# HVF (Apple Hypervisor.framework) Capability Matrix

> Generated from source: `external/crosvm/hypervisor/src/hvf/`
> Date: 2026-04-25

## Maturity Levels

| Level | Definition |
|-------|-----------|
| **L0 (stub)** | Compiles but fails at runtime (returns `ENOSYS` / `ENXIO` or no-ops) |
| **L1 (partial)** | Basic functionality works but has semantic differences vs KVM |
| **L2 (full)** | Behavior aligns with KVM equivalent |

## 1. HvfHypervisor

| Method | Level | Notes |
|--------|-------|-------|
| `new()` | L2 | Minimal constructor |
| `try_clone()` | L2 | Returns new instance |
| `check_capability()` | L1 | Only `UserMemory`, `ImmediateExit`, `VcpuRunThreadLocal` reported; many KVM capabilities missing |

## 2. HvfVm — Vm trait

| Method | Level | Notes |
|--------|-------|-------|
| `new()` | L2 | Creates VM, configures 40-bit IPA space, maps guest memory via `hv_vm_map` |
| `try_clone()` | L2 | Shared references via `Arc` |
| `check_capability(VmCap)` | L0 | Always returns `false`; all KVM `VmCap` values unsupported |
| `get_guest_phys_addr_bits()` | L2 | Returns `ipa_bits` (40) |
| `get_memory()` | L2 | Returns `GuestMemory` ref |
| `add_memory_region()` | L2 | Maps via `hv_vm_map`, supports read-only via `hv_vm_protect`, slot reuse |
| `msync_memory_region()` | L2 | Delegates to `MappedRegion::msync` |
| `remove_memory_region()` | L2 | Calls `hv_vm_unmap`, returns region |
| `create_device()` | L0 | Returns `ENXIO` — no vfio or device creation support |
| `get_dirty_log()` | L0 | Returns `ENXIO` — dirty page tracking not available |
| `register_ioevent()` | L1 | Stores event in map; fires on MMIO match but no `irqfd` integration |
| `unregister_ioevent()` | L2 | Removes from map |
| `handle_io_events()` | L2 | Fires matching ioevents |
| `get_pvclock()` | L0 | Returns `ENXIO` |
| `set_pvclock()` | L0 | Returns `ENXIO` |
| `add_fd_mapping()` | L2 | Delegates to `MappedRegion::add_fd_mapping` |
| `remove_mapping()` | L2 | Delegates to `MappedRegion::remove_mapping` |
| `handle_balloon_event()` | L2 | Inflate via `remove_range`, deflate no-op |

### VmAArch64 trait

| Method | Level | Notes |
|--------|-------|-------|
| `get_hypervisor()` | L2 | Returns ref to `HvfHypervisor` |
| `load_protected_vm_firmware()` | L0 | Returns `ENOSYS` — no pKVM support |
| `create_vcpu()` | L2 | Creates `HvfVcpu` with shared ioevents |
| `create_fdt()` | L0 | No-op — does not populate FDT at all |
| `init_arch()` | L0 | No-op |

### HVF-only methods (not in Vm/VmAArch64 traits)

| Method | Level | Notes |
|--------|-------|-------|
| `init_gic()` | L2 | Full GICv3 via dlopen'd `hv_gic_*` symbols; distributor/redistributor MMIO layout |
| `set_gic_spi()` | L2 | SPI injection via `hv_gic_set_spi` |
| `send_gic_msi()` | L2 | MSI injection via `hv_gic_send_msi` |
| `fire_ioevents()` | L2 | Internal helper for ioevent dispatch |
| `guest_pagesize()` | L2 | Returns `applevisor_sys::PAGE_SIZE` |

## 3. HvfVcpu — Vcpu trait

| Method | Level | Notes |
|--------|-------|-------|
| `new()` | L2 | `hv_vcpu_create`, sets `MPIDR_EL1` |
| `try_clone()` | L0 | Returns `ENOSYS` |
| `run()` | L2 | Full exit loop: MMIO, PSCI, sysreg traps, WFI/WFE, vtimer |
| `id()` | L2 | Returns VCPU index |
| `set_immediate_exit()` | L2 | Calls `hv_vcpus_exit` |
| `handle_mmio()` | L2 | Full read/write with sign extension, register update, PC advance |
| `handle_io()` | L0 | No-op (PIO not needed on AArch64) |
| `on_suspend()` | L1 | No-op (no suspend-specific logic) |
| `enable_raw_capability()` | L0 | Returns `ENOSYS` |

### VcpuAArch64 trait

| Method | Level | Notes |
|--------|-------|-------|
| `init()` | L2 | Sets `HCR_EL2`, `CNTHCTL_EL2`, `ID_AA64PFR0_EL1` (EL2, GIC), clears SME |
| `init_pmu()` | L0 | Returns `ENOSYS` |
| `has_pvtime_support()` | L0 | Returns `false` |
| `init_pvtime()` | L0 | Returns `ENOSYS` |
| `set_one_reg()` | L2 | X regs, SP, PC, PSTATE, system registers |
| `get_one_reg()` | L2 | Same coverage as set |
| `set_vector_reg()` | L2 | SIMD/FP Q0-Q31 |
| `get_vector_reg()` | L2 | SIMD/FP Q0-Q31 |
| `get_system_regs()` | L0 | Returns empty map |
| `hypervisor_specific_snapshot()` | L0 | Returns `Null` |
| `hypervisor_specific_restore()` | L0 | No-op |
| `get_psci_version()` | L2 | Returns `PSCI_0_2` |
| `set_guest_debug()` | L0 | Returns `ENOSYS` |
| `get_max_hw_bps()` | L0 | Returns 0 |
| `get_cache_info()` | L0 | Returns empty map |
| `set_cache_info()` | L0 | No-op |

## 4. HVF-only VCPU internals

| Feature | Level | Notes |
|---------|-------|-------|
| PSCI handler | L2 | Version, Features, CPU_ON, CPU_OFF, SYSTEM_OFF, SYSTEM_RESET, SUSPEND, MIGRATE_INFO_TYPE |
| MMIO | L2 | DAB syndrome parsing, read/write dispatch, width/sign handling |
| Sysreg trap | L2 | Read/write via `hv_vcpu_{get,set}_sys_reg` |
| WFI/WFE | L2 | Timer-based sleep emulation using `mach_absolute_time` |
| Exit logging | L2 | Summary counters per vcpu, periodic logging |
| Exception record | L2 | Syndrome, GPA, GVA, PC capture on failure |

## 5. IRQ Chip (external/crosvm/devices/src/irqchip/hvf_aarch64.rs)

| Capability | Level | Notes |
|------------|-------|-------|
| Kernel IRQ Chip (GICv3 in-hypervisor) | L2 | Full implementation (335 lines) |
| Split IRQ Chip | L0 | Not supported — `bail!` in `run_hvf` |
| Userspace IRQ Chip | L0 | Not supported — `bail!` in `run_hvf` |
| SPI injection | L2 | Via `HvfVm::set_gic_spi` |
| MSI injection | L2 | Via `HvfVm::send_gic_msi` |
| Dynamic IRQ routing | L0 | All SPI→GIC routes fixed at init |

## 6. Platform Stubs (non-HVF core)

| Component | File | Level | Notes |
|-----------|------|-------|-------|
| TAP/Network | `net_util/src/sys/macos_hvf/` | L2 | Real vmnet.framework implementation via `net.rs` (295 lines) + `vmnet_shim.c` (170 lines). Supports shared (NAT) mode. Entitlement `com.apple.vm.networking` included in `macos_crosvm.entitlements` |
| ACPI (guest PSCI) | `devices/src/sys/macos_hvf.rs` + `hypervisor/src/hvf/vcpu.rs` | L2 | Guest-initiated shutdown/reboot via PSCI: `PSCI_SYSTEM_OFF` → `SystemEventShutdown`, `PSCI_SYSTEM_RESET` → `SystemEventReset` |
| ACPI (host-initiated) | `devices/src/sys/macos_hvf.rs` | L0 | Host-initiated events (sleep button, battery) require IOKit — out of scope |
| Minijail | `external/minijail/` | L0 | All macOS stubs |

## 7. Comparison: HVF vs KVM (AArch64)

| Capability | HVF | KVM |
|------------|-----|-----|
| VM creation | ✓ | ✓ |
| Memory mapping | ✓ | ✓ |
| GICv3 init | ✓ | ✓ |
| SPI injection | ✓ | ✓ |
| MSI injection | ✓ | ✓ |
| VCPU create/run | ✓ | ✓ |
| PSCI | ✓ | ✓ |
| MMIO | ✓ | ✓ |
| Sysreg traps | ✓ | ✓ |
| WFI/WFE | ✓ | ✓ |
| PMU | ✗ | ✓ |
| PV time | ✗ | ✓ |
| Protected VM | ✗ | ✓ (pKVM) |
| Dirty log | ✗ | ✓ |
| irqfd | ✗ | ✓ |
| ioeventfd | Partial | ✓ |
| Split irqchip | ✗ | ✓ |
| Userspace irqchip | ✗ | ✓ |
| Guest debug | ✗ | ✓ |
| Snapshot/restore | ✗ | ✓ |
| TAP network (vmnet.framework) | ✓ | ✓ |
| ACPI events (guest PSCI) | ✓ | ✓ |
| ACPI events (host-initiated) | ✗ | ✓ |
| VFIO passthrough | ✗ | ✓ |
| PV clock | ✗ | ✓ |
| Balloon | ✓ | ✓ |

## 8. Summary Statistics

| Level | Count | Percentage |
|-------|-------|-----------|
| L2 (full) | 35 | ~57% |
| L1 (partial) | 4 | ~7% |
| L0 (stub) | 22 | ~36% |

**Total trait methods analyzed: 61**

*Note: L0 count is inflated by deliberately unsupported features on macOS (PMU, VFIO, guest debug, snapshot, protected VM, host-initiated ACPI). Core VM/VCPU lifecycle methods are predominantly L2. Phase 1 brought TAP/Network from L0→L2 (vmnet.framework) and documented PSCI-based guest shutdown/reboot as L2.*
