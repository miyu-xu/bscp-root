# 架构说明

简体中文 | [English](ARCHITECTURE.md)

## 设计目标

BSCP 在 KVM、HVF 与 WHPX 上提供统一的 Android 隔离算力控制面。跨平台能力通过明确接口
实现，不把平台差异隐藏在“能力完全相同”的表述后面。

## 运行时分层

1. `vm` 与 `virtmgr` 校验配置、准备载荷元数据和实例存储，并提供生命周期操作。
2. Virtualization 库把 Android 虚拟机请求转换为与主机无关的启动计划。
3. `crosvm` 将计划映射到 KVM、HVF 或 WHPX，并仅暴露显式请求的 virtio 设备。
4. Microdroid 启动精简 Android 用户空间，在 Guest 边界内执行载荷。
5. 可选的 gfxstream/ANGLE 集成提供图形加速，但不改变控制面的信任边界。

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
