# HD 功能分支

简体中文 | [English](HD_FEATURE.md)

HD 产品集成只存在于 `hd-feature`。主分支不包含其仓库入口，也不包含产品专用的图形、打包
或运行时挂钩。

请使用 manifest 仓库的 `hd-feature` 分支初始化工作区。该 manifest 会加入 `hd` 仓库，并
选择匹配的组件功能分支。Windows 下运行 `build_hd.bat`：脚本先构建以 Microdroid 为主的
平台基线，再构建、准备并审计 HD 工作区和 UI 产物。

本分支的 Cuttlefish 导入工具只把已经构建的兼容产物转换为 HD Guest 暂存目录，不负责下载、
签名、认证或发布 Android 镜像。生产发布仍必须满足 HD 仓库文档规定的安全与回归门禁。
