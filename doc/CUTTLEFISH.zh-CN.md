# Cuttlefish 兼容路径

简体中文 | [English](CUTTLEFISH.md)

基于 Cuttlefish 的完整 Android 路径是附带兼容能力。支持基线、文档顺序和发布门禁始终以
Microdroid 为先。

三平台实现矩阵、Guest 磁盘、图形架构、虚拟设备、代码修改、安全限制与发布门禁详见
[完整 Android / Cuttlefish 跨平台实现详解](ANDROID.zh-CN.md)。

该路径适合 Framework 级应用兼容、图形扫描输出验证，或精简 Guest 无法表达的设备模拟。
它需要 AOSP 产品镜像、更多主机守护进程、更大的可写存储和更完整的安全审查。

## 工作流

1. 在本仓库之外构建或取得匹配的 AOSP Cuttlefish 产品产物。
2. 使用 `scripts/package_aosp_vm_artifacts.sh` 打包不可变源产物。
3. 为每个实例创建私有可写镜像，不得直接修改发布源镜像。
4. 只启动业务所需的主机设备守护进程。
5. 使用平台 `run_android_*` 包装脚本启动，并归档生成日志。

Linux 使用 `run_android_linux.sh`，macOS 使用 `run_android_macos.sh`，Windows 加速路径使用
`run_android_windows_gfxstream_angle.ps1`。命令参数以各脚本的帮助输出为准。

关闭数据加密的开发配置必须明确标注，不能作为生产镜像发布。Cuttlefish 运行成功不能替代
Microdroid 回归门禁。
