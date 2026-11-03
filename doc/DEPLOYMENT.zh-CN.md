# 限定部署条件参考运行包

[当前发布](https://github.com/miyu-xu/bscp-root/releases/tag/bscp-aosp17-20261102)为 AOSP17，仅支持已有验收部署。运行包不会创建账户、私有目录、ACL、驱动或虚拟化访问权限。公开示例省略个人路径不代表产物可以任意迁移。

| 平台 | 已验环境与必要依赖 |
| --- | --- |
| Windows x86_64 | 原有准入账户/运行根；build 26200、RTX2060 驱动 32.0.16.1088、WHPX；匹配 v1830 Host、DLL 闭包及 x86 Guest |
| Linux x86_64 | 原有准入布局/KVM 权限；记录内核 7.0.0-38-generic 及 NVIDIA ICD/系统库；匹配 Linux Host 和 x86 Guest |
| macOS arm64 | 原有准入卷布局；Apple M1、macOS 26.5.2/25F84、签名 crosvm、冻结 Vulkan loader/MoltenVK/OpenSSL 及 arm64 Guest |

存在 arm64 源码目标不等于 Linux arm64 运行验收通过。Windows 独立 CLI 重连禁用。启动、重启或清理前请先读[运行操作](OPERATIONS.zh-CN.md)。

[下载校验](RELEASE.zh-CN.md)后，使用原有私有部署配置，包含 `BINDING.json`、`profile.json`、`closure.json`。它必须对应已准入安装文件和状态根，不是新准入模板。在本仓库检出目录中执行以下只读校验，不启动 VM：

```sh
python3 -B releases/aosp17/verify_installed.py --platform linux --deployment-config "$BSCP_DEPLOYMENT_CONFIG"
python3 -B releases/aosp17/verify_installed.py --platform macos --deployment-config "$BSCP_DEPLOYMENT_CONFIG"
```

```powershell
python -B releases/aosp17/verify_installed.py --platform windows --deployment-config "$env:BSCP_DEPLOYMENT_CONFIG"
```

只运行实际宿主对应的一条，先将环境变量设为原有私有配置目录。配置缺失时明确拒绝。独立文档 ZIP 不含全部相邻生产控制器，不能仅在该 ZIP 中运行此校验器。不要发布私有绑定，也不要从哈希通过推断权限已具备。

[English](DEPLOYMENT.md)
