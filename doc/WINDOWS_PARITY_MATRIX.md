# Windows Parameter Parity Matrix

本文档将 `virtmgr` 在 Windows 主机与 Android 设备上的参数一致性拆分为可执行矩阵，便于任务拆解与跟踪。

> Phase 2 更新：三平台共用的 DesktopHost 抽象层已提取到 `libs/desktop_host/` crate。Windows 权限检查通过 `MockPermissionProvider`（bypass/allowlist/strict 三模式，支持 JSON 配置文件）统一，SELinux 标签检查通过 `MockSelinuxProvider` 统一。ADB bridge 的双向 io::copy 核心已抽取到 `src/bridge.rs` 共享模块。

快速导航：
- **全模块主机移植总览**：`packages/modules/Virtualization/HOST_WINDOWS_PORTING_GUIDE.md`
- 架构与移植背景：`android/virtmgr/HOST_WINDOWS_PORT.md`

## 1) 评估口径

- **Full**: 参数格式与关键行为基本等价。
- **Partial**: 参数可用，但运行语义存在平台差异。
- **Unsupported**: 当前仅桩实现或无等价语义。

## 2) 参数一致性矩阵

| 参数/能力 | Android | Windows 主机 | 一致性 | 代码入口 |
|---|---|---|---|---|
| `name` | 原生支持 | 已支持 | Full | `libs/vmconfig/src/lib.rs`, `android/virtmgr/src/aidl.rs` |
| `memoryMib` / `memory_mib` | 原生支持 | 已支持 | Full | `libs/vmconfig/src/lib.rs` |
| `cpuTopology` / `cpu_topology` | 原生支持 | 已支持（`one_cpu`/`match_host`） | Full | `libs/vmconfig/src/lib.rs` |
| `kernel` / `initrd` / `bootloader` | 原生支持 | 已支持（PFD 打开） | Partial | `libs/vmconfig/src/lib.rs` |
| `params` | 原生支持 | 已支持 | Full | `libs/vmconfig/src/lib.rs` |
| `consoleInputDevice` | 原生支持 | 已支持 | Full | `libs/vmconfig/src/lib.rs` |
| `disks[].image` | 原生支持 | 已支持 | Full | `libs/vmconfig/src/lib.rs` |
| `disks[].partitions[]` | 原生支持 | 已支持（`label/path/writable/guid`）；composite 的 **`indirect_files`** 在 Windows 上 **keepalive**（路径写入 spec，crosvm 按路径打开组件） | Partial | `libs/vmconfig/src/lib.rs`, `android/virtmgr/src/crosvm/crosvm_windows.rs` |
| `prefer_staged` | `package_native` + apexd | `VIRTMGR_STAGED_APEX_DIR` + 可选 **`VIRTMGR_MOCK_STAGED_APEX_JSON`**（mock `IPackageManagerNative` 元数据） | Partial | `android/virtmgr/src/payload.rs` |
| `protectedVm` | 原生支持 | 参数透传 | Partial | `android/virtmgr/src/aidl.rs`, `libs/vmconfig/src/lib.rs` |
| `customConfig.vendorImage` | 原生支持 | 已支持 | Partial | `android/virtmgr/src/aidl.rs` |
| `customConfig.devices` | 原生支持 | 字段透传 | Partial | `android/virtmgr/src/aidl.rs` |
| `networkSupported` | 原生支持 | 字段透传 | Partial | `android/virtmgr/src/aidl.rs` |
| `gdbPort` | 原生支持 | 字段透传 | Partial | `android/virtmgr/src/aidl.rs` |
| `hugePages` / `boostUclamp` | 原生支持 | 字段透传 | Partial | `android/virtmgr/src/aidl.rs` |
| `connectVsock` | AF_VSOCK | 命名管道映射 | Partial | `android/virtmgr/src/vsock_transport.rs`, `android/virtmgr/src/aidl.rs` |
| 权限检查 | 权限服务 | mock allowlist / 严格模式；见 `HOST_WINDOWS_PORT.md` | Partial | `android/virtmgr/src/aidl.rs` |
| SELinux 标签检查 | 真实 SELinux | mock allowlist / 严格模式 | Partial | `android/virtmgr/src/aidl.rs`, `android/virtmgr/src/selinux.rs` |
| **启动 crosvm（`run`）** | `/apex/.../bin/crosvm` + FD 传参 | **`VIRTMGR_CROSVM_PATH` 或 `crosvm.exe`** + 路径参数 + **`--socket`** 命名管道 | Partial | `android/virtmgr/src/crosvm/crosvm_windows.rs` |
| **VM 挂起 / 恢复 / 气球调节** | `vm_control` / 序包 | Windows：**`crosvm suspend|resume|balloon|balloon_stats`**；**stats** 解析 JSON 中 **`balloon_actual`** | Partial | `libs/vm_control/src/lib.rs` |

## 3) 当前 Windows APEX 替代方案

`prefer_staged` 在 Windows 的行为：

1. 若设置 **`VIRTMGR_MOCK_STAGED_APEX_JSON`**，先加载该 JSON 数组（与 `staged_apexes.json` 同结构），可与后续步骤 **合并**。  
2. 若设置 `VIRTMGR_STAGED_APEX_DIR`，读取该目录（若仅 mock 已足够，目录可省略）。  
3. 若目录内存在 `staged_apexes.json`，按 JSON 显式映射 staged 模块。  
4. 若存在 `staged_state.json`，仅应用 `active_modules` 中列出的模块（模拟“已激活”状态）。  
5. 若不存在 `staged_apexes.json`，回退扫描目录下 `*.apex`，使用文件名 stem 作为 `moduleName`。  
6. 最后复用 `override_staged_apex` 覆盖 active APEX 条目。

路径映射（Windows）：

- `vmconfig::open_parcel_file` 会先尝试原路径；若不存在，再按前缀映射：
  - `/apex/...` -> `${VIRTMGR_APEX_ROOT}`（默认 `${VIRTMGR_ANDROID_ROOT}/apex`）
  - `/system_ext/...` -> `${VIRTMGR_SYSTEM_EXT_ROOT}`（默认 `${VIRTMGR_ANDROID_ROOT}/system_ext`）
  - `/system/...` -> `${VIRTMGR_SYSTEM_ROOT}`（默认 `${VIRTMGR_ANDROID_ROOT}/system`）
- `VIRTMGR_ANDROID_ROOT` 默认值：`C:/workspace/aosp`
- 严格一致性门禁：`VIRTMGR_STRICT_PARITY=1` 时，以下能力在 Windows 将从“warning/bypass”升级为错误：`protectedVm`、`hugePages`、`boostUclamp`、`customConfig.devices`、`networkSupported`、`permission` 检查、SELinux 标签检查
- Debug policy overlay 替代源：`VIRTMGR_DEBUG_POLICY_JSON`（JSON: `log`, `ramdump`, `adb`）
- DT overlay 替代源：`VIRTMGR_DT_OVERLAY_JSON`（Windows 下写入临时 overlay 文件供后续链路使用）
- **crosvm 可执行文件**：`VIRTMGR_CROSVM_PATH`（完整路径）；未设置时在 `PATH` 中查找 **`crosvm.exe`**（与 `crosvm_windows`、`libs/vm_control` 的 Windows 实现一致）
- 可插拔 mock provider（用于测试一致语义，Phase 2 由 `desktop_host` crate 统一提供）：
  - JSON 配置（推荐）：`VIRTMGR_MOCK_PERMISSION_JSON` → `{"mode": "bypass|allowlist|strict", "allowlist": [...]}`
  - JSON 配置（推荐）：`VIRTMGR_MOCK_SELINUX_JSON` → `{"mode": "bypass|allowlist|strict", "allowlist": [...]}`
  - 旧版 CSV（向后兼容）：`VIRTMGR_MOCK_PERMISSION_ALLOWLIST`（逗号分隔）或 `VIRTMGR_MOCK_PERMISSION_ALLOWLIST_FILE`（逐行）
  - 旧版 CSV（向后兼容）：`VIRTMGR_MOCK_SELINUX_LABEL_ALLOWLIST`（逗号分隔）或 `VIRTMGR_MOCK_SELINUX_LABEL_ALLOWLIST_FILE`（逐行）
  - 当 mock 生效时，`permission`（**`IPermissionController`** 替代）/SELinux 检查按 allowlist 判定；未命中则拒绝。
  - `VIRTMGR_STRICT_PARITY=1` 时切换到 strict 模式（所有检查均拒绝，无 silent bypass）
  - **`VIRTMGR_MOCK_STAGED_APEX_JSON`**：mock **`IPackageManagerNative`** 的 staged APEX 列表（见上文 `prefer_staged`）。

`staged_apexes.json` 示例：

```json
[
  {
    "module_name": "com.android.art",
    "disk_image_path": "C:/staged/com.android.art.apex",
    "version_code": 123456,
    "has_classpath_jars": true
  }
]
```

`staged_state.json` 示例：

```json
{
  "active_modules": [
    "com.android.art",
    "com.android.runtime"
  ]
}
```

示例模板（可直接复制修改）：

- `android/virtmgr/examples/windows_debug_policy.json`
- `android/virtmgr/examples/windows_dt_overlay.json`

PowerShell 示例：

```powershell
$env:VIRTMGR_DEBUG_POLICY_JSON="C:/workspace/aosp/packages/modules/Virtualization/android/virtmgr/examples/windows_debug_policy.json"
$env:VIRTMGR_DT_OVERLAY_JSON="C:/workspace/aosp/packages/modules/Virtualization/android/virtmgr/examples/windows_dt_overlay.json"
```

## 4) 状态

P0/P1/P2 计划项已全部落地。当前文档用于记录实现现状与运行配置入口。

**crosvm on Windows（主机）**：`virtmgr` 可在满足第 2 节矩阵与 `crosvm_windows` 校验约束的前提下，真实启动本机构建的 **`crosvm.exe`** 并管理子进程生命周期；与设备侧 Unix FD/序包传参不同，属 **Partial** 语义。详见 `HOST_WINDOWS_PORT.md` 第 5、13.6、15 节。

已完成项（对应本文件计划）：

- `P0`：Windows 下 `permission` 与 SELinux 路径增加 warning-once，避免 silent bypass。
- `P0`：`protectedVm`、`hugePages`、`boostUclamp`、`customConfig.devices`、`networkSupported` 在 Windows 增加显式运行告警。
- `P1`：`vmconfig` 增加统一路径映射层（`/apex`、`/system_ext`、`/system` 到可配置主机目录）。
- `P1`：`prefer_staged` 增加激活态模型（`staged_state.json` 的 `active_modules` 过滤）。
- `P0`：支持 `VIRTMGR_STRICT_PARITY=1` 严格门禁，将关键 `Partial` 参数升级为错误。
- `P0`：`VIRTMGR_STRICT_PARITY=1` 下，`permission` 与 SELinux bypass 升级为错误（不再静默绕过）。
- `P2`：实现可插拔 mock provider（permission / SELinux allowlist），支持环境变量或文件配置。
- `P1`：实现 Windows debug policy / DT overlay 的 JSON 替代源，并在 `VIRTMGR_STRICT_PARITY=1` 下无替代源时报错。
- **Windows crosvm 集成**：路径传参 + 命名管道 `--socket` + `SharedChild`；`vm_control` 委托 CLI（挂起/恢复/气球调节）；composite `indirect_files` 等待后续打通。

