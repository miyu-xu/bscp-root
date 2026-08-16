# 运维手册

简体中文 | [English](OPERATIONS.md)

## 支持入口

| 用途 | Linux | macOS | Windows |
| --- | --- | --- | --- |
| 构建主机工具 | `build_all.sh` | `build_all.sh` | `build_all.bat` |
| Microdroid 生命周期 | `scripts/vm_linux.sh` | `scripts/vm_macos.sh` | `scripts/vm_windows.ps1` |
| Guest Shell/控制台 | `vm_shell_linux.sh` | `vm_shell_macos.sh` | `vm_shell_windows.ps1` |
| 回归验证 | `run_linux_avf_regression.sh` | `run_macos_avf_regression.sh` | `run_windows_avf_regression.ps1` |
| 完整 Android（附带） | `run_android_linux.sh` | `run_android_macos.sh` | `run_android_windows_gfxstream_angle.ps1` |

传递底层 crosvm 参数前先查看包装脚本帮助。推荐顺序为 `validate-prereqs`、`info`、
`run-microdroid`，这样可把主机配置错误和 Guest 启动错误分开。

## 验证顺序

1. 执行平台前置检查。
2. 修改原生 ABI 或 Rust target 后执行干净构建。
3. 不启用可选设备，先运行 Microdroid 冒烟路径。
4. 运行平台回归包装脚本并检查输出目录。
5. 基线通过后再启用网络、图形或完整 Android。

Python 辅助脚本可用 `python -m compileall scripts tests` 检查，Shell 使用 `bash -n`，
PowerShell 应在干净进程中解析。`tests/` 保存根编排单元测试。

## 日志和失败处理

包装脚本把日志写入 `out/dist/logs/` 或显式指定的日志目录。应把启动命令、主机能力报告、
crosvm 日志、Guest 控制台和 manifest revision 一起保存。发布日志前必须检查本地路径、令牌、
Guest 数据与密钥。

常见失败包括主机虚拟化不可用、APEX 架构不匹配、Windows ABI 混用、旧输出目录、TAP 设备
不可用和图形加载器不匹配。修改主机后应重新执行前置检查，不要绕过检查继续运行。
