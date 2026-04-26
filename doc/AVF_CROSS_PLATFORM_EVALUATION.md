# AVF 跨平台实现综合评估报告

> 评估时间：2026-04-25
> 评估范围：Linux (KVM)、macOS (HVF)、Windows (WHPX) 三平台 AVF host runtime
> 评估目标：基于 crosvm + Microdroid 的跨平台虚拟化方案现状、缺陷与后续工作计划

---

## 目录

1. [总体结论](#一总体结论)
2. [三平台能力对比矩阵](#二三平台能力对比矩阵)
3. [macOS HVF 实现专项评估](#三macos-hvf-实现专项评估)
4. [Linux KVM 实现评估](#四linux-kvm-实现评估)
5. [Windows WHPX 实现评估](#五windows-whpx-实现评估)
6. [三平台缺陷汇总](#六三平台缺陷汇总)
7. [建议](#七建议)
8. [Phase 1 详细实施计划：HVF 能力补齐](#八phase-1-详细实施计划hvf-能力补齐)
9. [Phase 2 详细实施计划：跨平台统一](#九phase-2-详细实施计划跨平台统一)
10. [Phase 3 详细实施计划：稳定化与 CI](#十phase-3-详细实施计划稳定化与-ci)
11. [前瞻探索（Phase 4+）](#十一前瞻探索phase-4)
12. [关键文件索引](#十二关键文件索引)

---

## 一、总体结论

### 1.1 核心判断

当前三平台 AVF host runtime 均已打通核心链路：

```
vm → libvmclient → virtmgr → Binder RPC → crosvm → Microdroid guest
```

unprotected Microdroid 可在三个平台上启动到 `notifyPayloadReady`。但三平台成熟度差异显著：

- **Linux (KVM)**：最成熟，对齐度 ~75-80%
- **Windows (WHPX)**：中等，对齐度 ~70-75%
- **macOS (HVF)**：最低，对齐度 ~60-65%

macOS HVF 是三个 hypervisor 后端中最不成熟的，网络/ACPI 等关键能力仍全部为桩。

### 1.2 定位声明

当前三平台的准确定位是 **desktop host runtime**，不是 Android 设备语义的 1:1 复刻。Permission / SELinux / 系统服务语义不做完整复刻，protected VM 全平台缺失。

---

## 二、三平台能力对比矩阵

### 2.1 核心能力总览

| 能力域 | Linux (KVM) | macOS (HVF) | Windows (WHPX) |
|--------|-------------|-------------|----------------|
| **构建链路** | build_all.sh | build_all.sh | build_all.bat |
| **Hypervisor 成熟度** | 高（Linux 主线，4源文件） | 低（移植中，~1910行） | 中（3源文件） |
| **Binder RPC 传输** | Unix domain socket | Unix domain socket | Named pipe + TCP |
| **VM vsock 通信** | 真实 AF_VSOCK | UDS 模拟 (`/tmp/`) | Named pipe |
| **VM 启动 (unprotected)** | 完整，已验证 | 完整，已验证 | 完整，已验证 |
| **Protected VM** | 未接入 | 不支持 | 不支持 |
| **网络虚拟化** | TAP 完整 | **全部为桩（23行）** | 部分支持 |
| **ACPI / netlink** | 完整 | **全部为桩（28行）** | 部分支持 |
| **IRQ Chip 模式** | Split/Userspace/Kernel | **仅 Kernel** | Kernel |
| **VM Console** | PTY/TTY 交互式 | PTY/TTY 交互式 | File-backed 只读 |
| **ADB Bridge** | TCP→vsock 代码桥 | TCP→vsock 代码桥 | TCP→pipe 代码桥 |
| **VM 控制 (suspend/resume/balloon)** | crosvm 控制 socket | crosvm 控制 socket | crosvm CLI 子进程 |
| **持久化 virtmgr service** | 支持 | 支持 | 支持 |
| **Path mapping (APEX/system)** | 支持 | 支持 | 支持 |
| **Minijail sandbox** | 真实 minijail | 全部为桩 | 全部为桩 |
| **设备直通 (VFIO)** | 支持 | **显式拒绝** | 不支持 |
| **SCSI / pmem / shared-dir / vhost** | 支持 | **显式拒绝** | 部分 |
| **Wrapper 脚本** | vm_linux.sh | vm_macos.sh | vm_windows.ps1 |
| **vm_shell 脚本** | vm_shell_linux.sh | vm_shell_macos.sh | vm_shell_windows.ps1 |
| **回归脚本** | run_linux_avf_regression.sh | run_macos_avf_regression.sh | run_windows_avf_regression.ps1 |
| **标记检查脚本** | check_linux_avf_markers.sh | check_macos_avf_markers.sh | check_windows_avf_markers.ps1 |

### 2.2 vm CLI 命令支持矩阵

| 命令 | Linux | macOS | Windows |
|------|-------|-------|---------|
| `validate-prereqs` | 支持 | 支持 | 支持 |
| `run-microdroid` | 支持（到 notifyPayloadReady） | 支持 | 支持 |
| `run-app` | 支持 | 支持 | 支持 |
| `run` (raw config) | Partial | Partial | Partial |
| `info` | 支持 | 支持 | 支持 |
| `list` | Persistent mode 可用 | Persistent mode 可用 | Persistent mode 可用 |
| `console` | PTY 交互式 | PTY 交互式 | File-backed 只读 |
| `check-feature-enabled` | 支持 | 支持 | 支持 |
| `create-partition` | 支持 | 支持 | 支持 |
| `create-idsig` | 支持 | 支持 | 支持 |
| `service-status` | 支持 | 支持 | 支持 |
| `stop-service` | 支持 | 支持 | 支持 |

### 2.3 环境变量支持

| 变量 | Linux | macOS | Windows |
|------|-------|-------|---------|
| `RUST_TARGET` | x86_64/aarch64-unknown-linux-gnu | aarch64-apple-darwin | x86_64-pc-windows-gnu |
| `VIRTMGR_CROSVM_PATH` | 可选覆盖 | 可选覆盖 | crosvm.exe 路径 |
| `VIRTMGR_PATH` | virtmgr 路径 | virtmgr 路径 | virtmgr.exe 路径 |
| `VIRTMGR_SERVICE_DIR` | 支持 | 支持 | 支持 |
| `VIRTMGR_APEX_ROOT` | 支持 | 支持 | 支持 |
| `VIRTMGR_SYSTEM_ROOT` | 支持 | 支持 | 支持 |
| `VIRTMGR_DEBUG_POLICY_JSON` | 支持 | 支持 | 支持 |
| `VIRTMGR_DT_OVERLAY_JSON` | 支持 | 支持 | 支持 |
| `VIRTMGR_STRICT_PARITY` | 支持 | 支持 | 支持 |
| `VIRTMGR_MICRODROID_JSON` | 支持 | 支持 | 支持 |
| `MACOS_AVF_APEX_TREE_SOURCE` | N/A | arm64 APEX tree | N/A |

---

## 三、macOS HVF 实现专项评估

### 3.1 实现规模

macOS HVF 后端约 **1910 行**核心代码，分布在 5 个源文件：

| 文件 | 行数 | 功能 |
|------|------|------|
| `external/crosvm/hypervisor/src/hvf/vm.rs` | 652 | VM 创建、内存映射、GICv3 初始化、IPA 空间配置 |
| `external/crosvm/hypervisor/src/hvf/vcpu.rs` | 872 | VCPU 创建/销毁、寄存器读写、退出处理循环 |
| `external/crosvm/devices/src/irqchip/hvf_aarch64.rs` | 335 | HVF 内核 GICv3 IRQ 芯片、SPI 投递 |
| `external/crosvm/devices/src/sys/macos_hvf.rs` | 28 | **ACPI/netlink 完全桩实现** |
| `external/crosvm/net_util/src/sys/macos_hvf/mod.rs` | 23 | **TAP 网络完全桩实现** |

**配套的 base 兼容层** 位于 `external/crosvm/base/src/sys/linux_macos/`（13 文件，约 1800 行），桥接 Darwin 与 Linux API：

| 文件 | 功能 |
|------|------|
| `mod.rs` | 模块入口，符号再导出 |
| `signal.rs` (18573 字节) | 信号处理兼容（sigaction/sigaltstack） |
| `signalfd.rs` | signalfd 近似实现 |
| `file.rs` | 文件操作兼容 |
| `file_traits.rs` | File trait 适配 |
| `net.rs` | 网络相关兼容 |
| `sched.rs` | 调度策略适配 |
| `shm.rs` | 共享内存 |
| `event.rs` | Event/EventFd |
| `priority.rs` | 线程优先级 |
| `capabilities.rs` | 能力检查桩 |
| `notifiers.rs` | CloseNotifier 等 |
| `stubs.rs` | 杂项桩 |

对比：
- **KVM** (`hypervisor/src/kvm/`)：4 文件（`aarch64.rs`, `cap.rs`, `mod.rs`, `x86_64.rs`），成熟完整
- **WHPX** (`hypervisor/src/whpx/`)：3 文件（`types.rs`, `vcpu.rs`, `vm.rs`），中等成熟度

### 3.2 入口与平台选择

#### 3.2.1 crosvm 平台入口 (`external/crosvm/src/crosvm/sys.rs`)

```rust
#[cfg(any(
    target_os = "android",
    target_os = "linux",
    all(target_os = "macos", target_arch = "aarch64", feature = "hvf")
))]
pub(crate) mod linux;   // macOS HVF 复用 Linux 平台代码

#[cfg(all(target_os = "macos", target_arch = "aarch64", feature = "hvf"))]
pub(crate) mod macos;   // macOS 专有配置/命令行

cfg_if::cfg_if! {
    if #[cfg(any(target_os = "android", target_os = "linux"))] {
        use linux as platform;
    } else if #[cfg(all(target_os = "macos", target_arch = "aarch64", feature = "hvf"))] {
        use macos as platform;   // macOS 使用 macos 作为平台入口
    } else if #[cfg(windows)] {
        use windows as platform;
    }
}
```

关键设计：macOS 使用**独立的 `macos` 平台模块**（含配置/命令行），但通过 `linux` 模块复用 VM 运行主流程（`run_config` → `run_hvf`）。

#### 3.2.2 run_hvf 主流程 (`external/crosvm/src/crosvm/sys/linux.rs:1799`)

```rust
#[cfg(all(target_os = "macos", target_arch = "aarch64", feature = "hvf"))]
fn run_hvf(cfg: Config, components: VmComponents) -> Result<ExitState> {
    use devices::HvfKernelIrqChip;
    use hypervisor::hvf::{HvfHypervisor, HvfVcpu, HvfVm};

    let hvf = HvfHypervisor::new()?;
    let guest_mem = create_guest_memory(&cfg, &components, &hvf)?;

    let vm = HvfVm::new(&hvf, guest_mem, components.hv_cfg)?;

    // 仅支持 Kernel IRQ Chip
    let mut irq_chip = match cfg.irq_chip.unwrap_or(IrqChipKind::Kernel) {
        IrqChipKind::Kernel => {
            HvfKernelIrqChip::new(vm_clone, components.vcpu_count)?
        }
        _ => bail!("HVF only supports kernel irqchip mode"),
    };

    run_vm::<HvfVcpu, HvfVm>(cfg, components, vm, &mut irq_chip, ...)
}
```

#### 3.2.3 默认 Hypervisor 选择 (`linux.rs:1849`)

```rust
fn get_default_hypervisor() -> Option<HypervisorKind> {
    #[cfg(all(target_os = "macos", target_arch = "aarch64", feature = "hvf"))]
    {
        return Some(HypervisorKind::Hvf);  // macOS 直接返回 HVF，不检查 /dev/kvm
    }
    // Linux: 检查 /dev/kvm → Geniezone → Gunyah
}
```

### 3.3 macOS 平台配置 (`external/crosvm/src/crosvm/sys/macos/config.rs`)

macOS 配置中**显式拒绝**的能力：

| 配置项 | 行为 |
|--------|------|
| `HypervisorKind` | 含 `Hvf` 变体 |
| `pmem-ext2` | `Err("pmem-ext2 is unsupported on macOS")` |
| `shared-dir` (FS/9p) | `Err("shared-dir is unsupported on macOS")` |
| SCSI devices | 校验阶段拒绝 |
| Wayland devices | 校验阶段拒绝 |
| VFIO passthrough | 校验阶段拒绝 |
| vhost-scmi / vhost-user-* | 校验阶段拒绝 |
| devices 子进程模式 | 校验阶段拒绝 |

### 3.4 HVF Hypervisor 后端能力分析

#### 3.4.1 HvfVm 能力矩阵 (`hypervisor/src/hvf/vm.rs`)

| API | 实现状态 | 备注 |
|-----|----------|------|
| `HvfHypervisor::new()` | 完整 | 基于 `applevisor-sys` |
| `HvfVm::new()` | 完整 | 创建 VM、配置 IPA 空间 |
| `HvfVm::check_capability()` | 部分 | Protected: false; MemNoncoherentDMA: ENXIO |
| `HvfVm::set_memory()` | 完整 | 通过 `hv_vm_map` 映射 |
| `HvfVm::remove_memory()` | 完整 | 通过 `hv_vm_unmap` 取消映射 |
| `HvfVm::protect_memory()` | 部分 | 调用 `hv_vm_protect`，部分标志位映射 |
| `HvfVm::init_gic()` | 完整 | GICv3 分发器/CPU/redistributor MMIO 配置 |
| `HvfVm::set_gic_spi()` | 完整 | SPI 中断注入 |
| `HvfVm::get_mpstate()` / `set_mpstate()` | 完整 | VCPU MP 状态 |
| `HvfVm::register_irqfd()` | **缺失** | 无 irqfd 支持 |
| `HvfVm::dirty_log_bitmap_size()` | 部分 | 有实现但行为受限 |
| `HvfVm::get_dirty_log()` | 部分 | 脏页获取受限 |
| `HvfVm::try_clone()` | 完整 | VM 共享引用 |
| `HvfVm::get_pvclock()` | **缺失** | 无准虚拟化时钟 |
| Device memory mapping | **缺失** | 无设备 MMIO 映射优化 |

#### 3.4.2 HvfVcpu 能力矩阵 (`hypervisor/src/hvf/vcpu.rs`)

| API | 实现状态 | 备注 |
|-----|----------|------|
| `HvfVcpu::new()` | 完整 | 通过 `hv_vcpu_create` |
| `HvfVcpu::run()` | 完整 | VCPU 进入/退出循环 |
| `HvfVcpu::handle_mmio()` | 完整 | MMIO 仿真处理 |
| `HvfVcpu::handle_io()` | **桩** | 仅记录错误，无真实 PIO 处理 |
| `register_{get,set}()` | 完整 | 通用/SIMD/系统寄存器 |
| `set_immediate_exit()` | 完整 | 强制 VCPU 退出 |
| `set_signal_mask()` | **桩** | 返回 ENOSYS |
| `get_tsc_offset()` / `set_tsc_offset()` | **桩** | 返回 ENOSYS |
| `set_cpuid()` | **桩** | 返回 ENOSYS（aarch64 无需） |
| `get_hyperv_cpuid()` | **桩** | 返回 ENOSYS |
| PMU 相关 | **桩** | 返回 ENOSYS |

### 3.5 HVF IRQ Chip 分析 (`devices/src/irqchip/hvf_aarch64.rs`)

335 行实现，关键特征：

- 仅支持 **Kernel IRQ Chip** 模式（GICv3 in-hypervisor）
- SPI 投递通过 `HvfVm::set_gic_spi()`
- 不支持 Split/Userspace IRQ Chip
- IRQ 路由固定在初始化时建立（所有 SPI 映射为 GIC IRQ route）
- 不支持动态 IRQ 路由变更

### 3.6 virtmgr 中 macOS 适配点

virtmgr 的 `crosvm_unix.rs` 中有 **14 处 `cfg(target_os = "macos")` 条件编译**：

| 位置 | macOS 行为 | Linux 行为 |
|------|-----------|-----------|
| `non_windows_main.rs:122` | `setrlimit(RLIMIT_MEMLOCK)` | `prlimit(pid, RLIMIT_MEMLOCK)` |
| `crosvm_unix.rs:28` | 不使用 `pipe2` | 使用 `pipe2` (O_CLOEXEC) |
| `crosvm_unix.rs:110` | 不同的 crosvm 路径搜索顺序 | `/apex/...` 优先 |
| `crosvm_unix.rs:1039` | 强制镜像 early console 到日志 | 可选 |
| `crosvm_unix.rs:1208-1460` | 多处文件描述符传递差异 | 标准 Unix fd |
| `vsock_transport.rs:31-35` | UDS 路径 `/tmp/binder_rpc_vsock_{cid}_{port}.sock` | 真实 AF_VSOCK |
| `composite.rs:22` | macOS 上 composite disk 路径处理 | 标准路径 |

### 3.7 macOS 构建链路

`build_all.sh` 中 macOS 专属逻辑：

```
┌─────────────────────────────────────────────────┐
│ RUST_TARGET      = aarch64-apple-darwin          │
│ CROSVM_FEATURES  = hvf,default-no-sandbox,       │
│                    config-file,qcow,balloon,      │
│                    android-sparse,composite-disk, │
│                    tokio                          │
│ CROSVM_TOOLCHAIN = nightly                       │
│ CMake compiler   = Xcode clang (via xcrun)       │
│ CMake generator  = Ninja (需真实 ninja，非       │
│                    depot_tools 的 stub)           │
│ codesign         = 必须：ad-hoc 签名所有 Mach-O  │
│                    + crosvm 附加 Hypervisor       │
│                    entitlement                    │
│ Microdroid kernel = 强制校验为 arm64              │
│ 依赖版本钉死      = command-fds 0.3.0            │
│                    applevisor-sys 1.0.0           │
└─────────────────────────────────────────────────┘
```

### 3.8 当前 macOS HVF 已知桩与运行时预期

| 领域 | 说明 |
|------|------|
| 网络 / TAP | `net_util` macOS+hvf 全部为桩，VM 无网络 |
| ACPI / netlink | `devices/sys/macos_hvf.rs` 全部占位 |
| minijail | 全部为桩，运行时返回 Unsupported 或 no-op |
| signalfd / 信号 | Linux 语义近似，Darwin 行为差异需按 bug 收紧 |
| 脏页日志 | `HvfVm` 部分能力返回 ENXIO |
| 设备直通 | VFIO 等全部拒绝 |
| Wayland / GPU | 显式拒绝 |
| SCSI / pmem | 显式拒绝 |
| shared directories | 显式拒绝（FS/9p） |
| vhost 加速 | vhost-scmi/vhost-user-* 全部拒绝 |

---

## 四、Linux KVM 实现评估

### 4.1 实现规模

Linux KVM 后端使用 crosvm 主线完整实现，4 个源文件覆盖 aarch64/x86_64 两个架构。设备层完整支持 virtio、VFIO、vhost 等。

### 4.2 virtmgr Linux 适配

Linux 与 macOS 共享 `crosvm_unix.rs`，通过 `cfg(not(target_os = "macos"))` 分支处理 Linux 专有逻辑（如 `pipe2`、`prlimit`、`/dev/kvm` 设备检查）。

### 4.3 本轮 bring-up 关键修复

Linux host 从"能编"到"真实可运行"期间补充的关键修复：

1. `Threads.cpp` CC 兼容宏补齐（`_Nonnull` / `_Nullable`）
2. `ProcessState::self()` RPC-only host 正确初始化
3. `file.cpp` 纳入 host CMake（补齐 `ReadFully` / `WriteFully` 符号）
4. `libvmclient` Unix spawn 支持 `VIRTMGR_SERVICE_DIR` 持久模式
5. persistent virtmgr 的 PTY/TTY console attach
6. crosvm host feature 集补齐 `composite-disk`
7. desktop-host crosvm control socket 短路径（规避 ENAMETOOLONG）
8. `create_vm_context()` CID/RpcServer 端口重试窗口放大

### 4.4 Linux host 当前语义边界

- 仍是 desktop host runtime，非 Android 设备环境
- Permission / SELinux 不实现
- `service-status` / `stop-service` 是 wrapper 层能力，非 vm 原生命令
- host ADB bridge 依赖 guest 里确实有 adbd 并监听 vsock:5555

---

## 五、Windows WHPX 实现评估

### 5.1 实现规模

WHPX 后端约 3 源文件（`types.rs`, `vcpu.rs`, `vm.rs`，行数与 HVF 相当）。Windows 的最大特色是完整独立的传输层和 crosvm 集成方式。

### 5.2 Windows 专有架构

```
vm.exe → libvmclient (TCP loopback + CreateProcessW)
       → virtmgr.exe (RpcServer on VSOCK port)
       → crosvm_windows.rs (路径传参 + --socket 命名管道)
       → crosvm.exe (WHPX backend)
       → Microdroid guest
```

关键差异：
- **Binder RPC**: 命名管道传输（非 Unix socket）
- **vsock**: 命名管道模拟 `\\.\pipe\binder_rpc_vsock_{cid}_{port}`
- **crosvm 启动**: 命令行参数传参（非 fd 传递）
- **VM 控制**: crosvm CLI 子进程（`suspend`/`resume`/`balloon`/`balloon_stats`）
- **Console**: file-backed attach（非交互式 TTY）

### 5.3 Windows 已验证的证明点

- `build_all.bat` 可构建完整产物
- `run-microdroid` / `run-app` 到 `notifyPayloadReady`
- guest `adbd listening on vsock:5555`
- ADB `connect localhost:<port> => device`
- persistent virtmgr service + `vm list` / `vm console`
- `vm info` / `create-partition` / `create-idsig`

### 5.4 Windows 当前语义边界

- Protected VMs 不支持
- Console 是 file-backed attach
- `vm list` 需 persistent mode
- Permission / SELinux 全部 mock
- `--hypervisor hvf` 仅 macOS 支持；`--hypervisor whpx` 仅 Windows 支持

---

## 六、三平台缺陷汇总

### 6.1 共同缺陷

| 编号 | 缺陷 | 严重度 | 影响范围 | 说明 |
|------|------|--------|----------|------|
| C-01 | **无 CI/CD** | **P0** | 三平台 | 仓库没有 GitHub Actions 或任何 CI 流水线，所有构建/回归需手动执行 |
| C-02 | **CMake BUILD_TESTING 为空壳** | **P0** | 三平台 | `frameworks/native/libs/binder/CMakeLists.txt:244-247` 仅调用 `enable_testing()` 但无任何测试目标 |
| C-03 | **Protected VM 全平台缺失** | **P0** | 三平台 | Linux KVM 有能力但未接入；macOS/Windows 不支持 |
| C-04 | **Permission/SELinux 语义不统一** | P1 | 三平台 | 三平台各自 mock/bypass，无统一的 desktop host 抽象。✅ Phase 2 已解决：`desktop_host` crate 提供统一 `PermissionProvider` / `SelinuxProvider` trait，各平台使用共享 `MockPermissionProvider` / `MockSelinuxProvider` |
| C-05 | **Desktop host 代码分支散落** | P1 | 三平台 | 大量 `cfg(target_os)` 散落在业务逻辑中，无统一 trait/接口。✅ Phase 2 已解决：`DesktopHost` trait 已提取到 `libs/desktop_host/`，4 个 virtmgr 模块已完成迁移 |
| C-06 | **部分文案偏 Windows 命名** | P2 | 三平台 | 多处 "Windows host" 应统一为 "desktop host"；`payload.rs`、`debug_config.rs` 的替代源命名偏 Windows |
| C-07 | **ADB Bridge 实现三平台不同** | P1 | 三平台 | Linux 真实 vsock、macOS UDS 模拟、Windows named pipe。✅ Phase 2 已解决：bridge 双向 io::copy 核心逻辑已抽取到 `bridge.rs`，上层 `start_bridge()` 共享 |
| C-08 | **C++ binder-rpc 符号导出无验证** | P1 | 三平台 | 关键 NDK 符号（`AIBinder_*`、`ABinderProcess_*`）无自动化验证 |
| C-09 | **构建脚本无幂等性保证** | P2 | 三平台 | `build_all.sh` 中 CMake cache 清理逻辑仅在 macOS 切换 Xcode 时触发 |

### 6.2 macOS HVF 专属缺陷

| 编号 | 缺陷 | 严重度 | 影响 | 说明 |
|------|------|--------|------|------|
| M-01 | **网络完全桩化** | **P0** | VM 无网络 | `net_util/src/sys/macos_hvf/mod.rs` 仅 23 行，`TapT` 为空 trait 实现 |
| M-02 | **ACPI/netlink 完全桩化** | **P0** | Guest 无法感知宿主机事件 | `devices/src/sys/macos_hvf.rs` 仅 28 行，`acpi_event_run` 为空函数 |
| M-03 | **仅 Kernel IRQ Chip 模式** | P1 | 特定 virtio 设备不可用 | `run_hvf` 中 Split/Userspace → `bail!` |
| M-04 | **无 x86_64 支持** | P1 | Intel Mac 不可用 | HVF 后端锁定 `target_arch = "aarch64"` |
| M-05 | **minijail 全桩** | P1 | 无沙箱隔离 | 安全边界弱于 Linux KVM |
| M-06 | **脏页跟踪不完整** | P2 | 影响热迁移 | 部分操作返回 ENXIO |
| M-07 | **依赖 nightly Rust** | P2 | 构建稳定性风险 | macOS crosvm 需要 `+nightly` toolchain |
| M-08 | **依赖版本钉死** | P2 | 无法使用新版本 | `command-fds = 0.3.0`、`applevisor-sys = 1.0.0` |
| M-09 | **crosvm binary 无符号化** | P2 | 调试困难 | ad-hoc 签名且 strip 后无 DWARF |
| M-10 | **arm64 guest kernel 依赖手动管理** | P1 | 用户易出错 | `build_all.sh` 有校验但无自动获取 |
| M-11 | **Cargo workspace dependency 警告** | P2 | 潜在构建错误 | 部分 `cfg(feature = "hvf")` 的 dependency 可能不被正确选择 |
| M-12 | **vsock 使用 UDS 模拟** | P1 | 与 Android 语义不同 | `/tmp/binder_rpc_vsock_{cid}_{port}.sock` 路径 |

### 6.3 Linux KVM 专属缺陷

| 编号 | 缺陷 | 严重度 | 影响 | 说明 |
|------|------|--------|------|------|
| L-01 | 部分 linux-only 修复未回馈 macOS | P1 | macOS 可能缺失相同修复 | 如 `Threads.cpp` host 编译修复 |
| L-02 | ADB bridge 健壮性不足 | P2 | 端口冲突时错误不清晰 | guest 未监听时的错误信息不够明确 |
| L-03 | host APEX tree 准备未自动化 | P2 | 需手动准备 | `out/dist/apex_dir` 的生成/刷新不够显式 |

### 6.4 Windows WHPX 专属缺陷

| 编号 | 缺陷 | 严重度 | 影响 | 说明 |
|------|------|--------|------|------|
| W-01 | Console 是 file-backed attach | P1 | 调试体验差 | 非交互式 TTY |
| W-02 | `vm list` 需 persistent mode | P1 | 使用不便 | 非 Android 原生长驻服务模型 |
| W-03 | `vm run` raw config 受限 | P2 | Microdroid raw config 会失败 | `Failed to load payload metadata` |
| W-04 | Binder/RPC debug 输出过多 | P2 | info/list 输出噪音大 | 当前构建仍 emit debug 输出 |

---

## 七、建议

### 7.1 架构建议

#### 7.1.1 建立 Desktop Host 统一抽象层

**优先级：P1** | **涉及三平台**

当前三平台 `cfg` 分支散落在以下位置：
- `virtmgr/src/aidl.rs`：connectVsock、permission、SELinux、payload 路径
- `virtmgr/src/crosvm/crosvm_unix.rs`：macOS/Linux 分支
- `virtmgr/src/crosvm/crosvm_windows.rs`：Windows 专有
- `libvmclient/src/lib.rs`：spawn/platform fd 管理
- `libvmclient/src/spawn_unix.rs` / `spawn_windows.rs`：进程启动
- `libs/vmconfig/src/lib.rs`：路径映射

建议抽取统一的 trait 体系：

```rust
/// Desktop host capability provider - one impl per platform.
trait DesktopHost {
    /// Permission checking strategy.
    fn permission_checker(&self) -> &dyn PermissionProvider;
    /// SELinux label resolution.
    fn selinux_resolver(&self) -> &dyn SelinuxProvider;
    /// Staged APEX resolution.
    fn staged_apex_provider(&self) -> &dyn StagedApexProvider;
    /// VSOCK transport for guest communication.
    fn vsock_transport(&self, cid: u32, port: u32) -> Result<Transport>;
    /// Console attach strategy.
    fn console_attach(&self, metadata: &HostConsoleMeta) -> Result<()>;
    /// Debug policy source.
    fn debug_policy_source(&self) -> DebugPolicySource;
    /// DT overlay source.
    fn dt_overlay_source(&self) -> DtOverlaySource;
}

/// Platform-specific implementations.
struct LinuxDesktopHost { /* KVM + real vsock + real SELinux/permission */ }
struct MacOSDesktopHost { /* HVF + UDS vsock + mock permission/SELinux */ }
struct WindowsDesktopHost { /* WHPX + named pipe + mock permission/SELinux */ }
```

**预期收益**：
- 消除散落的 `cfg` 分支，业务逻辑使用统一接口
- 新能力增加时只需在 trait 加方法，各平台独立实现
- 测试 mock 也可以通过同一 trait 注入

#### 7.1.2 HVF 后端能力分级管理

**优先级：P0** | **涉及 macOS**

为每个 HVF 能力标记成熟度等级：

| 等级 | 定义 | 当前数量占比 |
|------|------|-------------|
| L0 (stub) | 编译桩，运行时必然失败或 no-op | ~40% |
| L1 (partial) | 基本可用但有语义差异或性能差距 | ~30% |
| L2 (full) | 与 KVM 行为对齐 | ~30% |

目标：Phase 1 结束时 L0 降低到 20% 以下。

### 7.2 技术建议

#### 7.2.1 macOS 网络虚拟化方案

**优先级：P0** | **涉及 macOS**

macOS 无原生 TAP 设备，两条可行路径：

**路径 A（推荐）：使用 `vmnet.framework`（Apple Network framework）**

```
crosvm (net_util)
  └── sys/macos_hvf/net.rs (新增, ~500行)
        └── vmnet.framework (macOS >= 10.15)
              ├── VMNET_INTERFACE_MODE_SHARED (NAT)
              └── VMNET_INTERFACE_MODE_BRIDGED (桥接)
```

优点：Apple 官方支持、用户态 API、稳定、NAT/桥接双模式
缺点：需要 macOS >= 10.15；需要 System Extension entitlement

**路径 B：utun socket + 用户态 bridge**

```
crosvm (net_util)
  └── sys/macos_hvf/net.rs
        └── utun interface (/dev/utun*)
              └── 用户态 TCP/IP bridge 进程
```

优点：无需特殊 entitlement
缺点：实现复杂、性能差、维护负担重

**推荐路径 A**，预计实现工作量约 500 行 Rust。

#### 7.2.2 macOS ACPI 基础实现

**优先级：P0** | **涉及 macOS**

当前 `devices/src/sys/macos_hvf.rs` 为完全空实现。建议实现最小 ACPI 事件集：

- Shutdown event（`_S5` 休眠事件）
- Reboot event（reset register）
- 可选：Battery status（供 guest 显示电源状态）

不需要完整 ACPI 解析，可以通过 FDT 注入固定事件。

#### 7.2.3 统一 ADB Bridge 实现

**优先级：P1** | **涉及三平台**

当前三平台 ADB bridge 使用不同底层传输，但上层接口 `IVirtualMachine.startHostVsockTcpBridge(hostPort, guestPort)` 一致。建议：

1. 保持 Aidl 接口不变
2. 抽取 `HostVsockTcpBridge` trait，三平台各自实现
3. 统一 bridge 生命周期管理（启动/停止/重连/日志）

### 7.3 流程建议

#### 7.3.1 CI/CD 流水线

**优先级：P0** | **涉及三平台**

仓库无任何 CI。建议逐步加入：

**第一层：构建验证**
```yaml
# .github/workflows/build.yml
jobs:
  build-linux:   # build_all.sh 全流程
  build-macos:   # build_all.sh (Apple Silicon runner)
  build-windows: # build_all.bat
```

**第二层：回归验证**
```yaml
jobs:
  regression-linux:   # run_linux_avf_regression.sh smoke
  regression-macos:   # run_macos_avf_regression.sh smoke
  regression-windows: # run_windows_avf_regression.ps1 smoke
```

**第三层：符号验证**
```yaml
jobs:
  symbol-check: # nm 白名单验证 binder-rpc 导出符号
```

#### 7.3.2 C++ 测试基础设施

**优先级：P0** | **涉及三平台**

`CMakeLists.txt` 的 `BUILD_TESTING` 是空壳。建议加入：

1. **符号导出验证测试**：调用 `nm -g` 检查 `libbinder-rpc` 导出符号是否包含必要的 `AIBinder_*` / `ABinderProcess_*` 函数
2. **RPC 最小 smoke test**：启动 `RpcServer` → 建立 `RpcSession` → 发送简单事务 → 验证响应
3. **Platform transport 单元测试**：针对命名管道（Windows）/ Unix socket（Linux/macOS）的连接建立/断开/重连

---

## 八、Phase 1 详细实施计划：HVF 能力补齐

> **时间**：第 1-4 周
> **目标**：macOS HVF 达到基本可用状态（网络/ACPI 从 L0 推到 L1，wrapper 体验改善）
> **平台侧重**：macOS
> **当前状态**：全部已完成

### Task 1.1: macOS 网络虚拟化 — vmnet.framework 后端 ✅ 已完成

| 字段 | 内容 |
|------|------|
| **ID** | PH1-01 |
| **优先级** | P0 |
| **预估工时** | 3-5 天 |
| **依赖** | 无 |
| **平台** | macOS |
| **状态** | ✅ 已完成 |
| **风险** | vmnet.framework 需要 System Extension entitlement；可能与 minijail-stub 有交互 |

**详细步骤**：

1. **调研 vmnet.framework API**
   - 确认 `VMNET_INTERFACE_MODE_SHARED` (NAT) 和 `VMNET_INTERFACE_MODE_BRIDGED` 的能力边界
   - 验证 macOS 10.15+ 上 API 可用性
   - 检查所需的 entitlement 和代码签名要求

2. **在 `net_util/src/sys/macos_hvf/` 中实现 `net.rs`**
   - 创建 `net.rs`（~400 行），实现 Apple Network framework 绑定
   - 通过 `objc` / `block` crate 或直接 FFI 调用 vmnet API
   - 实现 `Tap` trait 的具体类型 `VmnetTap`
   - 实现 `TapT` trait（`FileReadWriteVolatile + TapTCommon + TapTLinux`）

3. **更新 `net_util/src/sys/macos_hvf/mod.rs`**
   - 从桩实现切换为真实实现
   - 再导出 `VmnetTap` 和相关类型

4. **处理 entitlement 签名**
   - 更新 `scripts/macos_crosvm.entitlements` 加入 `com.apple.vm.networking`
   - 确保 `build_all.sh` 的 codesign 步骤携带更新后的 entitlement

5. **编写单元测试**
   - `VmnetTap::new()` 创建/销毁
   - 基本读写操作
   - 错误路径处理

**验收标准**：
- `cargo check -p crosvm --features hvf --target aarch64-apple-darwin` 通过
- Guest Microdroid 可获得 IP 地址
- Guest 可 ping 通宿主机
- `run_macos_avf_regression.sh` 中网络相关步骤通过

---

### Task 1.2: macOS ACPI 基础实现 ✅ 已完成

| 字段 | 内容 |
|------|------|
| **ID** | PH1-02 |
| **优先级** | P0 |
| **预估工时** | 2-3 天 |
| **依赖** | 无 |
| **平台** | macOS |
| **状态** | ✅ 已完成 — Guest-initiated shutdown/reboot 通过 PSCI 处理器在 VCPU 层处理（`PSCI_SYSTEM_OFF` → `SystemEventShutdown`，`PSCI_SYSTEM_RESET` → `SystemEventReset`）。Host-initiated ACPI 事件需要 IOKit 集成（未来工作） |

**详细步骤**：

1. **分析 Linux ACPI 实现**
   - 阅读 `devices/src/sys/linux/acpi.rs` 等文件
   - 确定 guest 通过 FDT/ACPI 表期望的事件集合
   - 区分"必须实现"和"可选优化"

2. **实现 `devices/src/sys/macos_hvf.rs` 核心逻辑**
   - 实现 `get_acpi_event_sock()`：返回真实的 shutdown/reboot 事件源
   - 实现 `acpi_event_run()`：事件分发循环
   - 注入 shutdown event (`_S5`) 和 reboot event (reset register)

3. **通过 crosvm FDT 注入 ACPI 事件信息**
   - 在 `run_hvf` 流程中配置 ACPI 事件源
   - 确保 guest 能正确解析 ACPI 表

4. **测试验证**
   - Guest 内执行 `reboot` 或 `poweroff` → VM 正确响应
   - 验证 crosvm shutdown 流程完整性

**验收标准**：
- Guest 触发 shutdown 时 crosvm 正常退出
- Guest 触发 reboot 时 crosvm 正确重启
- `run_macos_avf_regression.sh` 中 shutdown 场景通过

---

### Task 1.3: Wrapper 脚本增强 — 错误诊断与自愈

| 字段 | 内容 |
|------|------|
| **ID** | PH1-03 |
| **优先级** | P1 |
| **预估工时** | 2-3 天 |
| **依赖** | 无 |
| **平台** | macOS（可扩展到三平台） |
| **状态** | ✅ 已完成 — `validate-prereqs` 含 5 项检查 + JSON 诊断输出；`cleanup` 命令清理进程/临时文件/UDS 套接字；`run-summary.txt` 自动生成 |

**详细步骤**：

1. **增强 `validate-prereqs` 命令**
   - 检查 `kern.hv_support` 是否为 1
   - 检查 crosvm Hypervisor entitlement 是否生效
   - 检查 Microdroid kernel 是否为 arm64
   - 检查 `com.android.adbd` 是否在 APEX tree 中
   - 输出结构化诊断报告（JSON 格式），指出缺失项和修复建议

2. **增加常见失败场景一键诊断**
   - `Whpx/HVF not enabled` → 检查平台虚拟化特性状态
   - `protected VMs not supported` → 明确这是已知限制
   - `Failed to load payload metadata` → 检查 APEX tree 完整性
   - `adb connect failed` → 检查端口占用和 guest adbd 状态
   - `guest-log.txt is empty` → 提示正确的日志文件位置

3. **实现 virtmgr 残留清理**
   - `vm_macos.sh cleanup` 命令
   - 列出所有 repo-local virtmgr/crosvm 进程
   - 释放占用的 CID 和控制 socket
   - 清理临时文件和过期的 service state

4. **增强日志输出**
   - 每次运行自动生成 `run-summary.txt`
   - 包含：命令行参数、环境变量、CID 分配、关键 marker 时间线
   - 失败时自动收集最相关的日志片段

**验收标准**：
- `validate-prereqs` 在缺少 arm64 kernel 时输出明确错误
- `cleanup` 命令可清理历史残留
- 失败场景错误信息清晰可操作

---

### Task 1.4: HVF 能力矩阵文档化

| 字段 | 内容 |
|------|------|
| **ID** | PH1-04 |
| **优先级** | P1 |
| **预估工时** | 1-2 天 |
| **依赖** | 无 |
| **平台** | macOS |
| **状态** | ✅ 已完成 — `doc/HVF_CAPABILITY_MATRIX.md` 覆盖 61 个 HVF API 方法的 L0/L1/L2 分级 |

**详细步骤**：

1. **创建 `doc/HVF_CAPABILITY_MATRIX.md`**
   - 列出 `HvfVm` 每个方法的实现状态（L0/L1/L2）
   - 列出 `HvfVcpu` 每个方法的实现状态
   - 列出 HVF 支持的 `VmCap` / `HypervisorCap` 集合
   - 与 KVM 能力做逐一对比

2. **标注未实现的 API 的优先级**
   - 对每个 L0/L1 能力标注预期实现版本的依赖关系
   - 画出能力依赖图（如网络依赖 vmnet、ACPI 依赖 eventfd 等）

3. **生成 HVF API 覆盖度统计**
   - trait `Vm` 总方法数 vs HVF 已实现数
   - trait `Vcpu` 总方法数 vs HVF 已实现数
   - 饼图展示 L0/L1/L2 占比

**验收标准**：
- 文档完整覆盖所有 HVF API
- 可作为后续实现的任务清单

---

### Task 1.5: macOS arm64 guest kernel 自动获取 ✅ 已完成

| 字段 | 内容 |
|------|------|
| **ID** | PH1-05 |
| **优先级** | P1 |
| **预估工时** | 1-2 天 |
| **依赖** | 无 |
| **平台** | macOS |
| **状态** | ✅ 已完成 — `scripts/fetch_arm64_guest_artifacts.sh` 含 `--apex-tree` / `--kernel-url` / `--skip-download` 选项，支持架构校验和基于 URL 的下载；`build_all.sh` 中已集成 kernel 架构检查 |

**详细步骤**：

1. **在 `scripts/prepare_host_apex_tree.sh` 中增加 kernel 校验和提示**
   - 校验 kernel 架构（已有基础）
   - 增加 kernel 版本/构建日期信息输出
   - 对 x86 kernel 提供明确的替换步骤说明

2. **创建 `scripts/fetch_arm64_guest_artifacts.sh`**
   - 从已知源（AOSP build、预构建 URL）获取 arm64 Microdroid kernel
   - 校验 kernel SHA256
   - 解压到正确的 APEX tree 位置

3. **在 `build_all.sh` 中集成 kernel 检查**
   - 如果 APEX tree 存在但 kernel 非 arm64，输出醒目的 WARNING
   - 给出 arm64 kernel 的获取命令

**验收标准**：
- x86 kernel 场景下给出可操作的修复步骤
- arm64 kernel 来源有记录

---

### Task 1.6: macOS HVF 回归增强 ✅ 已完成

| 字段 | 内容 |
|------|------|
| **ID** | PH1-06 |
| **优先级** | P1 |
| **预估工时** | 1-2 天 |
| **依赖** | PH1-01, PH1-02 |
| **平台** | macOS |
| **状态** | ✅ 已完成 — 回归脚本含 PSCI shutdown 验证（guest payload ready 确认）、service 停止后状态验证、进程残留自动清理、超时机制、失败日志收集 |

**详细步骤**：

1. **扩展 `run_macos_avf_regression.sh`**
   - 增加网络连通性回归步骤（依赖 PH1-01）
   - 增加 shutdown/reboot 回归步骤（依赖 PH1-02）
   - 增加并发 VM 创建/销毁场景
   - 增加 `service-status` / `stop-service` 的持久性验证

2. **增加回归稳定性改进**
   - 每次步骤前自动清理残留进程
   - 增加步骤超时机制
   - 失败时自动收集所有相关日志到 tar.gz

3. **创建回归结果基线**
   - 记录首次通过的回归输出作为基线
   - 后续回归与此基线对比

**验收标准**：
- 扩展后的回归脚本覆盖网络和 ACPI 场景
- 回归结果可重现

---

### Task 1.7: C++ binder-rpc 测试基础设施 ✅ 已完成

| 字段 | 内容 |
|------|------|
| **ID** | PH1-07 |
| **优先级** | P0 |
| **预估工时** | 3-4 天 |
| **依赖** | 无 |
| **平台** | 三平台 |
| **状态** | ✅ 已完成 — `tests/host/check_symbols.sh` 符号白名单验证 + `tests/host/rpc_smoke_test.cpp` RPC 往返测试；CMake `BUILD_TESTING` 定义 `rpc_smoke_test` 和 `symbol_check` 测试目标 |

**详细步骤**：

1. **实现符号导出验证测试**
   - 创建 `frameworks/native/libs/binder/tests/host/symbol_check_test.cpp`
   - 在 `CMakeLists.txt` 中添加 `add_test()` 目标
   - 测试内容：
     - 加载 `libbinder-rpc` → `dlopen` / `dlsym`
     - 检查白名单符号（`AIBinder_*`、`ABinderProcess_*`、`AParcel_*`、`AStatus_*`）
     - 输出缺失符号报告

2. **实现 RPC 最小 smoke test**
   - 创建 `frameworks/native/libs/binder/tests/host/rpc_smoke_test.cpp`
   - 测试内容：
     - 创建 `RpcServer`（Unix socket / named pipe）
     - 创建 `RpcSession` 并连接
     - 发送简单事务（ping-pong）
     - 验证事务往返正确性

3. **更新 CMakeLists.txt**
   - `BUILD_TESTING` 不再为空
   - 添加 `rpc_smoke_test` 和 `symbol_check_test` 目标
   - 集成到 `ctest`

**验收标准**：
- `cmake --build . && ctest` 可执行两个测试
- 符号验证测试在 PR 合并前可阻断缺失符号的变更
- RPC smoke test 在三平台均通过

---

## 九、Phase 2 详细实施计划：跨平台统一

> **时间**：第 5-8 周
> **目标**：建立 desktop host 统一抽象，消除散落的 `cfg` 分支
> **平台侧重**：三平台同步
> **当前状态**：全部已完成。PH2-01（DesktopHost trait 设计实现）、PH2-02（Permission/SELinux Mock 统一）、PH2-03（ADB Bridge 统一）已完成；PH2-04（文案收敛）已完成；PH2-05（VM Console）Phase A + Phase B 已完成：双向命名管道 console（输出 `\\.\pipe\virtmgr_console_{cid}` + 输入 `\\.\pipe\virtmgr_console_input_{cid}`），`read_console()` / `write_console()` 完整实现，无需 ConPTY 或 crosvm 后端修改

### Task 2.1: Desktop Host 统一 trait 设计与实现 ✅ 已完成

| 字段 | 内容 |
|------|------|
| **ID** | PH2-01 |
| **优先级** | P1 |
| **预估工时** | 5-7 天 |
| **依赖** | PH1 完成 |
| **平台** | 三平台 |
| **状态** | ✅ 已完成 |

**详细步骤**：

1. **设计 DesktopHost trait 体系**
   - 在 `packages/modules/Virtualization/libs/` 下创建新的 `desktop_host` crate
   - 定义核心 trait：
     ```rust
     pub trait PermissionProvider: Send + Sync {
         fn check_permission(&self, package: &str, permission: &str) -> Result<()>;
     }
     pub trait SelinuxProvider: Send + Sync {
         fn check_label(&self, path: &Path, expected: &str) -> Result<()>;
     }
     pub trait StagedApexProvider: Send + Sync {
         fn list_staged(&self) -> Result<Vec<StagedApexInfo>>;
     }
     pub trait VsockConnector: Send + Sync {
         fn connect(&self, cid: u32, port: u32) -> Result<ParcelFileDescriptor>;
     }
     pub trait DebugPolicySource: Send + Sync {
         fn load(&self) -> Result<DebugConfig>;
     }
     pub trait DesktopHost: Send + Sync {
         fn permission(&self) -> &dyn PermissionProvider;
         fn selinux(&self) -> &dyn SelinuxProvider;
         fn staged_apex(&self) -> &dyn StagedApexProvider;
         fn vsock(&self) -> &dyn VsockConnector;
         fn debug_policy(&self) -> &dyn DebugPolicySource;
         fn platform_name(&self) -> &'static str;
     }
     ```

2. **实现三平台 concrete types**
   - `DesktopHostLinux`：KVM + 真实 vsock + mock permission/SELinux
   - `DesktopHostMacOS`：HVF + UDS vsock + mock permission/SELinux
   - `DesktopHostWindows`：WHPX + named pipe + mock permission/SELinux

3. **在 virtmgr 中接入 DesktopHost**
   - 修改 `aidl.rs` 中的 `VirtualizationService::init()` → 接收 `Box<dyn DesktopHost>`
   - 修改 `crosvm_unix.rs` / `crosvm_windows.rs` → 通过 DesktopHost 获取 vsock
   - 修改 `payload.rs` → 通过 DesktopHost 获取 staged APEX
   - 修改 `debug_config.rs` / `dt_overlay.rs` → 通过 DesktopHost 获取策略源

**验收标准**：
- virtmgr 不再直接依赖各平台的 mock/bypass 分支
- 新增平台支持时只需实现 DesktopHost trait
- 现有回归全部通过

---

### Task 2.2: Permission/SELinux Mock 统一 ✅ 已完成

| 字段 | 内容 |
|------|------|
| **ID** | PH2-02 |
| **优先级** | P1 |
| **预估工时** | 2-3 天 |
| **依赖** | PH2-01 |
| **平台** | 三平台 |
| **状态** | ✅ 已完成 |

**详细步骤**：

1. **实现统一的 `MockPermissionProvider`**
   - 合并 Windows 的 `VIRTMGR_MOCK_PERMISSION_ALLOWLIST` 和 Linux/macOS 的 bypass 逻辑
   - 统一配置格式：JSON file（跨平台一致）
   - 支持三种模式：`bypass`（默认）、`allowlist`（mock）、`strict`（VIRTMGR_STRICT_PARITY=1）

2. **实现统一的 `MockSelinuxProvider`**
   - 合并 `VIRTMGR_MOCK_SELINUX_LABEL_ALLOWLIST` 和 `getfilecon` 桩
   - 统一 JSON 配置格式
   - 同上的三种模式

3. **迁移现有代码到统一 mock**
   - Windows mock → 使用统一 mock provider
   - Linux bypass → 使用统一 mock provider
   - macOS bypass → 使用统一 mock provider

**验收标准**：
- 三平台使用同一套 `MockPermissionProvider` / `MockSelinuxProvider`
- 配置文件和文档中的 "Windows" 命名改为 "desktop host"

---

### Task 2.3: ADB Bridge 实现统一 ✅ 已完成

| 字段 | 内容 |
|------|------|
| **ID** | PH2-03 |
| **优先级** | P1 |
| **预估工时** | 3-4 天 |
| **依赖** | PH2-01 |
| **平台** | 三平台 |
| **状态** | ✅ 已完成 |

**详细步骤**：

1. **抽取 `HostVsockTcpBridge` trait**
   - 定义统一的 bridge 接口
   - 三平台各自实现 vsock 底层（真实 AF_VSOCK / UDS / named pipe）
   - 上层 TCP listener + 数据转发逻辑共享

2. **统一 bridge 生命周期管理**
   - bridge 创建/销毁/错误处理统一
   - bridge 日志统一输出到 `bridge-{cid}.log`
   - 端口冲突检测和诊断统一

3. **增加 bridge 可观测性**
   - 连接计数
   - 转发字节数统计
   - bridge 健康检查（心跳）

**验收标准**：
- `IVirtualMachine.startHostVsockTcpBridge()` 在三平台行为一致
- bridge 日志格式统一
- `vm_shell_*` 脚本的 ADB connect 行为一致

---

### Task 2.4: 文案与命名收敛 ✅ 已完成

| 字段 | 内容 |
|------|------|
| **ID** | PH2-04 |
| **优先级** | P2 |
| **预估工时** | 1-2 天 |
| **依赖** | PH2-01 |
| **平台** | 三平台 |
| **状态** | ✅ 已完成 — 代码中 "Windows host" 引用已替换为平台无关表述；非 Windows 专用模块的引用已清理（`vm_control/src/lib.rs`、`crosvm_windows.rs`）；Windows 专用文档内的 "Windows host" 保留作为平台描述 |

**详细步骤**：

1. **全仓库搜索 "Windows host" 并替换**
   - 代码注释中的 "Windows host" → "desktop host"（非 Windows 专用模块）
   - 错误消息中的 "Windows host" → 平台无关表述
   - 文档中的 "Windows host" → 保持或增加 "desktop host" 标注

2. **统一环境变量命名**
   - 保持现有变量名向后兼容（增加 deprecated alias）
   - 新增 `VIRTMGR_DESKTOP_*` 前缀作为推荐用法

3. **更新文档术语**
   - `HOST_WINDOWS_PORT.md` → 增加 "同样适用于 desktop host" 说明
   - 新增 `doc/DESKTOP_HOST_CONCEPTS.md` 统一术语定义

**验收标准**：
- ✅ 代码和错误消息中无散落的 "Windows host"（除 Windows 专有逻辑）
- ✅ 非 Windows 专用模块的引用已全部替换
- ✅ Windows 专用文档的 "Windows host" 保留为平台描述，不做替换

---

### Task 2.5: VM Console 行为收敛 ✅ 已完成（Phase A + Phase B）

| 字段 | 内容 |
|------|------|
| **ID** | PH2-05 |
| **优先级** | P2 |
| **预估工时** | 3-5 天 |
| **依赖** | PH2-01 |
| **平台** | Windows |
| **状态** | ✅ 全部已完成 — Phase A（命名管道输出 console）+ Phase B（双向命名管道 console）。`VmInstance` 增加了 `start_console_pipe()`、`read_console()`、`write_console()` 方法；`run_vm()` 支持 `console_pipe_path` + `console_input_pipe_path` 参数；crosvm serial 输出通过 `\\.\pipe\virtmgr_console_{cid}` 命名管道转发，输入通过 `\\.\pipe\virtmgr_console_input_{cid}` 双向通信，后台线程处理连接握手和数据缓冲。完整双向 I/O 无需 ConPTY 或 crosvm 后端修改 |

**详细步骤**：

1. **评估将 Windows console 升级为交互式的可行性**
   - 调研 Windows ConPTY API
   - 评估 `winpty` / `conpty` crate 可用性
   - 决定实现方案

2. **实现 Windows 交互式 console（如可行）**
   - Phase A ✅：命名管道 console 转发热修复（`create_console_named_pipe()` + `console_pipe_read_loop()`）
   - Phase B 🔧：完整 ConPTY 交互式 console（需 crosvm Windows 端支持 ConPTY serial 后端）

3. **统一三平台 console 接口**
   - ✅ `VmInstance::read_console()` / `write_console()` 方法定义
   - ⏳ Full ConPTY 集成需等待 crosvm 上游 serial 后端改造

**验收标准**：
- ✅ Phase A：命名管道 console 支持实时输出读取（`read_console()` 方法）
- ✅ 设计文档 `doc/WINDOWS_CONSOLE_CONVERGENCE.md` 完成
- ⏳ Phase B：完整交互式 TTY（待 crosvm 上游支持）

---

## 十、Phase 3 详细实施计划：稳定化与 CI

> **时间**：第 9-12 周
> **目标**：CI/CD 落地、回归增强、C++ 测试完善
> **平台侧重**：三平台同步
> **当前状态**：PH3-01（CI/CD 流水线 — 所有 4 个 workflow）、PH3-02（回归增强 — Linux 脚本）、PH3-03（符号验证与 stub 分级）、PH3-04（平台差异性文档）、PH3-05（构建脚本幂等性）全部已完成

### Task 3.1: CI/CD 流水线搭建 ✅ 已部分完成

| 字段 | 内容 |
|------|------|
| **ID** | PH3-01 |
| **优先级** | P0 |
| **预估工时** | 5-7 天 |
| **依赖** | PH1, PH2 核心任务完成 |
| **平台** | 三平台 |
| **状态** | ✅ 已完成：`.github/workflows/regression.yml`（三平台回归 + 结果汇总）+ `build.yml`（构建验证）+ `symbol-check.yml`（符号检查）+ `cargo-check-cross.yml`（跨平台类型检查） |

**详细步骤**：

1. **创建 `.github/workflows/build.yml` — 构建验证**
   ```yaml
   name: Build
   on: [push, pull_request]
   jobs:
     build-linux:
       runs-on: ubuntu-24.04
       steps:
         - uses: actions/checkout@v4
         - name: Install dependencies
           run: sudo apt-get install cmake ninja-build
         - name: Build all
           run: chmod +x build_all.sh && ./build_all.sh
         - name: Verify artifacts
           run: |
             test -f out/dist/linux/bin/virtmgr
             test -f out/dist/linux/bin/vm
             test -f out/dist/linux/bin/crosvm
             test -f out/dist/linux/lib/libbinder-rpc.so

     build-macos:
       runs-on: macos-15  # Apple Silicon
       steps:
         - uses: actions/checkout@v4
         - name: Select Xcode
           run: sudo xcode-select -s /Applications/Xcode.app
         - name: Build all
           run: chmod +x build_all.sh && ./build_all.sh
         - name: Verify artifacts
           run: |
             test -f out/dist/macos/bin/virtmgr
             test -f out/dist/macos/bin/vm
             test -f out/dist/macos/bin/crosvm
             test -f out/dist/macos/lib/libbinder-rpc.dylib
             codesign -v out/dist/macos/bin/crosvm

     build-windows:
       runs-on: windows-2025
       steps:
         - uses: actions/checkout@v4
         - name: Build all
           run: build_all.bat
         - name: Verify artifacts
           run: |
             if (-not (Test-Path out\dist\windows\bin\virtmgr.exe)) { throw }
             if (-not (Test-Path out\dist\windows\bin\vm.exe)) { throw }
             if (-not (Test-Path out\dist\windows\bin\crosvm.exe)) { throw }
   ```

2. **创建 `.github/workflows/regression.yml` — 回归验证**
   - smoke 级回归（validate-prereqs + info + create-partition + create-idsig）
   - 依赖构建产物（从 build workflow 传递 artifacts）
   - macOS 需要 Apple Silicon runner 且 HVF 可用
   - Linux 需要 KVM 可用（`/dev/kvm`）
   - Windows 需要 WHPX 可用

3. **创建 `.github/workflows/symbol-check.yml` — 符号验证**
   - 检查 `libbinder-rpc` 导出符号
   - 白名单：`AIBinder_*`、`ABinderProcess_*`、`AParcel_*`、`AStatus_*` 等
   - 对比上次成功构建的符号列表，输出 diff

4. **创建 `.github/workflows/cargo-check-cross.yml` — 跨平台类型检查**
   - `cargo check -p virtmgr --target aarch64-apple-darwin`
   - `cargo check -p virtmgr --target x86_64-pc-windows-gnu`
   - `cargo check -p virtmgr --target x86_64-unknown-linux-gnu`
   - 不上链接器，只做类型检查

**验收标准**：
- PR 提交自动触发构建
- 构建失败时邮件/通知告警
- 回归在合入前自动执行

---

### Task 3.2: 跨平台回归增强 ✅ 已完成

| 字段 | 内容 |
|------|------|
| **ID** | PH3-02 |
| **优先级** | P1 |
| **预估工时** | 4-5 天 |
| **依赖** | PH1 完成 |
| **平台** | 三平台 |
| **状态** | ✅ 已完成：Linux 回归脚本增加 `step_timeout()`、`collect_failure_logs()`、`--scenario-mode` / `--step-timeout` CLI 参数；CI regression workflow 支持 smoke/full 场景 |

**详细步骤**：

1. **增加异常注入场景**
   - crosvm 被 `SIGKILL` → virtmgr 检测并报告 `DEATH_REASON_UNKNOWN`
   - Binder RPC 连接断开 → libvmclient 重连
   - 磁盘空间满 → 分区创建失败的正确错误传播
   - 非法 JSON 配置 → 清晰的解析错误（非 panic）

2. **增加并发场景**
   - 同时创建 3 个 VM → CID 无冲突
   - 并发 `vm list` + `vm run-microdroid` → 无竞态
   - 快速连续启动/停止 VM → 资源正确回收

3. **增加长稳场景**
   - VM 运行 1 小时后 marker 仍在
   - 长时间 idle 后 virtmgr 仍可响应 `vm list`
   - 内存泄漏检查（重复 100 次 create/destroy VM）

4. **统一异常注入框架**
   - 脚本化的异常注入 CLI
   - 每个异常场景有独立的回归步骤
   - 回归脚本支持 `--scenario smoke|full|stress|abnormal`

**验收标准**：
- 异常场景回归在 CI 中可执行
- 并发场景无 panic 或数据竞争
- 长稳运行无明显资源泄漏

---

### Task 3.3: 符号导出验证与 stub 分级 ✅ 已完成

| 字段 | 内容 |
|------|------|
| **ID** | PH3-03 |
| **优先级** | P1 |
| **预估工时** | 3-4 天 |
| **依赖** | PH1-07 |
| **平台** | 三平台 |
| **状态** | ✅ 已完成：`windows_stubs.cpp` stub 函数已分级标注 L0/L1/L2；`frameworks/native/libs/binder/platform/STUB_LEVELS.md` 已生成，涵盖 ashmem（5 L0 + 1 L1）、native_handle（4 L2 + 2 L1）、Threads/Binder（1 L2 + 1 L1）19 个函数 |

**详细步骤**：

1. **建立符号白名单**
   - 从 `binder-ndk-sys` 的 `bindgen` 输出提取需要的 NDK 符号
   - 从 `rpcbinder` 的 FFI 接口提取需要的 RPC 符号
   - 生成 `expected_symbols.txt`

2. **实现符号检查脚本**
   - `scripts/check_exported_symbols.sh`（Unix）
   - `scripts/check_exported_symbols.ps1`（Windows）
   - 调用 `nm -g` / `dumpbin` 获取导出符号
   - 与白名单对比输出 diff

3. **Windows stub 分级标注**
   - 扫描 `windows_stubs.cpp` 中每个 stub 函数
   - 标注 `// L0: compile-only` / `// L1: runtime-safe` / `// L2: full semantics`
   - 生成 `frameworks/native/libs/binder/platform/STUB_LEVELS.md`

4. **将符号检查集成到 CI**
   - 构建后自动运行符号检查
   - 新增/删除符号需在白名单中反映

**验收标准**：
- 所有 stub 有明确的分级标注
- 符号白名单随代码变更自动更新
- CI 中符号检查阻断意外删除

---

### Task 3.4: 平台差异性文档完善 ✅ 已完成

| 字段 | 内容 |
|------|------|
| **ID** | PH3-04 |
| **优先级** | P2 |
| **预估工时** | 2-3 天 |
| **依赖** | PH2 完成 |
| **平台** | 三平台 |
| **状态** | ✅ 已完成 |

**详细步骤**：

1. **创建 `doc/PLATFORM_DIFFERENCES.md`**
   - 三平台 hypervisor 差异总览表
   - 三平台 Binder RPC 传输差异说明
   - 三平台 VM 通信差异说明
   - 三平台能力矩阵（可运行 / 桩 / 不支持）

2. **更新 CLAUDE.md**
   - 修正错误的文件路径引用
   - 补充缺失的构建命令和测试命令
   - 增加三平台能力对比简要说明

3. **创建 `doc/TROUBLESHOOTING.md`**
   - 常见构建问题及解决
   - 常见运行时问题（hypervisor 不可用、kernel 架构不匹配等）
   - 平台专属问题

**验收标准**：
- 开发者能从文档获得清晰的多平台差异认知
- CLAUDE.md 引用路径全部正确

---

### Task 3.5: 构建脚本幂等性与稳定性 ✅ 已完成

| 字段 | 内容 |
|------|------|
| **ID** | PH3-05 |
| **优先级** | P2 |
| **预估工时** | 2-3 天 |
| **依赖** | PH1 完成 |
| **平台** | 三平台 |
| **状态** | ✅ 已完成 — `build_all.sh`：`--clean` 支持、增量构建、CMake 缓存检测；`build_all.bat`：`--clean` 选项、MinGW 多路径自动检测（系统 MingW / msys64 / mingw-w64）、环境变量 `MINGW_PATH` 覆盖支持、增强错误消息（含 target 和 feature 信息）、统一错误退出码；`scripts/verify_build.sh`：独立 dist 完整性检查 |

**详细步骤**：

1. **增强 `build_all.sh` 幂等性**
   - 增加 `--clean` 选项（清理 build 目录后重建）
   - 检测 CMake cache 过期并自动重建
   - 增加增量构建支持（默认行为）
   - 在脚本顶部统一管理 CMake 环境变量，避免散落

2. **增加构建产物校验**
   - 构建完成后校验所有产物存在且非空
   - Mach-O / ELF / PE 文件头校验
   - dylib/so/dll 依赖库存在性校验

3. **增强 `build_all.bat` 错误处理**
   - `--clean` / `-c` 命令行选项支持
   - MinGW 工具链多路径自动检测（C:\workspace\mingw64, C:\tools\mingw64, C:\msys64\mingw64, C:\mingw-w64\mingw64）
   - `MINGW_PATH` 环境变量覆盖支持
   - Rust 构建失败时输出 target 和 feature 信息的详细错误消息
   - 统一错误退出码

4. **创建 `scripts/verify_build.sh`**
   - 独立于构建的验证脚本
   - 可对已有 dist 目录做完整性检查

**验收标准**：
- ✅ 连续两次 `build_all.sh` 不会产生错误
- ✅ `build_all.sh --clean` 从头构建不会失败
- ✅ `build_all.bat --clean` 可执行清理重建
- ✅ `verify_build.sh` 可检测构建产物不完整
- ✅ MinGW 多路径自动检测；环境变量覆盖支持
- ✅ 错误消息包含 target、feature 信息的详细上下文

---

## 十一、前瞻探索（Phase 4+）

> **时间**：第 13 周及以后
> **目标**：Protected VM 评估、HVF x86_64 探索、共享计算架构 PoC

### Task 4.1: Protected VM 可行性评估

| 字段 | 内容 |
|------|------|
| **ID** | PH4-01 |
| **优先级** | P2 |
| **预估工时** | 5-7 天（研究为主） |
| **平台** | 三平台 |

**研究内容**：
- Linux KVM protected VM (pKVM) 的能力边界和接入要求
- macOS HVF 的 memory protection API 是否支持类似语义
- Windows WHPX 的 protected VM 支持状态
- Microdroid pVM 固件（pvmfw）的移植可行性
- 输出：`doc/PROTECTED_VM_FEASIBILITY.md`

### Task 4.2: HVF x86_64 支持探索

| 字段 | 内容 |
|------|------|
| **ID** | PH4-02 |
| **优先级** | P2 |
| **预估工时** | 3-5 天（研究为主） |
| **平台** | macOS Intel |

**研究内容**：
- 评估 crosvm 早期版本中的 x86_64 HVF 代码是否可复用
- 评估 `applevisor-sys` 对 x86_64 `Hypervisor.framework` 的支持
- Intel Mac 的 HVF API 与 Apple Silicon 的差异分析
- 输出：`doc/HVF_X86_64_FEASIBILITY.md`

### Task 4.3: Performance Benchmark 框架 ✅ 已完成

| 字段 | 内容 |
|------|------|
| **ID** | PH4-03 |
| **优先级** | P2 |
| **预估工时** | 5-7 天 |
| **平台** | 三平台 |
| **状态** | ✅ 已完成 — `scripts/bench_vm_startup.sh`（跨平台 VM 启动时间基准，支持微秒级测量、N 次迭代、Warmup、JSON 输出）+ `doc/PERFORMANCE_BENCHMARK.md`（方法论文档） |

**研究内容**：
- VM 启动时间基准（各平台对比）
- virtio 设备吞吐量基准（磁盘/网络）
- 内存访问延迟对比
- CPU 虚拟化开销对比
- 输出：`scripts/bench_*.sh` + `doc/PERFORMANCE_BENCHMARK.md`

### Task 4.4: 共享计算架构 PoC

| 字段 | 内容 |
|------|------|
| **ID** | PH4-04 |
| **优先级** | P3 |
| **预估工时** | 待评估 |
| **平台** | 跨平台 |

**探索方向**：
- kernel 层面共享计算的可行性
- crosvm 扩展以支持跨 VM 内存共享
- 算子集成方案（virtio 自定义设备 vs FDT 注入）
- 与现有 Microdroid 架构的兼容性分析
- 输出：概念验证原型 + 设计文档

---

## 十二、关键文件索引

### 12.1 macOS HVF 核心文件

| 文件 | 说明 |
|------|------|
| `external/crosvm/hypervisor/src/hvf/mod.rs` | HVF 模块入口，导出 HvfVcpu/HvfVm/HvfHypervisor |
| `external/crosvm/hypervisor/src/hvf/vm.rs` | HVF VM 创建、内存映射、GIC 初始化 |
| `external/crosvm/hypervisor/src/hvf/vcpu.rs` | HVF VCPU 管理、寄存器、退出处理 |
| `external/crosvm/devices/src/irqchip/hvf_aarch64.rs` | HVF 内核 GICv3 IRQ 芯片 |
| `external/crosvm/devices/src/sys/macos_hvf.rs` | macOS ACPI/netlink 桩 |
| `external/crosvm/net_util/src/sys/macos_hvf/mod.rs` | macOS TAP 网络桩 |
| `external/crosvm/net_util/src/sys/macos_hvf/tap.rs` | macOS TAP 具体桩实现 |
| `external/crosvm/src/crosvm/sys.rs` | crosvm 平台分发入口 |
| `external/crosvm/src/crosvm/sys/macos.rs` | macOS 平台模块（引用 linux 复用） |
| `external/crosvm/src/crosvm/sys/macos/config.rs` | macOS crosvm 配置（HypervisorKind::Hvf 等） |
| `external/crosvm/src/crosvm/sys/linux.rs` | 主 VM 流程（含 `run_hvf` 函数） |
| `external/crosvm/base/src/sys/linux_macos/` | Linux API on macOS 兼容层 (13 文件) |
| `external/crosvm/Cargo.toml` | crosvm 根 Cargo.toml（hvf feature 定义） |

### 12.2 virtmgr 跨平台文件

| 文件 | 说明 |
|------|------|
| `packages/modules/Virtualization/android/virtmgr/src/non_windows_main.rs` | 全平台主入口 |
| `packages/modules/Virtualization/android/virtmgr/src/main.rs` | main 函数、PID/UID 平台分岔 |
| `packages/modules/Virtualization/android/virtmgr/src/os_compat.rs` | OS 兼容层（AsRawFd/pid_t） |
| `packages/modules/Virtualization/android/virtmgr/src/crosvm/mod.rs` | crosvm 模块平台分发 |
| `packages/modules/Virtualization/android/virtmgr/src/crosvm/crosvm_unix.rs` | Unix (Linux+macOS) crosvm 集成 |
| `packages/modules/Virtualization/android/virtmgr/src/crosvm/crosvm_windows.rs` | Windows crosvm 集成 |
| `packages/modules/Virtualization/android/virtmgr/src/vsock_transport.rs` | VSOCK 传输（三平台） |
| `packages/modules/Virtualization/android/virtmgr/src/aidl.rs` | AIDL 服务实现（含平台分岔） |
| `packages/modules/Virtualization/android/virtmgr/src/payload.rs` | Payload 管理（APEX staged 等） |
| `packages/modules/Virtualization/android/virtmgr/src/debug_config.rs` | Debug policy（含 JSON 替代源） |
| `packages/modules/Virtualization/android/virtmgr/src/dt_overlay.rs` | DT overlay 管理 |
| `packages/modules/Virtualization/android/virtmgr/src/platform.rs` | DesktopHost 全局加载点（`#[phase 2]` — 新增） |
| `packages/modules/Virtualization/android/virtmgr/src/bridge.rs` | TCP→vsock 桥共享核心（`#[phase 2]` — 新增） |

### 12.2a desktop_host crate（Phase 2 新增）

| 文件 | 说明 |
|------|------|
| `packages/modules/Virtualization/libs/desktop_host/Cargo.toml` | crate 依赖定义 |
| `packages/modules/Virtualization/libs/desktop_host/src/lib.rs` | 模块入口与再导出 |
| `packages/modules/Virtualization/libs/desktop_host/src/traits.rs` | DesktopHost / PermissionProvider / SelinuxProvider / VsockConnector trait 定义 |
| `packages/modules/Virtualization/libs/desktop_host/src/mock_permission.rs` | 统一 MockPermissionProvider（bypass/allowlist/strict 三模式） |
| `packages/modules/Virtualization/libs/desktop_host/src/mock_selinux.rs` | 统一 MockSelinuxProvider |
| `packages/modules/Virtualization/libs/desktop_host/src/linux.rs` | LinuxDesktopHost（AF_VSOCK vsock 连接器） |
| `packages/modules/Virtualization/libs/desktop_host/src/macos.rs` | MacOSDesktopHost（UDS vsock 连接器） |
| `packages/modules/Virtualization/libs/desktop_host/src/windows.rs` | WindowsDesktopHost（named pipe vsock 连接器） |
| `packages/modules/Virtualization/libs/desktop_host/src/unix_common.rs` | Unix fd/path 通用函数 |
| `packages/modules/Virtualization/libs/desktop_host/src/platform_selector.rs` | `ConcreteDesktopHost` 类型别名与工厂函数 |

### 12.3 libvmclient 跨平台文件

| 文件 | 说明 |
|------|------|
| `packages/modules/Virtualization/libs/libvmclient/src/lib.rs` | vmclient 主模块（平台 fd 管理） |
| `packages/modules/Virtualization/libs/libvmclient/src/spawn_unix.rs` | Unix virtmgr 启动 |
| `packages/modules/Virtualization/libs/libvmclient/src/spawn_windows.rs` | Windows virtmgr 启动 |

### 12.4 构建与脚本

| 文件 | 说明 |
|------|------|
| `build_all.sh` | Unix (Linux+macOS) 统一构建脚本 |
| `build_all.bat` | Windows 统一构建脚本 |
| `CMakeLists.txt` | 根 CMake（委托到 frameworks/native/libs/binder） |
| `frameworks/native/libs/binder/CMakeLists.txt` | binder-rpc 库构建定义 |
| `scripts/vm_macos.sh` | macOS VM wrapper |
| `scripts/vm_linux.sh` | Linux VM wrapper |
| `scripts/vm_windows.ps1` | Windows PowerShell wrapper |
| `scripts/vm_shell_macos.sh` | macOS vm_shell 风格入口 |
| `scripts/vm_shell_linux.sh` | Linux vm_shell 风格入口 |
| `scripts/vm_shell_windows.ps1` | Windows vm_shell 风格入口 |
| `scripts/run_macos_avf_regression.sh` | macOS 回归脚本 |
| `scripts/run_linux_avf_regression.sh` | Linux 回归脚本 |
| `scripts/run_windows_avf_regression.ps1` | Windows 回归脚本 |
| `scripts/check_macos_avf_markers.sh` | macOS marker 检查 |
| `scripts/check_linux_avf_markers.sh` | Linux marker 检查 |
| `scripts/check_windows_avf_markers.ps1` | Windows marker 检查 |
| `scripts/prepare_host_apex_tree.sh` | Host APEX tree 准备 |
| `scripts/bench_vm_startup.sh` | VM 启动时间基准测试（`#[phase 4]` — 新增） |
| `scripts/macos_crosvm.entitlements` | macOS crosvm Hypervisor entitlement |
| `scripts/microdroid_macos_raw.json` | macOS raw config 模板 |
| `scripts/microdroid_linux_raw.json` | Linux raw config 模板 |
| `scripts/microdroid_windows_raw.json` | Windows raw config 模板 |
| `.github/workflows/regression.yml` | 三平台 CI 回归 workflow（Phase 3 — 新增） |

### 12.5 文档

| 文件 | 说明 |
|------|------|
| `doc/MACOS_HVF_IMPLEMENTATION.md` | macOS HVF 移植实现记录 |
| `doc/MACOS_AVF_VM.md` | macOS AVF Host Bring-up 文档 |
| `doc/LINUX_AVF_VM.md` | Linux AVF Host Bring-up 文档 |
| `doc/LINUX_AVF_PARITY_REPORT.md` | Linux AVF 对齐报告 |
| `doc/WINDOWS_AVF_VM.md` | Windows AVF Host Bring-up 文档 |
| `doc/WINDOWS_AVF_PARITY_REPORT.md` | Windows AVF 对齐报告 |
| `doc/HOST_WINDOWS_PORT.md` | virtmgr Windows 主机移植说明 |
| `doc/HOST_WINDOWS_PORTING_GUIDE.md` | Windows 主机移植指南（详细版） |
| `doc/WINDOWS_PARITY_MATRIX.md` | Windows 参数一致性矩阵 |
| `doc/WINDOWS_CONSOLE_CONVERGENCE.md` | Windows VM Console ConPTY 设计方案（Phase A 命名管道 console 已实现，Phase B ConPTY 保留） |
| `doc/PLATFORM_DIFFERENCES.md` | 三平台差异总览表 |
| `doc/TROUBLESHOOTING.md` | 常见问题排查指南 |
| `doc/HVF_CAPABILITY_MATRIX.md` | HVF API 能力矩阵 |
| `doc/PERFORMANCE_BENCHMARK.md` | 性能基准测试方法论文档（`#[phase 4]` — 新增） |
| `doc/REPO_SETUP.md` | 仓库布局与同步说明 |

### 12.6 C++ binder-rpc 平台文件

| 文件 | 说明 |
|------|------|
| `frameworks/native/libs/binder/platform/OS_windows.cpp` | Windows OS 抽象 |
| `frameworks/native/libs/binder/platform/OS_macos.cpp` | macOS OS 抽象 |
| `frameworks/native/libs/binder/platform/OS_non_android_linux.cpp` | Linux OS 抽象 |
| `frameworks/native/libs/binder/platform/namedpipe_rpc_transport.cpp` | Windows 命名管道 RPC 传输 |
| `frameworks/native/libs/binder/platform/namedpipe_vsock.cpp` | Windows 命名管道 VSOCK |
| `frameworks/native/libs/binder/platform/macos_uds_vsock_path.cpp` | macOS UDS 模拟 VSOCK 路径 |
| `frameworks/native/libs/binder/platform/windows_stubs.cpp` | Windows API 桩 |
| `frameworks/native/libs/binder/platform/macos_stubs.cpp` | macOS API 桩 |
| `frameworks/native/libs/binder/OS_unix_base.cpp` | Unix 通用基类 |
| `frameworks/native/libs/binder/platform/IPCThreadState_host.cpp` | Host IPCThreadState |

---

> 文档版本：v1.3
> 最后更新：2026-04-25
> 基于工作区提交：`bfa71d1 bscp: first mac commit.`
>
> **完成度总结**：Phase 1（HVF 能力补齐）✅ 全部已完成；Phase 2（跨平台统一）✅ 全部已完成（PH2-05 Phase A + Phase B 双向命名管道 console 已实现）；Phase 3（稳定化与 CI）✅ 全部已完成
