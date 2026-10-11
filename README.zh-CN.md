# BSCP

简体中文 | [English](README.md)

BSCP 是一个跨平台、安全隔离的 Android 通用算力平台。项目把 Android Virtualization
Framework 的主机工作流扩展到 Linux/KVM、macOS/HVF 与 Windows/WHPX，并在三个平台上
保持一致的控制模型：`virtmgr` 与 `vm` 管理载荷，`crosvm` 提供虚拟机监控器，Microdroid
作为首要的精简 Android Guest。

本项目面向隔离计算、可复现构建和跨平台运行，不以手机模拟器为定位。基于 Cuttlefish
镜像运行完整 Android 仅作为兼容性附带能力，文档和发布流程均以 Microdroid 为先。

> manifest 仓库是规范的检出和发布主入口。多仓库同步、固定版本与分支选择应从 manifest
> 仓库开始。

## 平台模型

| 主机 | 虚拟化后端 | 首要 Guest | 能力边界 |
| --- | --- | --- | --- |
| Linux x86_64/arm64 | KVM | Microdroid | 参考开发与验证平台 |
| macOS arm64 | Hypervisor.framework | arm64 Microdroid | 需要 arm64 `com.android.virt` APEX 树 |
| Windows x86_64 | Windows Hypervisor Platform | x86_64 Microdroid | 使用 MinGW 主机工具链；受保护虚拟机能力取决于主机 |

安全隔离由多层共同实现：硬件虚拟化、独立 Guest 内核、显式 virtio 设备、最小权限主机
服务、经过认证的载荷元数据和实例级存储。处理不可信载荷前，请先阅读
[安全模型](doc/SECURITY.zh-CN.md)。

## 仓库边界

本仓库只负责编排、主机脚本、发布文档、测试与固件资源。`external/`、`frameworks/`、
`hardware/`、`packages/` 和 `system/` 下的源码由 `repo` 管理，分别属于独立 Git 仓库；
根仓库不再重复跟踪这些文件，避免版本漂移和双重提交。

组件职责参见[架构说明](doc/ARCHITECTURE.zh-CN.md)，脚本入口参见
[运维手册](doc/OPERATIONS.zh-CN.md)。

## 快速开始：Microdroid

先从 manifest 仓库初始化工作区，再进入本仓库根目录构建。

```bash
./build_all.sh
./scripts/vm_linux.sh --command validate-prereqs
./scripts/vm_linux.sh --command run-microdroid

MACOS_AVF_APEX_TREE_SOURCE=/path/to/arm64/apex_tree ./build_all.sh
./scripts/vm_macos.sh --command validate-prereqs
./scripts/vm_macos.sh --command run-microdroid
```

```powershell
.\build_all.bat
.\scripts\vm_windows.ps1 -Command validate-prereqs
.\scripts\run_microdroid_windows.ps1
```

构建成功后，使用 `run_linux_avf_regression.sh`、`run_macos_avf_regression.sh` 或
`run_windows_avf_regression.ps1` 运行对应平台回归。工具链、产物、APEX 准备与生产加固
要求见[部署指南](doc/DEPLOYMENT.zh-CN.md)。

## 无硬件 AVF 检查器测试

Linux/macOS 日志检查器要求 Ready 证据，不能只凭创建或启动消息放行；按 CID 字段或
完整控制台行匹配目标 CID，并保留持久模式重试失败的退出码。超时、场景选择和清理
策略保持不变。

使用 Python 3、Bash、grep 和 awk 运行以下专项检查，不会启动虚拟机：

```bash
python3 tests/check_avf_markers_fixtures.py
python3 tests/check_avf_gate_boundaries.py --repo-root .
python3 tests/test_retry_exit_status.py --repo-root .
python3 tests/test_service_status_validation.py --repo-root .
python3 -m unittest discover -s tests -v
```

合成日志与模拟重试测试不能证明原生 macOS/Windows 运行、Guest 启停、虚拟化后端、
图形或发布就绪。macOS UDS 单元测试在其他主机上跳过；平台前置条件和完整回归检查
仍须另行执行。

macOS 完整回归要求停止服务后的状态命令执行成功，且只有一个明确的 `Running: 0`
字段。状态读取失败或输出含糊时，先执行最终清理，再使验证失败。此检查验证封装脚本
报告的状态，不能独立证明宿主进程退出或 Guest 正常关机。

## 附带的完整 Android 路径

基于 Cuttlefish 的路径用于兼容性验证、图形链路调试及必须依赖完整 Android Framework
的工作负载，不是默认运行时。请先阅读[Cuttlefish 兼容路径](doc/CUTTLEFISH.zh-CN.md)，
再使用各平台的 `run_android_*` 包装脚本。生成镜像和可写覆盖层不得加入 Git。

## 发布与贡献约束

- 构建产物统一写入 `out/`，不得提交。
- 组件改动必须提交到对应组件仓库，不能提交到根编排仓库。
- 发布分支必须通过 Shell 语法、Python 编译与测试、文档链接、提交身份和提交日期审查。
- 本仓库不包含 GitHub Actions；验证由文档化的本地流程或下游发布系统执行。

提交修改前请阅读[贡献指南](CONTRIBUTING.zh-CN.md)、[安全说明](SECURITY.zh-CN.md)和
[中英文文档索引](doc/README.zh-CN.md)。
## 许可证与商业使用

本仓库直接跟踪的 BSCP 原创编排、脚本、测试、配置和文档采用
[PolyForm Noncommercial License 1.0.0](LICENSE)。商业使用必须事先取得书面授权，并另行
签订或获得[商业许可证](COMMERCIAL_LICENSING.zh-CN.md)。

由于限制商业用途，BSCP 原创部分属于 source-available，而不是 OSI 批准的开源软件。独立
AOSP、crosvm、gfxstream、aemu、固件和其他第三方组件继续适用其自身许可证与声明；BSCP
许可证不会替换或限制它们。使用或再分发前请阅读完整[许可证策略](LICENSE_POLICY.zh-CN.md)。

