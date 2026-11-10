> 技术参考。当前 AOSP17 部署和能力边界以[部署指南](DEPLOYMENT.zh-CN.md)、[发布说明](RELEASE.zh-CN.md)为准；Android/Cuttlefish 不属于当前 Microdroid 发布验收。

# 架构说明

简体中文 | [English](ARCHITECTURE.md)

## 设计目标

BSCP 在 KVM、HVF 与 WHPX 上提供统一的 Android 隔离算力控制面。跨平台能力通过明确接口
实现，不把平台差异隐藏在“能力完全相同”的表述后面。

## Payload 契约：已有机制与后续组合

Guest 入口是 `AVmPayload_main`，返回后以退出码结束 VM。Ready 通知、Binder RPC 服务、
打包资源读取及已配置实例存储，让 payload 可以实现一次性任务或持续服务；业务不限定为 AI。
当前有限证据覆盖 CPU/GPU 工作负载、离屏读回、coherent memory 与持久化，不证明任意
native 库或 Android 应用兼容。

NDK/CMake、bionic 和 APK 资源可在匹配 ABI/API 级别及库命名空间的前提下复用。
AIDL 需要匹配的生成绑定、运行库和传输，不会暴露全部 Android 系统服务。
Framework/UI、设备 HAL 服务及硬件编解码不是 payload 自动具备的能力；BSCP 图形 Guest
是超出最小 Microdroid 环境的显式集成。

统一产物身份、输入输出、资源要求、能力协商与状态迁移的任务描述是设计方向，不是已完成
的可移植 SDK。x86_64 与 arm64 仍需分别构建 native 产物。手机原生测试不证明手机
Microdroid GPU 支持。跨设备调度、统一配额和对抗性多租户隔离仍属目标；桌面 HVF/WHPX
不是 Android pKVM。pVM/证明保证需要独立支持并验证的硬件、固件与运行时配置。

## 运行时分层

1. `vm` 与 `virtmgr` 校验配置、准备载荷元数据和实例存储，并提供生命周期操作。
2. Virtualization 库把 Android 虚拟机请求转换为与主机无关的启动计划。
3. `crosvm` 将计划映射到 KVM、HVF 或 WHPX，并仅暴露显式请求的 virtio 设备。
4. Microdroid 启动精简 Android 用户空间，在 Guest 边界内执行载荷。
5. 可选的 gfxstream/ANGLE 集成提供图形加速；renderer、共享缓冲区与宿主驱动增加了需要单独评估的数据面信任依赖。

## 仓库所有权

manifest 分别固定 crosvm、Virtualization、binder/native 支持、gfxstream、aemu 支持与
Android core 等独立仓库；根仓库只负责编排。每个文件只有一个 Git 所有者，便于审查、
回滚、许可证追踪和发布溯源。

## 隔离边界

虚拟机进程、Guest 内核、载荷、虚拟设备、主机桥接进程和输出存储是不同主体。只有启动
计划明确启用时，载荷才能访问主机文件、网络、图形、音频或调试接口。包装脚本为每个实例
使用独立工作和日志目录；生产系统还应叠加主机操作系统的进程沙箱。

## 跨平台契约

- Linux/KVM 是参考实现。
- macOS/HVF 使用 arm64 Guest 与 Apple 虚拟化 entitlement。
- Windows/WHPX 使用 GNU Windows Rust target 和 MinGW 兼容原生库。
- 不支持的能力必须在前置检查或启动计划构建阶段明确失败，受保护工作负载不得静默降级。

完整 Android 兼容路径会复用 crosvm、图形和设备栈，但不属于 Microdroid 的安全与发布基线。
