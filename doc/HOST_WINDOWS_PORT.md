# virtmgr Windows / 主机交叉编译修改说明

本文档描述在将 `virtmgr` 以 **MinGW 目标**（如 `x86_64-pc-windows-gnu`）交叉编译时所做的结构性修改，以及与 **Android 设备上真实行为** 的差异边界。

> Phase 2 更新：三平台共用的 DesktopHost 抽象层已提取到 `libs/desktop_host/` crate。WindowsDesktopHost 通过统一的 `VsockConnector` trait 提供 named pipe vsock 连接器，使用与 Linux/macOS 同一套 `MockPermissionProvider` / `MockSelinuxProvider`。详见 `libs/desktop_host/src/`。

快速导航：
- **全模块主机移植总览（含 `libvmclient` / `vm` / 环境变量 / 排错）**：`packages/modules/Virtualization/HOST_WINDOWS_PORTING_GUIDE.md`
- 参数一致性矩阵：`android/virtmgr/WINDOWS_PARITY_MATRIX.md`
- **Microdroid 与 `VirtualizationService` 的关系（guest 只连 `IVirtualMachineService`）**：见 **§1.1**

---

## 1. 背景与目标

- **目标**：在保持 **与设备相同的入口与主流程**（`main` → `non_windows_main::run()`）的前提下，使 `cargo check --target x86_64-pc-windows-gnu` 能够通过。
- **手段**：将仅适用于 Unix/Android 的依赖与代码路径用 `#[cfg(unix)]` / `#[cfg(windows)]` / `#[cfg(not(windows))]` 隔离；对无法在主机实现的子系统提供 **桩实现** 或 **明确返回错误**；将 **`command-fds`** 等依赖 Unix `Command` 预执行 API 的 crate **仅放在 Unix 目标**；**`shared_child`** 在 Unix 与 Windows 目标均可使用（用于等待/终止 crosvm 子进程）。与此同时，将 `libs/disk` 与 `libs/vmconfig` 从“仅编译桩”推进到可用的主机实现。

> 补充定位：Windows 侧以“**纯 RPC host**”为目标。对于 `binder-rpc`，优先实现 AVF host 实际链路会触达的 API 语义，不追求完整 Android 设备语义复刻。更完整评估见 `packages/modules/Virtualization/HOST_WINDOWS_PORTING_GUIDE.md` §6.1。

### 1.1 Microdroid 与 VirtualizationService 的关系（移植边界）

- **Guest（microdroid）不通过 `IVirtualizationService` / `android.system.virtualizationservice` 与设备守护进程通讯。**  
  `microdroid_manager` 在 guest 内通过 **vsock** 连接宿主机 Binder RPC，获取的是 **`IVirtualMachineService`**：宿主机在 **port = 该 VM 的 CID** 上提供该服务（见 `packages/modules/Virtualization/guest/microdroid_manager/src/main.rs` 中 `get_vms_rpc_binder()`）。
- **宿主机客户端**（例如 **`vm` CLI**）通过 **libvmclient** 连接 **virtmgr** 暴露的 **`IVirtualizationService`**（RpcBinder）；这与 guest 侧的 **`IVirtualMachineService`** 是两条不同路径。
- **设备守护进程** `packages/modules/Virtualization/android/virtualizationservice/`（`src/main.rs` 二进制）在真机上向 **virtmgr** 提供 **`IVirtualizationServiceInternal`** 等能力（对应 `virtmgr` 内 `GLOBAL_SERVICE` / `wait_for_interface("android.system.virtualizationservice")`）；**guest microdroid 不直连该进程**。
- **对 Windows 主机场景的结论**：若目标仅为 **crosvm + microdroid** 且宿主机使用已移植的 **virtmgr**，**不必** 为 guest 通路去完整移植 **`android/virtualizationservice` 二进制**。若需复现真机上 **virtmgr ↔ VirtualizationServiceInternal** 的全链路，再单独评估对 `GLOBAL_SERVICE` 相关路径的 mock/stub 或模块化。

---

## 2. 入口与进程标识（`main.rs`）

| 项目 | Unix | Windows（主机桩） |
|------|------|-------------------|
| `main` | `non_windows_main::run()` | 相同 |
| `get_this_pid` / `get_calling_pid` | `nix::Pid`（当前 / 父进程） | `libc::getpid()`（桩：父子均视为当前进程） |
| `get_calling_uid` | `libc::uid_t`（当前 UID） | `u32`，固定返回 `0`（等价“特权”桩，配合 `check_permission` 短路） |
| `pid_t` 类型 | `std::os::unix::raw::pid_t` | 见 `os_compat` |

`LazyLock` / `nix` 仅在 `#[cfg(unix)]` 下引用，避免 Windows 上未使用导入与类型不匹配。

---

## 3. 主流程（`non_windows_main.rs`）

- **名称**：模块名仍为 `non_windows_main`，但 **所有目标**（含 Windows）均执行其中的 `run()`。
- **流程概要**：`rustutils::inherited_fd::init_once` → `android_logger` → `hypervisor_props::is_any_vm_supported`（桩通常为真）→ 解析 `--rpc-server-fd` / `--ready-fd` → `ProcessState::start_thread_pool` →（Unix：`prlimit` 或 `removeMemlockRlimit`；Windows：非 `early` 时 `removeMemlockRlimit`）→ 注册 `VirtualizationService` → `RpcServer::new_unix_domain_bootstrap` → 向 `ready_fd` 写就绪字节 → `server.join()`。
- **说明**：RPC 传输在 binder-rpc 主机侧仍以 Unix 域语义命名；Windows 下由 CRT 句柄与 binder 实现承载，与 Android 套接字路径不同属预期。

---

## 4. `Cargo.toml` 依赖划分

- **全局 `[dependencies]`**：保留与业务、Binder AIDL 生成 crate、路径桩库共用的依赖。
- **`[target.'cfg(unix)'.dependencies]`**：
  - `nix`
  - `vsock`
  - **`command-fds`**（依赖 Unix `Command::pre_exec` 等，不可在 Windows 构建）
  - **`shared_child`**
- **`[target.'cfg(windows)'.dependencies]`**：
  - **`shared_child`**（启动与管理 crosvm 子进程）
  - **`windows-sys`**（`GetFinalPathNameByHandleW` 等，用于将 `File` 还原为路径传给 crosvm）

**binder / rpcbinder 路径**：相对仓库根目录，例如  
`../../../../../frameworks/native/libs/binder/rust` 与 `.../rpcbinder`（自 `android/virtmgr` 起五级 `..` 到 AOSP 根），便于 CI 与换机复现。

---

## 5. `crosvm` 模块拆分（`src/crosvm/`）

| 文件 | 作用 |
|------|------|
| `mod.rs` | `#[cfg(unix)]` 导出 `crosvm_unix::*`；`#[cfg(windows)]` 导出 `crosvm_windows::*`。 |
| `crosvm_unix.rs` | 设备侧完整实现：依赖 `nix`、`command-fds`、`SharedChild`、管道、Unix 序包控制套接字、`/proc/self/fd/...` 传参等。 |
| `crosvm_windows.rs` | **主机可执行路径**：导出与 `aidl` / `atom` 一致的公开类型；使用 **`std::process::Command` + 路径参数** 启动本机 **`crosvm.exe`**（默认 `PATH` 中的 `crosvm.exe`，或由环境变量 **`VIRTMGR_CROSVM_PATH`** 覆盖）。VM 控制通道为 **命名管道路径**（`\\.\pipe\virtmgr_crosvm_{cid}_{uuid}`），通过 **`--socket`** 传入 crosvm（由 crosvm 进程创建控制服务端）。镜像/磁盘等通过 **`GetFinalPathNameByHandleW`** 得到路径后写入命令行。 |

**Windows 上仍会在校验阶段 `bail!` 的配置**：**VFIO**、**TAP**（`cfg(network)` 且 `networkSupported` 时与 Linux `run` 的 `--net`/tap-fd 不对齐）、**`boost_uclamp`**。  
**已支持**：composite 磁盘所需的 **`indirect_files`**（分区句柄保存在 `VmInstance::keepalive_indirect_files`，路径已写入 composite spec，crosvm 按路径打开）；在启用 **`paravirtualized_devices`** 构建配置时，**GPU/display/input** 通过 `--gpu` / `--gpu-display` / `--input` 传递（与 Unix 侧 crosvm 参数形状一致；是否可用取决于本机 **Windows crosvm** 构建特性）。

**Phase 2 更新**：ADB bridge 的双向 io::copy 核心逻辑已从 `crosvm_windows.rs` 和 `crosvm_unix.rs` 抽取到共享 `src/bridge.rs`。Windows 的 `bridge_tcp_client_to_guest_vsock()` 现委托 `bridge::bridge_connection()`，并由 `crosvm_unix.rs` 复用相同入口。

这样在 Windows 上 **不再编译** Unix 专用实现；在已安装/指定 **可运行的 Windows crosvm** 的前提下，可以 **真实拉起** VM 进程。

---

## 6. 操作系统兼容层（`os_compat.rs`）

- **Unix**：重新导出 `std::os::unix` 的 `FileExt`、`AsRawFd`、`FromRawFd`、`IntoRawFd`、`ExitStatusExt`、`pid_t`。
- **Windows**：
  - `pid_t` 定义为 `libc::c_int`（MinGW 上常见做法）。
  - 为 `std::fs::File` 实现本地 **`AsRawFd` trait**：通过 `AsRawHandle` + `libc::open_osfhandle` 得到 CRT 整数 fd，供与 Binder / SELinux 桩交互的代码统一签名。

`aidl.rs` 等处使用 `crate::os_compat::pid_t`，避免 `libc::pid_t` 在 Windows `libc` crate 中缺失的问题。

---

## 7. `aidl.rs` 中与平台相关的要点

以下为 **逻辑分支** 摘要（具体行号以当前文件为准）：

- **`maybe_create_device_tree_overlay`**：
  - Unix：按 host DT + trusted/untrusted props 生成 overlay。
  - Windows：支持 `VIRTMGR_DT_OVERLAY_JSON` 作为替代输入；未设置时默认 `Ok(None)`；`VIRTMGR_STRICT_PARITY=1` 下若未提供替代输入则报错。
- **`check_permission`**：
  - Unix：走 `binder::wait_for_interface("permission")` 与 `IPermissionController`。
  - Windows：使用统一的 `MockPermissionProvider`（`libs/desktop_host/src/mock_permission.rs`）；支持 JSON 配置（`VIRTMGR_MOCK_PERMISSION_JSON`）和旧版 CSV（`VIRTMGR_MOCK_PERMISSION_ALLOWLIST{,_FILE}`）；无 mock 时默认 bypass；`VIRTMGR_STRICT_PARITY=1` 下报错。
- **`check_label_for_partition` / `check_label_for_file`**：
  - Unix：使用 `getfilecon` 进行 SELinux 检查。
  - Windows：使用统一的 `MockSelinuxProvider`（`libs/desktop_host/src/mock_selinux.rs`）；支持 JSON 配置（`VIRTMGR_MOCK_SELINUX_JSON`）和旧版 CSV（`VIRTMGR_MOCK_SELINUX_LABEL_ALLOWLIST{,_FILE}`）；无 mock 时默认 bypass；`VIRTMGR_STRICT_PARITY=1` 下报错。
- **`clone_or_prepare_logger_fd`**：无继承 fd 时 Windows 返回 `Status::new_exception_str(..., Some("..."))`；Unix 使用 `pipe` + 读线程。
- **`connectVsock`**：实现位于 **`src/vsock_transport.rs`**（Phase 2 统一通过 `desktop_host::VsockConnector` trait 转发）。Unix 使用 **`vsock`**（`VsockStream::connect_with_cid_port`）。Windows 使用 **命名管道**：路径与 C++ `frameworks/native/libs/binder/platform/namedpipe_vsock.h` 中 `NamedPipeVsockAddress` 一致，为 **`\\.\pipe\binder_rpc_vsock_{cid}_{port}`**（`CreateFileW` 客户端，结果封装为 `ParcelFileDescriptor`）。服务端须监听同名管道（例如 `NamedPipeVsockServer` / binder-rpc 主机示例）。
- **`load_app_config` 的 VM JSON 路径**：
  - Unix/Android：`/apex/com.android.virt/etc/{os_name}.json`
  - Windows：优先读取环境变量 **`VIRTMGR_MICRODROID_JSON`**；未设置时回退到 `C:/workspace/aosp/packages/modules/Virtualization/build/microdroid/{os_name}.json`
- **其它**：`check_partition_for_file` 等在 Windows 上可为 no-op。

---

## 8. `debug_config.rs`

- **`libfdt` / `OwnedFdt` / `get_fdt_prop_bool`**：仅在 **`#[cfg(unix)]`** 下参与真实 FDT 解析。
- **`DebugPolicy::from_overlay`**：
  - Windows：支持 `VIRTMGR_DEBUG_POLICY_JSON`（`log/ramdump/adb`）作为替代输入；`VIRTMGR_STRICT_PARITY=1` 下若请求 overlay 且无替代输入则报错。
  - Unix：保持从 DTBO 读属性的逻辑。
- **单元测试**：依赖真实 `.dtbo` 文件的测试放在 **`#[cfg(all(test, unix))]`** 的 `overlay_tests`；通用测试留在 `#[cfg(test)]`。

---

## 9. `dt_overlay.rs`

- **`create_device_tree_overlay`**：
  - **`#[cfg(unix)]`**：完整实现（`libfdt` API）。
  - **`#[cfg(windows)]`**：返回 **`Err(anyhow!(...))`**，防止误用；上层 `maybe_create_device_tree_overlay` 已在 Windows 提前返回，通常不会调用到。
- **模块内测试**：**`#[cfg(all(test, unix))]`**，避免 Windows 下跑依赖 FDT 的测试。

---

## 10. `payload.rs`（`prefer_staged`）

- 当 `prefer_staged` 为真时：
  - **Windows**：本地 staged provider（`VIRTMGR_STAGED_APEX_DIR` + 可选 `staged_apexes.json` + 可选 `staged_state.json`）；可选 **`VIRTMGR_MOCK_STAGED_APEX_JSON`** 指向与 `staged_apexes.json` 同结构的 JSON 数组，用于在无目录扫描时 **mock `IPackageManagerNative`** 元数据（可与目录方案合并）。
  - **非 Windows**：保留原逻辑（`early` 限制 + `getStagedApexModuleNames` / `getStagedApexInfo`）。

---

## 11. `atom.rs`

- **`get_num_cpus`**：
  - Unix：`libc::sysconf(_SC_NPROCESSORS_CONF)`。
  - Windows：`std::thread::available_parallelism()` 映射为 `Option<usize>`。

---

## 12. `selinux.rs`

- **Unix**：`use std::os::fd::AsRawFd`。
- **Windows**：`use crate::os_compat::AsRawFd`（与 `File` 上的桩实现一致）。

---

## 13. 共享库与桩的修改摘要

### 13.1 `libs/vmconfig`

- **`get_debug_level`**：`VirtualMachineConfig::RawConfig` 分支为 **`None`**，因生成后的 `VirtualMachineRawConfig` **已无** `debugLevel` 字段；调用方用 `unwrap_or(DebugLevel::NONE)`。
- **`VmConfig::load` / `to_parcelable`**：已实现 JSON 解析与 AIDL 结构映射（非桩）：
  - 支持字段：`name`、`cpu_topology`、`platform_version`、`memory_mib`、`console_input_device`、`bootloader`、`kernel`、`initrd`、`params`、`disks`、`protected`
  - `disks` 支持 `image` 与 `partitions`，并构造 `DiskImage` / `Partition`
  - 通过 `open_parcel_file` 将路径打开为 `ParcelFileDescriptor`
- **`open_parcel_file`**：按 `writable` 使用 `OpenOptions(read + write)` 打开文件（原先是只读桩行为）。

### 13.2 `libs/disk`

- 由空桩 `create_composite_disk -> Ok(())` 改为 **可生成 crosvm 兼容 composite disk**：
  - 使用与 crosvm 对齐的 `CDISK_MAGIC`（`composite_disk\x1d`）
  - 写入 protective MBR / GPT header / GPT footer（移植最小 GPT 写盘逻辑）
  - 使用 `cdisk_spec.proto` 通过 `prost` 生成并编码 `CompositeDisk` 结构
- 新增 `build.rs`（`prost-build` + `protoc-bin-vendored`），避免主机缺少 `protoc`。

### 13.3 `libs/packagemanager_aidl`

- `IPackageManagerNative` trait：`Interface` 改为 **`binder::Interface`** 全限定；`StagedApexInfo` 通过 **`super::StagedApexInfo`** 引用同模块结构体，修复子模块内路径解析错误。

### 13.4 `libs/microdroid_payload_config`

- 与 `aidl` 中构造/反序列化一致，补充字段：
  - `OsDesc`（`os.name`）
  - `hugepages`、`task: Option<Task>`、`extra_apks: Vec<ApkConfig>`（原为 `Vec<String>` 的需与 `aidl` 中 `ApkConfig { path }` 对齐）
- 提供 **`Default`** 实现供 `create_vm_payload_config` 使用。

### 13.5 `libs/libfdt`（桩）

- 为 **`FdtError`** 实现 **`Display`** 与 **`std::error::Error`**，便于 `debug_config` 使用 `anyhow::Error::msg` 等。

### 13.6 `libs/vm_control`

- **非 Windows**：保持 **stub**，气球 **stats** 返回 **`ENOTSUP`**（与设备上气球未就绪时的处理一致）。
- **Windows**：对 **`SuspendVcpus` / `ResumeVcpus` / `BalloonCommand::Adjust`** 委托为 **`crosvm` 子进程**；**`BalloonCommand::Stats`** 运行 **`crosvm balloon_stats <socket>`**，解析 stdout JSON 并递归查找 **`balloon_actual`**（与 `VmInstance::get_memory_balloon` 一致）。可执行文件由 **`VIRTMGR_CROSVM_PATH`** 或 **`crosvm.exe`** 解析。

---

## 14. 构建与验证命令

在仓库内（且已安装 `x86_64-pc-windows-gnu` target 与 MinGW 工具链）：

```bash
cd packages/modules/Virtualization/android/virtmgr
cargo check --target x86_64-pc-windows-gnu
```

成功表示当前主机侧交叉编译通过。若本机已提供可运行的 **`crosvm.exe`**（并视需要设置 **`VIRTMGR_CROSVM_PATH`**），且 VM 配置未触发 Windows 端不支持项（见第 5 节），则可在 Windows 上 **实际启动** crosvm 子进程。

Windows 快速上手（overlay / debug policy 替代源）：

- 示例文件：
  - `android/virtmgr/examples/windows_debug_policy.json`
  - `android/virtmgr/examples/windows_dt_overlay.json`
- PowerShell 环境变量（可按需覆盖为你自己的路径）：

```powershell
$env:VIRTMGR_DEBUG_POLICY_JSON="C:/workspace/aosp/packages/modules/Virtualization/android/virtmgr/examples/windows_debug_policy.json"
$env:VIRTMGR_DT_OVERLAY_JSON="C:/workspace/aosp/packages/modules/Virtualization/android/virtmgr/examples/windows_dt_overlay.json"
# 可选：显式指定本机 crosvm（否则依赖 PATH 中的 crosvm.exe）
# $env:VIRTMGR_CROSVM_PATH="C:/path/to/crosvm.exe"
```

---

## 15. 行为边界与后续可做项

| 领域 | 主机 Windows 行为 |
|------|-------------------|
| 启动 VM / crosvm | 通过 `crosvm_windows::run_vm` 启动 **`VIRTMGR_CROSVM_PATH` 或 `crosvm.exe`**；控制为命名管道 **`--socket`**；不支持项见第 5 节 |
| VM 控制（挂起/恢复/气球调节） | `libs/vm_control` 在 Windows 上通过 **crosvm CLI**（`suspend` / `resume` / `balloon` / **`balloon_stats`**）连接同一 **`--socket`** 路径；**stats** 解析 JSON 中的 **`balloon_actual`** |
| Ramdump → tombstoned | 不上传；仅记录日志（见 `crosvm_windows`） |
| DT overlay / 调试策略 overlay | 支持 JSON 替代源（`VIRTMGR_DT_OVERLAY_JSON` / `VIRTMGR_DEBUG_POLICY_JSON`）；严格模式下缺失则报错 |
| SELinux 标签检查 | 支持 mock allowlist；严格模式下无 mock 则报错 |
| `prefer_staged` APEX | Windows 本地替代（`VIRTMGR_STAGED_APEX_DIR` + 可选 `staged_apexes.json` + 可选 **`VIRTMGR_MOCK_STAGED_APEX_JSON`**） |
| 权限服务 `permission` | **`IPermissionController`**：Windows 上无服务；通过 `desktop_host::MockPermissionProvider` 统一 mock，支持 JSON（**`VIRTMGR_MOCK_PERMISSION_JSON`**）或旧版 CSV（**`VIRTMGR_MOCK_PERMISSION_ALLOWLIST{,_FILE}`**）；严格模式下无 mock 则报错 |
| UID / PID | 桩值，仅用于编译与有限逻辑 |
| `connectVsock` | 使用命名管道 `\\.\pipe\binder_rpc_vsock_{cid}_{port}`；需对端监听；VM 未运行时仍返回 “VM is not running” |
| Microdroid JSON 配置路径 | Windows 支持 `VIRTMGR_MICRODROID_JSON` 覆盖，默认回退到 `build/microdroid/{os}.json` |

**已完成/现状**：**`binder` / `rpcbinder`** 使用相对路径；**composite `indirect_files`** 与 **input**（在 `paravirtualized_devices` 下）已接线；**`balloon_stats`** JSON 解析已接入；**`IPackageManagerNative`** 可通过 **`VIRTMGR_MOCK_STAGED_APEX_JSON`** 与 **`IPermissionController`** 通过现有 permission mock 对齐主机测试。VFIO/TAP 仍无 Windows `run` 等价项（校验阶段拒绝）。

---

## 16. 参数一致性评估（索引）

参数一致性矩阵、每项源码入口、以及 P0/P1/P2 的补齐计划已拆分到：

- `android/virtmgr/WINDOWS_PARITY_MATRIX.md`

该文件作为单一事实来源（SSOT）；本文件仅保留主线架构与边界说明，避免重复维护。

---

## 17. 文件变更清单（按目录）

- `android/virtmgr/`
  - `Cargo.toml`：Unix 专用依赖迁移。
  - `src/main.rs`：PID/UID、`os_compat::pid_t`。
  - `src/non_windows_main.rs`：全目标主流程。
  - `src/os_compat.rs`：Windows `pid_t` 与 `AsRawFd`。
  - `src/crosvm/mod.rs`、`crosvm/crosvm_windows.rs`：Windows 上路径传参 + 命名管道控制 + `SharedChild` + composite **`indirect_files` keepalive**；`crosvm_unix.rs`：Unix 实现。
  - `src/aidl.rs`、`atom.rs`、`payload.rs`、`selinux.rs`、`debug_config.rs`、`dt_overlay.rs`：`cfg` 分岔与错误消息修正。
  - `src/vsock_transport.rs`：Unix vsock / Windows 命名管道映射、`connectVsock` 封装。
  - `examples/windows_debug_policy.json`、`examples/windows_dt_overlay.json`：Windows overlay / debug policy 替代源模板。
- `libs/disk`、`libs/vmconfig`、`libs/packagemanager_aidl`、`libs/microdroid_payload_config`、`libs/libfdt`：见上文各节。

---

*文档版本：与当前工作区树一致；若 AIDL 或上游 `virtmgr` 再次演进，请同步更新第 13 节与参数一致性矩阵。*
