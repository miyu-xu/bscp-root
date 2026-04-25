# Linux AVF Parity Report

本文档聚焦两个问题：

1. **Linux host 目前已经对齐了 Windows AVF 的哪些能力**
2. **在完成 Windows parity 之后，还能继续补哪些更接近完整 AVF 的能力**

---

## 1. 结论

本次 Linux host 移植已经把 **Windows host 已具备的核心使用面** 补齐到 Linux：

- host runtime 内部服务分流
- persistent virtmgr service
- reconnectable `vm list`
- reconnectable `vm console`
- `vm_shell` 风格启动
- localhost TCP -> guest vsock ADB bridge
- Linux wrapper / regression 脚本
- host APEX / system 路径映射

也就是说，Linux 现在不再只是“能编 / 能跑部分命令”，而是具备了与 Windows 类似的 **可操作 host runtime workflow**。

---

## 2. 与 Windows 对齐矩阵

| 能力 | Windows | Linux（本次后） | 说明 |
|---|---|---|---|
| `vm info` | 已支持 | 已支持 | wrapper 已补齐 |
| `vm create-partition` | 已支持 | 已支持 | wrapper 已补齐 |
| `vm create-idsig` | 已支持 | 已支持 | wrapper 已补齐 |
| `vm run-microdroid` | 已支持 | 已支持 | transient smoke 与 persistent READY/detach 回归都已跑通 |
| `vm run-app` | 已支持 | 已支持 | 同上 |
| `vm run` raw config | 已支持（Partial） | 已支持（Partial） | 新增 `microdroid_linux_raw.json` |
| `vm list` | persistent 模式可用 | persistent 模式可用 | Unix persistent virtmgr 已补齐 |
| `vm console` | file-backed attach | PTY/TTY attach | Linux 改为 Unix 主机原生 tty attach，Windows 逻辑不变 |
| `service-status` / `stop-service` | 已支持 | 已支持 | wrapper 层实现 |
| `vm_shell start-microdroid` | 已支持 | 已支持 | `vm_shell_linux.sh` |
| `vm_shell -AutoConnect` | 已支持 | 已支持 | 基于 host ADB bridge |
| host ADB bridge | 已支持 | 已支持 | `startHostVsockTcpBridge` 扩展到 desktop host |
| `VIRTMGR_SERVICE_DIR` | 已支持 | 已支持 | Unix spawn + named UDS |

补充说明：

- Linux host permission / SELinux 继续保持与 Windows 一致的 **不实现** 状态
- vsock 相关路径在 Linux 上改成 **能用真实 vsock 就优先真实 vsock**
- `vm console` 已不再以 “Windows 风格 file-backed console” 作为 Linux 主路径

---

## 3. 对齐 Windows 之后，Linux 还能继续补什么

下面这些属于 **超出 Windows parity、面向更完整 AVF/Android 语义** 的下一阶段工作。

### P1. 把更多 host-only mock/bypass 泛化为“desktop host runtime”一致语义

当前仍有部分逻辑和文案偏 Windows：

- `payload.rs` 中 staged APEX provider 仍主要按 Windows host 设计
- `debug_config.rs` / `dt_overlay.rs` 的 host-side overlay 替代源仍偏 Windows 命名与说明
- 某些 warning 文案仍带有 Windows 语义

本轮已经完成一部分收敛：

- 多处 `Windows host` 文案已改为 `desktop host`
- Unix host 的 `VIRTMGR_PATH` / `VIRTMGR_CROSVM_PATH` 与 Windows 覆盖语义已对齐
- binder RPC 的 Unix absolute socket path 语义已补齐，不再默认按 Android named socket 处理

建议：

- 把这些能力统一提升为 **desktop host runtime** 语义
- Windows / Linux / macOS 共享文案与开关，降低维护成本

### P1. Linux host 的 runtime 资源准备自动化

当前 Linux wrapper 直接消费现有：

- `out/dist/linux/bin`
- `out/dist/linux/lib`
- `out/dist/apex_dir`

后续可以继续做：

- 在 `build_all.sh` 里补齐 host runtime 产物的准备/校验
- 为 `out/dist/apex_dir` 做更显式的生成/刷新步骤
- 自动校验 `apex-info-list.xml`、`decompressed/*.apex`、`com.android.adbd` 是否齐备

### P1. Linux host ADB bridge 的更强健可观测性

当前已能启动 bridge，但还可以继续增强：

- 单独桥接日志文件
- 端口占用/冲突诊断
- guest 未监听时的更清晰错误
- `connect` 子命令下的重试与失败提示

### P2. host runtime 权限 / SELinux / staged APEX 能力统一抽象

当前 host runtime 跟 Android 真机仍有根本差异：

- permission service 不存在
- SELinux label 校验不是真实 Android 运行时
- staged APEX / package manager 也不是系统服务

后续可以做：

1. 把这些 host-only 依赖抽成统一 provider trait
2. default provider 对接 Android 真机
3. desktop provider 对接文件/JSON/mock
4. 用相同接口驱动 Linux / Windows / macOS

这会让 host runtime 更像“模块化 AVF 模拟层”，而不只是 scattered host-specific 分支。

### P2. 更贴近 Android 长驻服务模型

当前 persistent virtmgr 是 wrapper + service dir 驱动的 host service 近似模型。  
更进一步可以考虑：

- 单独的 `virtmgr --service` 模式
- 明确的 pidfile / socket lifecycle
- 原生 status / stop 子命令
- 更完整的多 client 共享服务语义

### P2. 网络 / VFIO / device assignment 的 host parity

Windows 当前对这些能力仍是 Partial / Unsupported；Linux 因为底层条件更好，理论上更有机会往前走：

- TAP networking
- VFIO / assignable devices
- 更完整的 device hotplug / passthrough 流程

这部分应当单独评估，因为它会开始逼近“Linux host 优于 Windows host”的分支，而不再只是 parity。

### P3. 与完整 Android AVF 内部服务链路进一步接近

如果目标从“host runtime 可用”升级到“最大程度接近 Android 设备语义”，则还可以继续：

- `VirtualizationServiceInternal` host mock 细化
- host-side attestation / Secretkeeper / authgraph 协议替身
- 更完整的 metrics / tombstoned / stats 路径

这属于更大范围的系统工程，不建议和本次 parity 任务混做。

---

## 4. 推荐后续优先级

建议顺序：

1. **先稳定 Linux wrapper + regression 路径**
2. **再把 remaining host-only 分支从“Windows host mode”统一收敛到“desktop host runtime”**
3. **最后再评估 Linux 是否要继续超越 Windows，补 TAP / VFIO / device assignment**

当前实际状态已经完成了前两步，并且 Linux wrapper / regression 已经拿到稳定绿灯。

---

## 5. 这次改动最重要的价值

不是单纯“Linux 也能跑一个 VM”，而是：

- Linux host 现在拥有了和 Windows 接近的 **完整入口面**
- 后续文档、脚本、回归、使用方法都能按同一套 host runtime 思路维护
- Android device / Windows host 既有逻辑没有被混进去

这为后续继续做 **desktop host runtime 收敛** 和 **Linux 专项增强** 提供了稳定起点。

---

## 6. 本轮新增的 Linux host 关键修复

为了把 Linux 从“接口对齐”推进到“真实可运行”，本轮还补了下面这些底层修复：

- `Threads.cpp` 的 host 编译兼容宏补齐，消除 `android::Thread` 相关构建/链接问题
- `ProcessState::self()` 在 RPC-only desktop host 上恢复正确初始化，避免 `vm info` 段错误
- `frameworks/native/libs/binder/file.cpp` 纳入 host CMake，补齐 `ReadFully` / `WriteFully` 等符号
- Unix `libvmclient` 改为尊重 `VIRTMGR_PATH`
- Unix `virtmgr` 改为尊重 `VIRTMGR_CROSVM_PATH`
- desktop host 不再调用 Android-only `derive_classpath`
- crosvm host feature 集补齐 `composite-disk`
- Linux regression marker 从 Windows trace-only 调整为 trace / guest log / run log 多源回退
- desktop-host crosvm control socket 改成短路径，规避 Unix `ENAMETOOLONG`
- Linux `vm console` 改为内置 PTY attach，不再依赖外部 `microcom`
- regression 会在 transient smoke 后清理由本轮新增的 repo-local host 进程，避免后续 persistent 场景撞上前一轮 transient VM
- regression 入口会先清掉旧的 repo-local regression `virtmgr` / `crosvm` 残留，解决历史失败运行长期占住低位 CID / binder-rpc port 的问题
- desktop host `create_vm_context()` 的重试窗口放大，避免共享环境里低位 CID / port 被占满时过早失败

## 7. 最新验证快照

已验证通过的路径：

- `./build_all.sh`
- `scripts/vm_linux.sh -Command info`
- `scripts/vm_linux.sh -Command create-partition`
- `scripts/vm_linux.sh -Command create-idsig`
- persistent `info/list/service-status/stop-service`
- `scripts/run_linux_avf_regression.sh`
- `scripts/run_linux_avf_regression.sh --include-run-app`

当前结论：

- Linux host 与 Windows host 的主用户面已对齐
- permission / SELinux 仍保持与 Windows 一致的不实现状态
- Linux 额外完成了更贴近桌面主机的 PTY console attach 和 real-vsock-first 收敛
