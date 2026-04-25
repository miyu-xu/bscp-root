# Windows AVF 与 Android AVF 功能对齐评估报告

> 评估时间：2026-04-17  
> 评估对象：本仓库当前 **Windows x86_64 + WHPX** 主机路径  
> 目标问题：**目前 Windows AVF 功能和 Android 中 AVF 功能对齐多少，差在哪里，已经验证到了什么程度**

---

## 1. 结论摘要

先给结论：

1. **如果评估口径是“本仓库当前面向 Windows 主机的 AVF 开发/联调工作流”**，我认为当前对齐度大约是 **70%~75%**。  
   这个口径下，核心原因是：**unprotected Microdroid 已经真实跑通，`run-microdroid` / `run-app` / `vm_shell` / ADB attach / 日志链路都可用**，Windows 已不再只是“能编译”，而是可以实际联调。

2. **如果评估口径是“Android 设备侧完整 AVF 能力”**，我认为当前对齐度大约是 **45%~55%**。  
   这个口径下，Windows 仍然明显落后，因为 **protected VM、设备侧守护进程语义、真实权限/SELinux 语义、AF_VSOCK 原生语义、完整 Soong/APEX/atest 体系、console 路径、跨进程长期服务模型** 还没有做到与 Android 设备一致。

换句话说：

- **作为 Windows 主机版 AVF bring-up / host-runtime / Microdroid 联调环境：已经可用。**
- **作为 Android 原生 AVF 的完整等价实现：还远未完全对齐。**

---

## 2. 评估口径与打分方法

为了避免“一个百分比覆盖所有语义”导致误判，这里采用两层口径：

### 2.1 口径 A：Windows 主机工作流对齐度

这层口径只问一个问题：

> 在 Windows 上，开发者是否已经能完成本仓库最关键的 AVF host 侧工作流？

重点看：

- `build_all.bat` 是否可构建完整产物
- `vm.exe -> libvmclient -> virtmgr -> crosvm -> guest` 是否跑通
- `run-microdroid` / `run-app` 是否到 `notifyPayloadReady`
- `vm_shell` 风格启动与 ADB attach 是否可用
- 日志、调试、运行时目录是否可维护

### 2.2 口径 B：Android 完整 AVF 能力对齐度

这层口径问的是：

> Windows 当前实现是否已经在行为上接近 Android 设备上的 AVF？

这里会额外考察：

- protected VM / pVM
- Android 设备守护进程与系统服务语义
- 真实 permission / SELinux / package manager / apexd 语义
- 真正的 AF_VSOCK / adb forward 模型
- `atest` / APEX 安装 / Soong 构建 / 设备部署链路
- console、list、长生命周期服务模型

### 2.3 状态定义

| 状态 | 含义 |
| --- | --- |
| **Full** | 关键行为已经基本与 Android 目标一致，或在 Windows 目标上已完整可用 |
| **Partial** | 能跑，但行为、实现、限制或运维方式与 Android 仍有明显差异 |
| **Missing** | 当前缺失，或关键功能仍不可用 |
| **N/A** | Android 设备能力，不属于 Windows host 目标范畴 |

---

## 3. 当前 Windows AVF 的真实定位

从仓库文档和代码来看，Windows 路径的真实定位已经非常明确：

- `packages\modules\Virtualization\HOST_WINDOWS_PORTING_GUIDE.md`
- `packages\modules\Virtualization\android\virtmgr\HOST_WINDOWS_PORT.md`
- `packages\modules\Virtualization\android\virtmgr\WINDOWS_PARITY_MATRIX.md`

这些文件共同指向一个现实结论：

> **Windows 目标不是“Android 设备语义的 1:1 复刻”，而是“纯 RPC host + 可实际运行 Microdroid 的主机移植”。**

这一定义很重要，因为它决定了哪些差异应视为“未完成”，哪些差异应视为“设计上允许的平台差异”。

当前 Windows 路径的主链路是：

```text
vm.exe -> libvmclient -> virtmgr.exe -> Binder RPC -> crosvm.exe -> guest
```

这条链路现在已经不是停留在编译层面，而是已经实跑到：

- `notifyPayloadStarted`
- `notifyPayloadReady`
- guest `adbd listening on vsock:5555`
- `adb connect localhost:<port> -> device`

因此当前状态应被定义为：

> **“Windows AVF host bring-up 已经完成核心跑通，但还没有达到 Android 设备侧完整语义对齐。”**

---

## 4. 总体功能对齐矩阵

## 4.1 核心能力总览

| 能力域 | Android AVF | Windows 当前状态 | 对齐度 | 结论 |
| --- | --- | --- | --- | --- |
| 构建链路 | Soong + APEX + host/device 体系 | `build_all.bat` 已能产出 `binder-rpc + virtmgr + vm + crosvm` | **Partial** | Windows host 构建可用，但不是 Android 原生构建体系 |
| Binder RPC 主控链路 | 原生 Binder / Binder RPC | Windows named-pipe Binder RPC 路径可用 | **Full（对主机目标）/ Partial（对 Android 语义）** | 对当前 host 目标已经足够 |
| `vm` CLI 主流程 | 原生支持 | `run-microdroid` / `run-app` / `info` / `check-feature-enabled` / `create-partition` / `create-idsig` 可用，`list` / `console` 已在 Windows persistent mode 打通 | **Partial** | 大部分可用，但 `console` 是 file-backed attach，`list` 仍是 host-side 持久化语义 |
| `run-microdroid` | Android 原生主路径 | 已实跑到 `notifyPayloadReady` | **Full（对 host bring-up）** | 这是当前 Windows 最关键的成功点 |
| `run-app` | Android 原生主路径 | 已实跑到 `notifyPayloadReady` | **Full（对 host bring-up）** | APK payload 路径已打通 |
| `run` 原始配置 | Android 原生支持 | 可启动 raw-config VM，但 Microdroid raw config 仍会因 payload metadata 失败 | **Partial** | 能启动，但不能替代 payload-ready 路径 |
| `vm_shell.sh` 风格工作流 | Android 上通过 `adb forward` + vsock | Windows 已有 `vm_shell_windows.ps1` / `.bat` | **Partial** | 使用体验接近，但底层不是同一实现模型 |
| ADB attach | Android 上 `adb forward tcp:8000 vsock:<cid>:5555` | Windows 通过 `IVirtualMachine.startHostVsockTcpBridge()` + `virtmgr` 内桥接已达到 `device` | **Partial 到 Full** | 功能已可用，但实现是 Windows 特化方案 |
| APEX / runtime 布局 | Android `/apex` + `/system` + `apexd` | 已改为直接消费 `out\dist\apex_dir` 恢复树，并生成 `apex\apex-info-list.xml` 与 `apex\decompressed\*.apex` 缓存 | **Partial** | 行为上够用，但仍是 host-side 替代方案 |
| protected VM / pVM | Android AVF 核心能力之一 | Windows `x86_64 + WHPX` 仍不支持 | **Missing** | 这是当前最大功能缺口 |
| 权限/SELinux/系统服务 | Android 真实服务与策略 | Windows 依赖 mock / warning / strict parity 门禁 | **Partial** | 不是设备等价语义 |
| `vm list` | Android 上依赖持久服务状态 | Windows 现在支持通过 persistent `virtmgr` 服务复用状态 | **Partial** | 能跨命令看到已注册 VM，但仍是 host-side opt-in 语义 |
| `vm console` | Android/Linux 可用 | Windows 现在支持 file-backed `vm console` attach | **Partial** | 可用，但不是原生 TTY 对等实现 |
| suspend/resume/balloon | Android 有现成控制路径 | Windows 通过 `crosvm suspend/resume/balloon/balloon_stats` CLI 代理 | **Partial** | 可接线，但不是原生对等实现 |
| 网络/VFIO/TAP/复杂设备语义 | Android/crosvm 原生支持 | Windows 对这些能力仍明显受限 | **Partial / Missing** | 代码路径大量警告或门禁 |
| 日志与调试 | Android 依赖 logcat / atest / host logs | Windows 已形成较完整 run-dir 日志体系 | **Full（对 host 目标）** | 这部分已经非常实用 |

---

## 5. 两个“对齐度数字”的详细解释

## 5.1 Windows 主机工作流对齐度：**约 70%~75%**

这个数字高于很多“移植还在 bring-up 期”的项目，原因是以下关键项已经真正可用：

### 已基本完成的部分

1. **Windows host 构建链路可用**
   - `build_all.bat`
   - 产物落到 `out\dist\windows`

2. **Binder RPC 主控链路可用**
   - `vm.exe`
   - `libvmclient`
   - `virtmgr.exe`
   - `crosvm.exe`

3. **Unprotected Microdroid 真实可启动**
   - `run-microdroid`
   - `run-app`
   - 已到 `notifyPayloadReady`

4. **ADB 现在已经真正可用**
   - `vm_shell_windows.ps1 -Command start-microdroid -AutoConnect`
   - `adb connect localhost:<port> => connected`
   - `adb -s localhost:<port> get-state => device`

5. **日志与调试工作流已经成型**
   - 单次 run 的 `LogDir`
   - `virtmgr-trace.log`
   - `vm-run-*.log`
   - `vmclient-trace.log`
   - `temp\virtmgr\<cid>\crosvm-stderr.txt`
   - `adb-connect.log`

### 仍然拖低评分的部分

1. `run` 原始配置路径仍不够完整  
2. `vm list` 仍不是 Android 原生 system-service 语义，而是 Windows host-side persistent mode  
3. `vm console` 是 file-backed attach，不是原生 TTY  
4. protected VM 缺失  
5. 权限/SELinux/设备能力仍是 host 侧替代语义

所以，**Windows 已经不是“能编译”阶段，而是“可用于 host bring-up、payload 联调、ADB 联调”的阶段**。  
这就是为什么这里给到 **70%~75%**，而不是 30% 或 90%。

## 5.2 Android 完整 AVF 能力对齐度：**约 45%~55%**

这个数字明显更低，因为 Android AVF 的完整能力远不止“把一个 unprotected Microdroid 跑起来”。

把 Android 设备侧完整能力纳入后，Windows 仍然缺很多关键语义：

1. **protected VM / pVM 缺失**
2. **不是 Android 设备上的真实 `virtualizationservice` / system service 形态**
3. **不是原生 AF_VSOCK + `adb forward` 语义**
4. **permission / SELinux / package manager / apexd 都不是设备真实实现**
5. **console 不是原生交互式 TTY，而是 Windows file-backed attach**
6. **`list` 已支持 persistent host service，但仍不是 Android 原生长期 system-service 模型**
7. **Soong / APEX 安装 / 设备部署 / `atest` 不是 Windows 的主路径**
8. **TAP / VFIO / 复杂设备/网络语义仍不完整**

所以，如果有人问：

> “Windows 现在是不是已经等价于 Android AVF 了？”

答案应该是：

> **还没有。它已经是可运行的 Windows host 版本，但不是 Android 设备侧 AVF 的完整等价实现。**

---

## 6. Android 与 Windows 架构差异：哪些是“合理差异”，哪些是“还没补齐”

## 6.1 属于合理平台差异的部分

这些差异不应简单判为 bug：

### 1. Binder RPC 传输实现不同

- Android/Linux：更接近 Unix socket / fd 语义
- Windows：named pipe、HANDLE、CRT fd 桥接

只要上层 `IVirtualizationService` / `IVirtualMachine` 调用链可用，这种差异是允许的。

### 2. ADB attach 的内部实现不同

Android `vm_shell.sh` 的核心逻辑是：

```bash
adb forward tcp:8000 vsock:${cid}:5555
adb connect localhost:8000
```

Windows 当前则是：

```text
vm.exe -> libvmclient -> IVirtualMachine.startHostVsockTcpBridge()
       -> virtmgr 在本机监听 localhost:<port>
       -> ConnectVsock
       -> 本地 TCP <-> guest pipe bridge
       -> adb connect localhost:<port>
```

从“行为结果”看已经对齐；从“内部机制”看仍是 Windows 专用实现，因此更适合标成 **Partial 到 Full**。

### 3. `/apex` / `/system` 路径映射

Windows 不可能真的拥有 Android 设备上的挂载体系，所以现在的做法是：

- `VIRTMGR_APEX_ROOT = out\dist\apex_dir\apex`
- `VIRTMGR_SYSTEM_ROOT = out\dist\apex_dir\system`
- `VIRTMGR_SYSTEM_EXT_ROOT = out\dist\apex_dir\system_ext`
- 只对 `.capex` 生成 `apex\decompressed\*.apex` 缓存

这属于**平台必要差异**，不是失败。

## 6.2 仍然属于“未补齐”的差异

### 1. protected VM

这是当前最明确的功能缺口，不是平台差异可以掩盖的。

已知结论：

- `WINDOWS_AVF_VM.md` 明确写出 **Protected VMs: not supported**
- 文档中有明确 marker：`protected VMs not supported on x86_64`

因此：

> **Windows 当前只能作为 unprotected Microdroid host 路径，不能视为 Android AVF 完整能力。**

### 2. `vm console`

文档中已明确：

- `console` | supported in persistent mode | Windows `vm.exe console` now attaches to registered file-backed host console metadata

这也是功能缺口，而不是设计差异。

### 3. 长生命周期服务语义

Android 下 `vm list` 的语义依赖持久存在的服务状态；  
Windows 现在已经支持 **persistent `virtmgr` host service mode**：

- `run-microdroid -PersistVirtmgr` 会在 guest 到 `READY` 后返回
- `virtmgr-service.state` 记录 `pid` / `rpc_port`
- 后续 `vm list` / `vm console` 会复用同一个 Windows host service

这意味着：

- 对 Windows host bring-up / debug 工作流已经足够
- 但它仍是 Windows host-side 的替代语义，不是 Android 设备上的原生 system service

---

## 7. 代码与文档证据

## 7.1 Windows 已跑通 unprotected Microdroid

证据来源：

- `WINDOWS_AVF_VM.md`
- 运行日志目录：`out\dist\logs\windows-vm-shell-restored-apex-8025`

已验证 marker：

- `notifyPayloadStarted`
- `notifyPayloadReady`

这些 marker 是 Windows bring-up 是否成功的最关键证据。

## 7.2 Windows ADB 已从“offline”变为“device”

证据来源：

- `out\dist\logs\windows-vm-shell-restored-apex-8025\adb-connect.log`

已观察到：

- `adb connect localhost:8025 => connected to localhost:8025`
- `adb -s localhost:8025 get-state => device`

这说明：

> **Windows 侧 ADB attach 已经不再只是 guest adbd 启动，而是 host-to-guest 数据面也真正打通了。**

## 7.3 Windows ADB 不是脚本桥，而是代码桥

关键证据：

- `packages\modules\Virtualization\android\virtualizationservice\aidl\android\system\virtualizationservice\IVirtualMachine.aidl`
  - `void startHostVsockTcpBridge(int hostPort, int guestPort);`
- `packages\modules\Virtualization\libs\libvmclient\src\lib.rs`
  - 调用 `startHostVsockTcpBridge(...)`
- `packages\modules\Virtualization\android\virtmgr\src\aidl.rs`
  - Windows 上实现该方法

因此当前 ADB 工作模式已经是：

> **in-process code bridge，而不是外部 PowerShell 脚本桥。**

## 7.4 `vm_shell.sh` 对等实现已经存在

Android 原始脚本：

- `packages\modules\Virtualization\android\vm\vm_shell.sh`

Windows 对应脚本：

- `scripts\vm_shell_windows.ps1`
- `scripts\vm_shell_windows.bat`

已对齐的命令面：

- `connect`
- `start-microdroid`
- `help`

但底层实现不是 1:1：

- Android 用 `adb forward tcp:8000 vsock:<cid>:5555`
- Windows 用本地 TCP 端口 + `virtmgr` bridge

所以这里应评为 **行为对等，机制部分对等**。

## 7.5 APEX 恢复树现在已经按用户期望使用

当前行为已改成：

- 保持 `com.android.virt` 位于 `out\dist\apex_dir\apex\com.android.virt`
- 生成 `out\dist\apex_dir\apex\apex-info-list.xml`
- 仅对 `.capex` 生成 `out\dist\apex_dir\apex\decompressed\*.apex`

这比之前“整理 / 平铺 apex_dir”更接近 Android 目录组织，因此：

- 对运行所需语义：**提升了对齐度**
- 对 Android 原生挂载语义：**仍然只是 host-side 替代**

---

## 8. 命令层面的对齐评估

以下表格更接近开发者实际使用体验。

| 命令/工作流 | Android | Windows 当前状态 | 对齐度 | 说明 |
| --- | --- | --- | --- | --- |
| `validate-prereqs` | Android 上无完全同名概念，但可类比前置检查 | 已可用 | **Full（Windows 目标）** | 适合 Windows 主机 |
| `run-microdroid` | 原生主路径 | 已可用 | **Full（host 目标）** | 已到 payload ready |
| `run-app` | 原生主路径 | 已可用 | **Full（host 目标）** | APK payload 路径已通 |
| `run` | 原生支持 | 部分可用 | **Partial** | raw config 能起机，但 Microdroid raw config 仍受限 |
| `info` | 原生支持 | 已可用 | **Full（host 目标）** | 能输出 host 状态 |
| `list` | 原生支持 | persistent mode 下可用 | **Partial** | 仍不是 Android 原生长期守护进程模型 |
| `check-feature-enabled` | 原生支持 | 已可用 | **Full（host 目标）** | 纯查询类 |
| `create-partition` | 原生支持 | 已可用 | **Full（host 目标）** | 已验证 |
| `create-idsig` | 原生支持 | 已可用 | **Full（host 目标）** | 已验证 |
| `console` | 原生支持 | persistent mode 下可用 | **Partial** | Windows 实现为 file-backed attach |
| `vm_shell connect` | 原生支持 | 已可用 | **Partial** | 用户体验接近，底层模型不同 |
| `vm_shell start-microdroid --auto-connect` | 原生支持 | 已可用 | **Partial 到 Full** | 结果对齐，机制为 Windows 特化 |

---

## 9. 运行时、APEX、guest payload 语义的对齐情况

## 9.1 已完成的关键点

### 1. `com.android.adbd` 已进入 payload

Windows 现在不再只是“guest 启动了但是没法 adb”，而是已经补上了：

- `com.android.adbd`
- `apex-info-list.xml`
- `.capex -> .apex` 运行时缓存

这直接决定 guest 是否会出现：

- `adbd listening on vsock:5555`

### 2. `com.android.virt` 路径已改回 AOSP 风格

现在脚本不会再把 `com.android.virt` 整理到错误位置，而是直接消费：

- `out\dist\apex_dir\apex\com.android.virt`

这对 initrd / kernel / etc / app 路径解析非常关键。

## 9.2 仍未完全对齐的点

Windows 上并没有 Android 的真实：

- apexd
- package manager native
- 系统挂载命名空间

当前对应的是 host 侧可运行替代：

- 路径映射
- `apex-info-list.xml` 生成
- staged APEX mock/provider

因此这里应判为 **Partial**，不是 **Full**。

---

## 10. 安全、权限与系统语义对齐情况

这一块是当前“看起来能跑，但其实和 Android 差很多”的重点区域。

## 10.1 permission 语义

Android：

- 依赖真实 `IPermissionController`

Windows：

- 支持 `VIRTMGR_MOCK_PERMISSION_ALLOWLIST`
- 支持 `VIRTMGR_MOCK_PERMISSION_ALLOWLIST_FILE`
- `VIRTMGR_STRICT_PARITY=1` 下可从 warning/bypass 升级为错误

结论：

- 对开发联调：够用
- 对 Android 真语义：**仍然只是部分对齐**

## 10.2 SELinux 语义

Android：

- 真实 `getfilecon` / 标签检查

Windows：

- `VIRTMGR_MOCK_SELINUX_LABEL_ALLOWLIST`
- `VIRTMGR_MOCK_SELINUX_LABEL_ALLOWLIST_FILE`
- strict parity 下升级为错误

结论：

- 这不是 Android 等价实现
- 只能算 **Partial**

## 10.3 设备/网络能力

文档和代码已经明确：

- `customConfig.devices` 在 Windows 上会 warning，strict parity 下会报错
- `networkSupported` 同样不是 Android 等价语义
- VFIO/TAP 等复杂路径仍不完整

所以设备能力这一块不应高估。

---

## 11. protected VM：当前最大的功能断点

如果只选一个“Windows 与 Android AVF 现在最本质的不对齐点”，那就是：

> **protected VM 仍不支持。**

这意味着什么？

1. 当前 Windows AVF **不能等价替代 Android pVM 开发环境**
2. Windows 目前更准确的定位是：
   - **unprotected Microdroid bring-up / host-side development runtime**
   - 不是完整 AVF security model runtime

这项能力不补齐，Windows 版本就永远不能说“和 Android AVF 基本对齐”。

---

## 12. 日志与调试体系：这是当前 Windows 路径最成熟的部分之一

从工程可维护性角度看，Windows 这部分已经做得相当完整。

## 12.1 当前日志组织的优点

现在一次运行的中间文件和日志都能聚合在一个目录下，例如：

```text
out\dist\logs\windows-vm-shell-restored-apex-8025\
  adb-connect.log
  guest-log.txt
  virtmgr-trace.log
  vm-run-stdout.tmp.log
  vm-run-stderr.tmp.log
  vmclient-trace.log
  vm-console.txt
  vm-console-in.txt
  work\
  temp\
```

这比 Android 上跨 logcat、host logs、atest zip、device shell 的分散排查更适合当前 Windows bring-up 阶段。

## 12.2 最重要的判定文件

当前建议的排查优先级是：

1. `virtmgr-trace.log`
2. `vm-run-<command>.log`
3. `vmclient-trace.log`
4. `temp\virtmgr\<cid>\crosvm-stderr.txt`
5. `adb-connect.log`
6. `temp\virtmgr\<cid>\guest-virtio-console*.txt`

这意味着：

> **Windows 在“可调试性”上已经达到可持续开发状态。**

这一项我会给较高评价。

---

## 13. 目前最准确的功能定位

综合所有事实，当前 Windows AVF 最准确的定位不是：

- “只是编译通过”
- 也不是“完全等价 Android AVF”

而是：

> **一个已经能在 Windows 上真实启动 unprotected Microdroid、跑 APK payload、附着 ADB、并具备较完整日志/调试链路的 AVF host 运行时。**

这是一个非常具体、也非常有价值的阶段。

---

## 14. 建议对外使用的结论表述

如果你需要向团队、README、issue、里程碑同步现状，建议直接使用下面这段表述：

> 当前仓库中的 Windows AVF 路径已经完成核心 host bring-up：  
> `vm.exe -> libvmclient -> virtmgr.exe -> Binder RPC -> crosvm.exe -> Microdroid` 已可真实运行，`run-microdroid`、`run-app`、`vm_shell` 风格启动与 ADB attach 均已打通。  
> 但 Windows 仍主要定位为 **unprotected Microdroid 的 host-side AVF runtime**，并未与 Android 设备上的完整 AVF 语义完全对齐；尤其在 **protected VM、真实系统服务/权限/SELinux 语义、AF_VSOCK 原生模型、console、持久服务模型** 等方面仍存在明显差距。

---

## 15. 后续优先级建议（按影响排序）

### P0：如果目标是“接近 Android AVF”

1. **补 protected VM**
2. **把 Windows persistent `virtmgr` 进一步收敛成更接近 Android 的默认服务模型**
3. **把 Windows `vm console` 从 file-backed attach 继续收敛到更接近原生交互式控制台**

### P1：如果目标是“把 Windows 变成稳定开发环境”

1. 补更多回归脚本，固定：
   - `run-microdroid`
   - persistent `vm list`
   - persistent `vm console`
   - `run-app`
   - `vm_shell start-microdroid -AutoConnect`
2. 固化日志 marker 检查
3. 补常见失败场景的一键诊断

### P2：如果目标是“进一步对齐 Android 语义”

1. 减少 permission / SELinux mock 依赖
2. 继续收敛 APEX / staged APEX 替代语义
3. 评估 network/VFIO/设备语义的真实需求

---

## 16. 最终判断

最终结论可以压缩成一句话：

> **Windows AVF 现在已经完成“可运行的 host 版 AVF”阶段，但还没有完成“Android AVF 完整等价”阶段。**

如果一定要给数字：

- **按 Windows 主机工作流评估：约 70%~75% 对齐**
- **按 Android 完整 AVF 能力评估：约 45%~55% 对齐**

这两个数字同时成立，而且缺一不可。前者说明当前成果已经足够实际可用，后者提醒不要高估与 Android 原生 AVF 的语义距离。
