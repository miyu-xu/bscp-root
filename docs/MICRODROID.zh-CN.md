> 技术参考。当前 AOSP17 部署和能力边界以[部署指南](DEPLOYMENT.zh-CN.md)、[发布说明](RELEASE.zh-CN.md)为准；Android/Cuttlefish 不属于当前 Microdroid 发布验收。

# Microdroid 跨平台实现详解

简体中文 | [English](MICRODROID.md)

本文依据当前仓库中的构建脚本、`packages/modules/Virtualization`、`crosvm`、Binder RPC
适配和三平台回归脚本整理。它描述“代码当前实现了什么、如何实现、由什么检查”，不把代码
存在或参数可解析等同于目标机器上的生产认证。

## 1. 定位、范围与状态定义

Microdroid 是 BSCP 的首要 Guest。它提供比完整 Android 更小的启动面，用于运行经过描述和
校验的 Payload，而不是模拟手机。完整 Android/Cuttlefish 是附带兼容路径，参见
[完整 Android 跨平台实现详解](ANDROID.zh-CN.md)。

本文使用以下状态：

- **参考路径**：控制面、Guest 启动和回归脚本以此平台为主要语义基准。
- **已实现**：当前源码存在完整调用链和明确错误处理。
- **有门禁**：仓库提供可重复的回归或 marker 检查；仍需在对应主机执行才能产生运行证据。
- **条件支持**：依赖主机能力、正确架构制品或外部运行时。
- **未对齐**：三平台没有等价安全语义，或代码明确拒绝。

## 2. 总体架构

```mermaid
flowchart LR
    CLI["vm CLI / platform wrapper"] --> VMCLIENT["libvmclient / Binder RPC client"]
    VMCLIENT --> VIRTMGR["virtmgr host service"]
    VIRTMGR --> PLAN["validated VM launch plan"]
    PLAN --> CROSVM["crosvm VMM"]
    CROSVM --> HYP["KVM / HVF / WHPX"]
    CROSVM --> DEV["virtio block / console / vsock / optional devices"]
    DEV --> GUEST["Microdroid kernel + initrd + super"]
    GUEST --> MGR["microdroid_manager"]
    MGR --> PAYLOAD["APK payload or native task"]
    MGR --> RPC["payload state / death reason / logs"]
    RPC --> VIRTMGR
```

各层职责如下：

1. 平台包装脚本定位 host 二进制、APEX 树、日志目录和实例目录，并执行前置检查。
2. `vm` 负责命令解析、APK/idsig/实例镜像参数、控制台和生命周期操作。
3. `virtmgr` 校验配置，解析 `com.android.virt` 资源，创建 composite disk、CID 和启动计划。
4. `crosvm` 将统一计划映射到 KVM、Hypervisor.framework 或 WHPX，并创建显式 virtio 设备。
5. Guest 内 `microdroid_manager` 校验 Payload 身份、启动任务，并通过 RPC 回报 Ready、完成或死亡原因。

## 3. Guest 启动资源

仓库提供三份结构相同的 raw VM 配置：

- `scripts/microdroid_linux_raw.json`
- `scripts/microdroid_macos_raw.json`
- `scripts/microdroid_windows_raw.json`

默认配置包含：

| 资源 | APEX 内路径/含义 |
| --- | --- |
| Kernel | `/apex/com.android.virt/etc/fs/microdroid_kernel` |
| Initrd | `microdroid_initrd_debuggable.img` |
| Verified metadata | `microdroid_vbmeta.img`，作为只读 `vbmeta_a` 分区 |
| System payload | `microdroid_super.img`，作为只读 `super` 分区 |
| Memory | 256 MiB 默认值，可由上层配置覆盖 |
| Console | `hvc0` |
| Platform contract | `~1.0` |

`prepare_host_apex_tree.sh` 将 product 输出整理为桌面主机可消费的 `apex/`、`system/`、
`system_ext/` 结构，并生成/刷新 `apex-info-list.xml`。Windows 包装器还会处理 CAPEX 的
`original_apex`，为桌面端构造确定性的解压 APEX 布局。macOS 会额外校验 Guest kernel 必须
是 arm64。

## 4. 生命周期与数据流

```mermaid
sequenceDiagram
    participant U as Operator
    participant W as vm wrapper
    participant V as vm
    participant M as virtmgr
    participant C as crosvm
    participant G as Microdroid

    U->>W: validate-prereqs / run-microdroid
    W->>W: validate binaries, APEX tree, hypervisor
    W->>V: run-microdroid --work-dir --log
    V->>M: create VM through Binder RPC
    M->>M: validate config, payload, idsig, instance storage
    M->>C: launch explicit kernel/initrd/disks/console/vsock
    C->>G: boot guest
    G->>M: payload started / ready / finished / failure
    M->>V: lifecycle and typed death reason
    V->>W: exit status and logs
    W->>U: marker result and artifact paths
```

持久 `virtmgr` 模式通过 `VIRTMGR_SERVICE_DIR` 保存 service state、PID、控制端点和 trace。
`list`、`console`、`service-status` 与 `stop-service` 因而可连接同一服务，而不是每次启动一套
无关联的进程。Linux/macOS 使用 Unix 文件与 socket；Windows 使用当前会话内的 named pipe
及显式状态文件。

## 5. 三平台实现矩阵

| 能力 | Linux | macOS | Windows |
| --- | --- | --- | --- |
| 主机/Guest 架构 | x86_64 或 arm64，Guest 必须匹配制品 | Apple Silicon/arm64，强制 arm64 Guest kernel | x86_64，GNU Rust/MinGW host |
| Hypervisor | KVM，参考路径 | Hypervisor.framework/HVF | Windows Hypervisor Platform/WHPX |
| 构建 target | `x86_64-unknown-linux-gnu` 或 `aarch64-unknown-linux-gnu` | `aarch64-apple-darwin` | `x86_64-pc-windows-gnu` |
| Guest RPC transport | 原生 `AF_VSOCK` | crosvm host-connect UDS + 端口握手 | `\\.\pipe\binder_rpc_vsock_<cid>_<port>` |
| crosvm control | Unix seqpacket/control socket | Unix stream/control socket | named-pipe control endpoint |
| Host Binder | CMake `libbinder-rpc.so` + Rust bindings | `libbinder-rpc.dylib` + Unix socket 兼容 | `libbinder-rpc.dll` + named-pipe transport |
| APEX 来源 | `out/dist/apex_dir` 或显式 source | 必须显式提供/准备 arm64 APEX 树 | product APEX/CAPEX 组成的桌面树 |
| 基本命令 | `run-microdroid`、`run-app`、`run`、`info`、`list`、`console` | 与 Linux 对齐，另有 `diagnose`、`cleanup` | 与 Linux 对齐，PowerShell 参数模型 |
| 持久服务 | 已实现并有回归流程 | 已实现并有回归流程 | 已实现并有回归流程 |
| Console | 文件或持久服务 PTY | 文件或持久服务 PTY | 文件/named-pipe 双向控制台，可抓取独立串口 |
| ADB bridge | 可选 localhost TCP→Guest vsock | 可选 localhost TCP→UDS-vsock | 可选 localhost TCP→named-pipe-vsock |
| Protected VM | 取决于 KVM/pKVM、pvmfw 与 host capability；非默认门禁 | 没有与 Android pVM 等价的发布证明 | 包装器明确拒绝 `-Protected` |
| Vulkan 离屏渲染 | AOSP17 Guest PIXEL：256 像素、0 不匹配；限定验收配置 | AOSP17 Guest PIXEL：256 像素、0 不匹配；限定验收配置 | AOSP17 Guest PIXEL：256 像素、0 不匹配；限定验收配置 |
| 自动门禁 | `run_linux_avf_regression.sh` | `run_macos_avf_regression.sh` | `run_windows_avf_regression.ps1` |

### 5.1 Linux/KVM

Linux 是参考实现。`vm_linux.sh` 使用 `out/dist/linux/bin/{vm,virtmgr,crosvm}`，APEX 根默认为
`out/dist/apex_dir`。Guest 通信使用真实 AF_VSOCK；control socket、PTY 和文件描述符传递均
使用 Unix 语义。

回归流程覆盖前置检查、`info`、分区创建、idsig 创建、短生命周期 Microdroid、可选
`run-app`、持久服务、CID 解析、`list`、`console`、可选 ADB 和服务停止。CID 冲突时最多
重建四次上下文，不把地址占用误判为 Guest 语义失败。

### 5.2 macOS/HVF

macOS 只接受 Apple Silicon 与 arm64 Microdroid kernel。构建脚本使用 nightly crosvm
toolchain，启用 `hvf`、`net`、`audio` 等 feature，并对 crosvm 做带
`com.apple.security.hypervisor` entitlement 的 ad-hoc 签名。

Guest vsock 不是主机原生 AF_VSOCK：crosvm 暴露 UDS，`virtmgr` 先连接按 CID 命名的
host-connect socket，再写入 Guest port 进行桥接。包装器检查 `kern.hv_support`、签名
entitlement、arm64 kernel 和 `com.android.adbd` 资源。回归提供 smoke/full 两种模式，并
检查 Guest Ready、PSCI 关机、持久服务和可选 ADB。

### 5.3 Windows/WHPX

Windows 使用固定 GNU host toolchain、MinGW-w64 C/C++、WHPX crosvm 和 Binder named-pipe
传输。`vm_windows.ps1` 检查 HypervisorPlatform/Hyper-V 状态和 `HypervisorPresent`，准备
APEX/CAPEX 目录，并设置 `VIRTMGR_*` 环境契约。

Windows crosvm 启动计划主动拒绝 VFIO、Unix TAP 和 `boost-uclamp`。双向 console 通过文件
或 named pipe 传递；Guest vsock 用 `binder_rpc_vsock_<cid>_<port>` 命名。回归会抓取 Guest
console 和 crosvm stdout/stderr，验证持久服务、list/console 与可选 ADB。Protected VM 在
包装层明确失败，不能静默退化成 non-protected VM。

## 6. Payload、实例与存储

### 6.1 `run-microdroid`

使用 APEX 中的 EmptyPayload 构造最小可运行实例，适合验证 Kernel、initrd、virtio block、
vsock、Binder RPC 和生命周期链。它验证基础设施，不代表任意业务 Payload 已兼容。

### 6.2 `run-app`

输入包括 APK、APK idsig、实例镜像和 Payload native library 名。默认示例是
`MicrodroidEmptyPayloadJniLib.so`。`vm` 创建或使用 idsig，`virtmgr` 从签名 Payload 配置
派生 Guest 任务。额外 APK 必须保持配置声明、idsig 和调用者提供描述符的一一对应，不能
通过配置字符串开放任意 host 路径。

### 6.3 Custom VM

`run <config>` 接收显式 JSON 配置，适合内核/initrd/磁盘自定义。它扩大了受信配置面，发布
系统必须固定 schema、文件来源和路径根，不能直接接受不可信用户提供的 host path。

### 6.4 实例状态

- `work-dir`：每次运行的 composite disk、临时文件和实例状态。
- `instance`：Payload 身份与持久 VM 状态，不能跨不相容 Payload 复用。
- `idsig`：APK 完整性辅助数据，应与 APK 一起管理。
- 可写存储：必须为每实例独立分配；发布源镜像保持只读。
- 日志：console、Guest log、virtmgr trace、vmclient trace 和 crosvm stdio 分开保存。

## 7. Host 抽象与跨平台修改

`desktop_host` 将平台差异拆成五个能力接口：Permission、SELinux、staged APEX、vsock 和
debug policy，并提供 CPU 数与文件句柄路径解析。主要修改分布如下：

| 仓库 | 主要修改 |
| --- | --- |
| `packages/modules/Virtualization` | 桌面 host abstraction；Windows crosvm launcher；macOS/Windows vsock 仿真；APEX/CAPEX discovery；持久 virtmgr；console、trace、Payload 与 death reason 对齐 |
| `frameworks/native` | Host Binder RPC CMake 构建；macOS socket 兼容；Windows named-pipe RPC/vsock；Rust bindgen/import library；Win32 OS/thread/fd 兼容层 |
| `external/crosvm` | KVM 基线；HVF aarch64 VCPU/VM、vmnet、Cocoa/console/vsock；WHPX 中断、SMP、串口与 named pipe；跨平台 block/net/gpu 设备 |
| `system/core` | Host 构建所需 atomic/thread portability，Guest 安全语义仍以 Android 基线为准 |
| 根仓库 | 三平台构建、APEX staging、运行包装、回归、日志和发布编排 |

## 8. 已验证的 Microdroid Vulkan 离屏渲染

**AOSP17 Guest 内真实 Vulkan 离屏执行已在三个准入平台通过验证。** 这不是仅创建 Host 设备或源码/构建就绪的结论。有限的 `PIXEL` 工作负载完成渲染与回读，256 个像素、0 个不匹配；正式 Guest 设备枚举、VADD uint32 向量（含溢出用例）以及选定的 coherent 内存映射/绑定/填充组合也已通过。

| 平台 | 实际验收版本 | 范围 |
| --- | --- | --- |
| Windows x86_64 / RTX 2060 | v1828 正常、v1829 故障、v1830 汇总 | GPU-on 初启和健康同盘重启；GPU 开/关共四例正常验证 |
| Linux x86_64 / RTX 2060 | v6 正常 / f6 故障 | GPU-on 初启和健康同盘重启；GPU 开/关共四例正常验证 |
| macOS arm64 / Apple M1 | v32 GPU-on 初启/恢复 v1786 | v32 受影响路径已验证；GPU-off 仍是单独记录的 v30 基线，不能混称全部模式重跑 |

见[绑定制品的公开证据摘要](aosp17/OFFSCREEN-VALIDATION.json)、[冻结版本索引](aosp17/frozen-delivery/versions.json)和 [Windows 最终记录](aosp17/WINDOWS-FINAL-RESULT-v1830.json)。摘要包含准确的 Host/Guest/renderer 哈希及原始回执哈希；脱敏展示副本不替代私有原始证据。对应发布为 `bscp-aosp17-20261102`，使用固定 AOSP17 图形 Guest 与配套平台 Host。

GPU-off 对 GPU 请求明确返回 `GPU_DISABLED`，不是 Vulkan 成功案例。VADD 计算、像素回读与 coherent 内存是不同检查。枚举响应里的 `actual_coherent_use=NOT_TESTED` 只描述枚举命令本身；另行执行的 `COHERENT` 命令验证了记录中的内存组合。

这些结果不代表真实画面/scanout 或输入支持、任意 Vulkan 功能/shader、性能保证、所有 GPU/驱动/内存类型，或任意新机器部署。本文修订不代表重新运行 VM。超时/断连任务仍为 UNKNOWN，即使资源已 REAPED；历史失败及 Mac ext4 限制继续保留。

### 8.1 三端 coherent memory 已支持，仍需同步

已验 Guest Vulkan 链路为 payload -> virtio-gpu/rutabaga -> gfxstream -> 平台 Vulkan 后端：Linux/KVM、Windows/WHPX 使用已验证的 NVIDIA RTX 2060 配置；macOS/HVF 使用 Apple M1 上的 MoltenVK/Metal。Mac 结果来自选定 M1 设备上的真实 Guest 内存操作，不是仅示例或 CPU 设备回退的结论。

实际探针申请 HOST_VISIBLE | HOST_COHERENT 的 CPU 源/目标缓冲区，写入映射内存，执行带 pipeline barrier 的 GPU buffer copy/fill，提交并等待 fence 后回读检查；应用侧不调用 flush/invalidate。每条命令覆盖 2 种 map/bind 顺序 × 2 种 padding × 3 次迭代，共 12 例，每例比较 1024 个 uint32，记录中均为零不匹配、fence SUCCESS，并包含 unmap/remap。

实测 CPU 源内存标志分别为 Windows type 3 / 0x6、Linux type 4 / 0xe、Mac type 1 / 0xf；这些索引只适用于对应配置，不能硬编码为跨平台常量。Linux 启用的 KVM cached-coherent 映射策略仅暴露兼容的真实 Host 内存类型；该配置与 VulkanAllocateHostMemory 同时启用明确不受支持。Mac 实测了标记为 HOST_VISIBLE/HOST_COHERENT/HOST_CACHED/device-local 的内存分配和映射路径。

Coherence 对这些分配免除了应用显式 cache flush/invalidate 的要求，**不免除** CPU/GPU 执行顺序、barrier 或 fence 等待。这是单个 VM 图形链路内的数据共享，不是跨主机分布式共享内存；不保证所有资源类型、所有传输零拷贝、任意硬件/驱动或性能。[证据摘要](aosp17/OFFSCREEN-VALIDATION.json)记录准确平台制品及探针源码哈希。

## 9. 安全边界与必须明确的限制

### 9.1 当前可依赖的边界

- Guest 使用独立 kernel、内存和显式虚拟设备。
- Kernel/initrd/super/vbmeta 和 Payload 身份在启动计划中明确绑定。
- 实例可写数据、日志与控制端点可分目录隔离。
- Protected 请求在不能满足时应失败；Windows 已在 wrapper 中强制这一行为。
- typed death reason 区分 Payload 变化、验证失败、无效配置、连接失败、VMM crash 与正常关机。

### 9.2 当前不能宣称的保证

- 桌面 Permission/SELinux provider 为平台适配，必须使用已准入的认证入口；
  这不等价于 Android system_server + SELinux policy 的安全保证。
- `virtmgr` 的 Unix 与 Windows crosvm launcher 当前都传入 `--disable-sandbox`；因此 VMM
  进程沙箱不是当前跨平台发布基线的既成保证。
- macOS HVF 与 Windows WHPX 不等价于 Android pKVM/pVM；没有硬件 KeyMint、远程证明和
  生产信任链时，不得宣传为 Android protected VM。
- 默认 raw 配置使用 debuggable initrd。调试 shell、ADB、console 和 trace 必须在生产
  profile 中显式关闭或重新审查。
- Host bridge 只应绑定 loopback；扩大监听地址会扩大 Guest 服务攻击面。

## 10. 构建与运行

Linux：

```bash
./build_all.sh
./scripts/vm_linux.sh --command validate-prereqs
./scripts/vm_linux.sh --command run-microdroid
./scripts/run_linux_avf_regression.sh
```

macOS：

```bash
MACOS_AVF_APEX_TREE_SOURCE=/absolute/arm64/apex_tree ./build_all.sh
./scripts/vm_macos.sh --command validate-prereqs
./scripts/vm_macos.sh --command run-microdroid
./scripts/run_macos_avf_regression.sh --scenario-mode smoke
```

Windows：

```powershell
.\build_all.bat
.\scripts\vm_windows.ps1 -Command validate-prereqs
.\scripts\run_microdroid_windows.ps1
.\scripts\run_windows_avf_regression.ps1
```

## 11. 验证层级

1. **静态层**：Shell/PowerShell/Python 语法、Rust/C++ 编译、文档链接与禁止信息扫描。
2. **产物层**：`virtmgr`、`vm`、`crosvm`、Binder RPC 动态库非空且架构正确。
3. **前置层**：Hypervisor、entitlement、APEX 树、Guest kernel 架构和可执行文件。
4. **启动层**：Guest kernel、initrd、block 和 console marker。
5. **控制层**：Payload Ready、typed death reason、list/console、持久 service。
6. **数据层**：可选 ADB 实际连接、Guest shell 回读和 Payload 输出。
7. **清理层**：服务停止、进程退出、控制端点与临时资源回收。

仓库中的 marker checker 是证据判定逻辑，不是已经执行的证据。发布时必须保存目标机器上
本次运行产生的日志、主机信息、制品摘要和退出状态。

## 12. 已知未对齐项

| 项目 | 当前结论 |
| --- | --- |
| pVM/protected VM | Linux 条件能力；macOS 未认证；Windows 明确不支持 |
| Android SELinux/Permission | 桌面平台准入不等价于 Android system_server/SELinux 执行机制 |
| crosvm process sandbox | 当前 launcher 禁用，需额外 host sandbox 或后续实现 |
| VFIO/device assignment | Linux/Android 专属；Windows launcher 拒绝，macOS 无等价路径 |
| hugepages/uclamp | 平台语义不同，不能只凭参数存在声明对齐 |
| Microdroid Vulkan | 三个准入平台的真实 Guest 离屏像素、VADD 和选定 coherent 用例已验证；见第 8 节 |
| 生产认证 | 需要签名制品、硬件能力、密钥/证明链和平台实机证据 |

## 13. 关键代码入口

- 构建：[build_all.sh](../build_all.sh)、[build_all.bat](../build_all.bat)
- Linux：[vm_linux.sh](../scripts/vm_linux.sh)
- macOS：[vm_macos.sh](../scripts/vm_macos.sh)
- Windows：[vm_windows.ps1](../scripts/vm_windows.ps1)
- Guest raw 配置：[microdroid_linux_raw.json](../scripts/microdroid_linux_raw.json)
- Host abstraction：`packages/modules/Virtualization/libs/desktop_host/`
- VMM launch：`packages/modules/Virtualization/android/virtmgr/src/crosvm/`
- Vsock transport：`packages/modules/Virtualization/android/virtmgr/src/vsock_transport.rs`
- 回归：[run_linux_avf_regression.sh](../scripts/run_linux_avf_regression.sh)、
  [run_macos_avf_regression.sh](../scripts/run_macos_avf_regression.sh)、
  [run_windows_avf_regression.ps1](../scripts/run_windows_avf_regression.ps1)
