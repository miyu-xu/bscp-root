# 源码同步与构建

默认版本是 AOSP 17；清单逐项目固定提交，不是给所有仓库指定同名分支。完整快照含 1087 个官方项目及 BSCP 根仓库。较小的 Guest 构建闭包不能证明全树同步完成。

新工作区使用 [README](../README.zh-CN.md) 命令，记录清单仓库提交，并执行 `repo manifest -r -o resolved-manifest.xml` 保存实际版本。已有工作树须先核对本地分支、已跟踪及未跟踪改动；不得用破坏性 reset/clean 强制同步。

已发布运行包对应 [AOSP17 证据](aosp17/)索引中的源码闭包和工具/配置记录。通用 `build_all.sh`/`build_all.bat` 是开发入口，仅运行它们不代表复现或验收了冻结运行包。必须使用各平台源码闭包、精确依赖锁、原生库配方和 Guest 输入，不能混入 r14 库。

配置/构建脚本可能下载依赖并生成大量输出，运行前先阅读参数。隔离输出并核算磁盘和内存，保留 r14 资产，内存不足时串行构建架构。文档命令不会替你自动安装驱动、修改 ACL、开启虚拟化或变更网络设置。

在本仓库执行以下源码/来源检查，不启动 VM：

```sh
python3 -B releases/aosp17/verify_snapshot.py
python3 -B releases/aosp17/verify_host_sources.py
python3 -B releases/aosp17/test_public_verifiers.py
```

Windows 用 `python` 替代 `python3`。最后一条会创建并保留测试夹具。这些检查不是原生编译或 VM 验收。GitHub AOSP17 工作流会在三种系统检查源码/来源；旧文档中“没有 GitHub Actions”的说法已过时。

[English](BUILD.md)
