# 部署指南

简体中文 | [English](DEPLOYMENT.md)

## 检出

使用 manifest 仓库及其固定分支。`repo sync` 成功后会得到本根仓库和各独立组件仓库。
不要在现有工作区上覆盖克隆组件目录，也不要提交生成的 `.repo/` 元数据。

## 主机依赖

- 通用：Git、Python 3、CMake、Ninja、通过 rustup 安装的 Rust，以及兼容 AOSP 的 `repo`。
- Linux：KVM 权限、受支持的 C/C++ 工具链；启用网络测试时还需创建网络设备的权限。
- macOS：Apple Silicon、Xcode Command Line Tools、Hypervisor.framework entitlement，以及
  arm64 `com.android.virt` APEX 树。
- Windows：启用 WHPX、PowerShell 5.1 或更新版本、CMake、Ninja、MinGW-w64、固定的 GNU
  主机 Rust 工具链，并通过 `LIBCLANG_PATH` 或受支持安装提供 `libclang.dll`。

## 构建

```bash
./build_all.sh
./build_all.sh --clean
```

```powershell
.\build_all.bat
.\build_all.bat --clean
```

构建会把 `virtmgr`、`vm`、`crosvm`、binder RPC 库及可选图形运行时放入
`out/dist/<platform>/`。源码目录保持不变，生成状态只写入 `out/`。

## Microdroid Guest 资源

Linux 与 Windows 通常使用同步 Android 构建中的匹配 `com.android.virt` APEX 树。macOS
必须显式提供 arm64 版本：

```bash
MACOS_AVF_APEX_TREE_SOURCE=/absolute/path/to/apex_tree ./build_all.sh
```

启动前用 `vm_linux.sh`、`vm_macos.sh` 或 `vm_windows.ps1` 的 `validate-prereqs` 命令检查环境。

## 生产检查表

- 发布标签必须把每个 manifest revision 固定到已审查提交。
- 除非业务明确需要，否则关闭调试策略、Shell、无限制网络和主机路径透传。
- 使用实例级可写存储，主机密钥保存在工作区之外。
- 校验产物哈希，并随发布包保留第三方声明。
- 晋级前运行平台回归脚本并归档日志。
- 阅读[安全模型](SECURITY.zh-CN.md)；开发模式可运行不等于满足生产安全要求。
