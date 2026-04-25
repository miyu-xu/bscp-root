# macOS（Apple Silicon）+ HVF 移植实现记录

本文档记录 crosvm 在 `aarch64-apple-darwin` 上启用 `--features hvf` 时的平台选择、依赖与主流程接线情况，便于后续继续实现与排错。

**最后更新**：基于工作区当前树整理；完整编译需在 **Apple Silicon Mac** 上执行  
`cargo check -p crosvm --features hvf --target aarch64-apple-darwin`。

---

## 1. 目标与约束

- **目标**：在 macOS Apple Silicon 上迭代 `--features hvf`，使用 Apple **Hypervisor.framework**（crate 内为 `hypervisor::hvf`，依赖 `applevisor-sys`）。
- **策略**：尽量复用现有 **Linux 路径**下的 `src/crosvm/sys/linux.rs`、aarch64 arch、`devices`  virtio 等，通过 `cfg` 切换 `base::linux`、`sys`、无 KVM 的内存/网络桩，以及 **minijail-stub** 替代真实 minijail。
- **约束**：避免 Cargo 中同一包名 `minijail` 指向不同 path（与 Linux 真包冲突）；改为显式依赖 **`minijail-stub`** 包，在 Rust 里用 `#[cfg]` 在 `minijail::` 与 `minijail_stub::` 之间切换。

---

## 2. 功能开关与工作区

### 2.1 根 `Cargo.toml` 中 `hvf` feature

启用时传递依赖（节选，以仓库为准）：

- `aarch64/hvf`, `arch/hvf`, `base/hvf`, `devices/hvf`, `hypervisor/hvf`, `net_util/hvf`, `vm_memory/hvf`

### 2.2 macOS 目标依赖（根包 `crosvm`）

- `[target.'cfg(target_os = "macos")'.dependencies]` 增加 **`minijail-stub`**（`path = "minijail-stub"`），供 `gpu.rs`、`plugin/process.rs` 等直接使用 `minijail_stub::…`。

**注意**：不要在 workspace 里把 `minijail-stub` 再 `package = "minijail"` 别名，否则会与 `[patch.crates-io] minijail = { path = "third_party/..." }` 冲突。

---

## 3. 入口与平台选择

| 位置 | 行为 |
|------|------|
| `src/sys.rs` | `all(target_os = "macos", target_arch = "aarch64", feature = "hvf")` 时与 Linux 一样 `mod linux` 并 `use crate::crosvm::sys::linux::{run_config, ExitState}`。 |
| `src/crosvm/sys.rs` | 同上，并 `pub(crate) use platform::config::HypervisorKind`。 |
| 非上述 macOS+hvf | 仍为 `compile_error!("Unsupported platform")`（macOS 上需带 `hvf` 才能编）。 |

---

## 4. `base`（Linux 兼容层 on macOS）

### 4.1 Feature

- `base/Cargo.toml`：`hvf = []`（文档说明：在 macOS 上启用 Linux 风格 `base::linux` 兼容实现）。

### 4.2 模块结构

- `base/src/sys.rs`：在 `all(target_os = "macos", feature = "hvf")` 下 `mod linux_macos`，并 `pub mod linux { pub use super::linux_macos::*; }`。
- `base/src/lib.rs`：与 Linux 分支类似，对 macOS+hvf 再导出 `sys::linux` 及顶层 re-export。
- `base/src/sys/linux_macos/`：具体桩与从 Linux 复用的文件（`#[path = "../linux/..."]`），包括：
  - ioctl、syslog（转 macOS PlatformSyslog）、mmap/poll、fallocate、`file_punch_hole`、`pipe` 等
  - 无 `/proc` 的 `safe_descriptor_from_path`
  - `getrlimit`/`setrlimit`（非 prlimit64）
  - cgroup 等返回 `ENOTSUP` 或桩
  - `signal` / `signalfd`（语义近似；非 Linux 完全等价）
  - `stubs`（如 AcpiNotify、Netlink 等占位）

**后续**：在真机上以编译错误为准，补齐 Darwin 与 Linux 的 API 差异（如 `sigaction`、`open` 错误映射等）。

---

## 5. 子 crate 要点

### 5.1 `arch` / `aarch64`

- `hvf` feature 定义；macOS+hvf 下依赖 **`minijail-stub`**（包名 `minijail-stub`，代码里 `minijail_stub`）。
- `arch/src/sys.rs`：Linux **或** `(macos, hvf)` 时 `pub mod linux`。
- `arch/src/sys/linux.rs`、`aarch64/src/lib.rs`：按 `cfg` 选择 `minijail::Minijail` vs `minijail_stub::Minijail`。

### 5.2 `jail`

- `jail/Cargo.toml`：Linux/Android 用 `minijail`；**所有 macOS** 目标依赖 `minijail-stub`。
- `jail/src/helpers.rs`：macOS 使用 `minijail_stub::Minijail`。

### 5.3 `devices`

- `devices/Cargo.toml`：`hvf` 启用 `hypervisor/hvf`, `net_util/hvf`, `vm_memory/hvf`。
- `devices/src/lib.rs`：macOS+hvf 分支与 Windows 类似为“空平台块”，避免 `compile_error!`；**不**导出 Linux 专有的 VFIO/proxy/usb 等大块（仍由 `cfg(any(android, linux))` 控制）。
- `devices/src/sys.rs`：`macos_hvf` +  
  `#[path = "sys/linux/serial_device.rs"]` → 解析为 `devices/src/sys/linux/serial_device.rs`（相对 `devices/src/`）。
- `devices/src/sys/macos_hvf.rs`：ACPI/netlink 等入口桩。
- `devices/src/irqchip/`：在 `all(target_os = "macos", target_arch = "aarch64", feature = "hvf")` 下编译 `hvf_aarch64`，导出 **`HvfKernelIrqChip`**（内部调用 `HvfVm::init_gic`）。

### 5.4 `net_util`

- `net_util/src/sys.rs`：macOS+hvf → `macos_hvf`（TAP 等桩；`IFF_TAP` 等与 Linux 对齐的常量需与真机需求核对）。

### 5.5 `vm_memory`

- `hvf = ["base/hvf"]`；`guest_memory/sys.rs` 在 macOS+hvf 下复用 Linux guest_memory 路径（无 KVM 时行为依赖 hypervisor 侧映射）。

### 5.6 `hypervisor`

- `hvf` feature：`dep:applevisor-sys`（`target_os = macos` + `aarch64`）。
- `hypervisor/src/lib.rs`：`pub mod hvf` 仅在 `all(target_os = "macos", target_arch = "aarch64", feature = "hvf")`。
- 实现位置：`hypervisor/src/hvf/`（`HvfHypervisor`, `HvfVm`, `HvfVcpu` 等）。

---

## 6. `crosvm` 主 crate：Minijail 与 GPU

- `src/crosvm/sys/linux/device_helpers.rs`：`Minijail` + `type MinijailError = minijail::Error | minijail_stub::Error`。
- `src/crosvm/sys/linux.rs`：同上 `Minijail`。
- `src/crosvm/sys/linux/gpu.rs`：`MinijailCommand` = `minijail::Command` 或 `minijail_stub::Command`。
- `src/crosvm/plugin/process.rs`：`Minijail` 按平台分流（plugin 功能在 macOS 上是否可用另受 `kvm` 等 feature 约束）。

---

## 7. Hypervisor 选择与 `run_hvf`

### 7.1 配置类型

- 文件：`src/crosvm/sys/linux/config.rs`
- 在 `all(target_os = "macos", target_arch = "aarch64", feature = "hvf")` 下为 `HypervisorKind` 增加枚举变体 **`Hvf`**（serde / keyvalue 为 **kebab-case** → 命令行 **`--hypervisor hvf`**）。

### 7.2 默认 hypervisor

- 文件：`src/crosvm/sys/linux.rs` 中 `get_default_hypervisor()`  
- 在 **macOS + aarch64 + hvf** 下**直接**返回 `Some(HypervisorKind::Hvf)`（不依赖 `/dev/kvm`）。

### 7.3 `run_hvf` 流程（仅 macOS aarch64 + hvf 编译）

1. `HvfHypervisor::new()`
2. `create_guest_memory(&cfg, &components, &hvf)`（与 KVM 路径相同抽象：`impl Hypervisor`）
3. `HvfVm::new(&hvf, guest_mem, components.hv_cfg)`
4. 保护内存等：若 `protection_type.isolates_memory()` 且 VM 不支持 `VmCap::Protected`，则 `bail!`（当前 HVF 能力较保守）
5. **IRQ**：仅支持 **`IrqChipKind::Kernel`**；Split/Userspace → `bail!` 说明仅支持 kernel irqchip
6. `HvfKernelIrqChip::new(vm_clone, vcpu_count)`（内部 `init_gic`）
7. `run_vm::<HvfVcpu, HvfVm>(...)`
8. 若启用 `swap` feature，结构与 `run_kvm` 类似传入 `swap_controller`（在 macOS 默认 feature 集上可能未启用）

### 7.4 测试

- `src/crosvm/sys/linux/config.rs`：`hypervisor_hvf` 测试，`cfg(all(target_os = "macos", target_arch = "aarch64", feature = "hvf"))` 包住。

---

## 8. 已知桩与运行时预期

| 领域 | 说明 |
|------|------|
| 网络 / TAP | `net_util` macOS+hvf 多为桩，真 virtio-net/TAP 需后续实现或换 backend。 |
| ACPI / netlink | `devices` `macos_hvf` 占位。 |
| minijail | 桩在运行时多返回 `Unsupported` 或 no-op；多进程 sandbox、部分 GPU jail 路径可能不可用。 |
| signalfd / 信号 | Linux 语义近似，与 Darwin 行为差异需按 bug 收紧。 |
| 脏页日志等 | `HvfVm` 部分能力返回 `ENXIO`/不支持，与 KVM 特性不对齐。 |

---

## 9. 待办与验证清单（建议顺序）

1. **Apple Silicon 上**：  
   `cargo check -p crosvm --features hvf --target aarch64-apple-darwin`  
   修复所有编译/链接错误（Framework、`applevisor-sys`、`base::linux_macos` 等）。
2. **Cargo 警告**：部分 crate 的 `Cargo.toml` 在  
   `[target.'cfg(... feature = "hvf" ...)'.dependencies]`  
   中使用 `feature =` **可能不会按预期选依赖**（Cargo 文档建议用 `[features]` 显式传递）。若 macOS 构建缺依赖，改为 feature 拉取 `minijail-stub` / `hypervisor/hvf` 等。
3. **最小可运行路径**：固定内核 + FDT、单网卡/无网、关闭 sandbox，跑通 `run_vm` 与 vCPU exit 循环。
4. **网络**：实现或接入 macOS 上可用的 tap/utun 或 user 态网络。
5. **与 Linux 行为对齐**：IRQ、MSI、balloon、内存热插拔等按测试与需求逐项对齐。

---

## 10. 关键文件索引（快速跳转）

| 区域 | 路径 |
|------|------|
| 根 feature / macOS dep | `Cargo.toml` |
| 入口 sys | `src/sys.rs`, `src/crosvm/sys.rs` |
| VM 主流程 | `src/crosvm/sys/linux.rs`（含 `run_hvf`, `get_default_hypervisor`, `run_config`） |
| Hypervisor 枚举 | `src/crosvm/sys/linux/config.rs` |
| HVF 后端 | `hypervisor/src/hvf/` |
| HVF IRQ | `devices/src/irqchip/hvf_aarch64.rs` |
| base 兼容 | `base/src/sys.rs`, `base/src/sys/linux_macos/` |
| devices 平台 | `devices/src/sys.rs`, `devices/src/sys/macos_hvf.rs` |
| 网络桩 | `net_util/src/sys/macos_hvf/` |
| minijail 桩 | `minijail-stub/src/lib.rs` |

---

## 11. 非目标 / 说明

- 本文档**不**替代上游官方文档；仅描述本仓库中 **macOS HVF 移植**相关改动与后续方向。
- Windows / Linux 默认 CI 路径不应因 macOS 条件编译而破坏；非 macOS 目标下 `HypervisorKind::Hvf` 与 `run_hvf` 不存在于编译单元中。
