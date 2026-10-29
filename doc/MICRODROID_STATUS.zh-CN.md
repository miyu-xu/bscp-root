# Microdroid r14 现行交付状态（2026-10-29）

本轮代码与清单已发布到两仓库的 `bscp-android-15.0.0_r14` 工程线；GitHub 默认分支仍为 `main`。新生命周期入口为显式 opt-in，未替换三台机器的旧默认入口。完整版本及证据说明见[英文状态页](MICRODROID_STATUS.md)，命令与配置合同见[生命周期使用说明](MICRODROID_LIFECYCLE.md)。清单当前 `default.xml` 的精确 root revision 是实际同步依据。

| 能力 | Linux x86_64/KVM | Windows x86_64/WHPX | Mac arm64/HVF |
|---|---|---|---|
| normal/no-adbd、普通 UID/GID6000 固定 payload | 有限实测通过 | 有限实测通过 | 有限实测通过 |
| 真正 GPU-off/on、新建/正常停止/健康恢复 | 两模式均通过 | 两模式均通过；分别 run/run-mp | 两模式均通过 |
| Vulkan 离屏/coherent | 256 像素零差；12轮12288字零差 | 同规格 | 同规格 |
| VADD 真 compute | 5请求14个 uint32，含溢出，CPU参考零差 | 同规格 | 同规格 |
| off 请求 GPU 计算 | 明确 GPU_DISABLED，无 CPU 回退 | 同左 | 同左 |
| 同盘持久化 | 写118→正常退出→恢复读118→更新119 | 同左 | 同左 |
| 请求超时/断连资源回收 | 原 pidfd | 原 Job/HANDLE | audit token/task-name port/kqueue |

仓库直接入口实测批次为12次正常启动和6个异常案例，绑定 e700090 产品源码；acc143c 只修测试夹具。发布提交89a584c保持已测生产脚本一致。r14 本地45项回归（Linux跳过一个Windows专用项）及三端CI成功；本轮发布和文档同步未新增VM测试。272份仓库证据、此前544份统一候选证据和历史平台实验分别保留，不能混称同一批次。六个文件的CRLF/LF原始字节不同，规范化源码一致；不宣称全平台原始hash相同。

异常任务保持 UNKNOWN，即使资源已证明 REAPED。每例业务 ADD1仅一次、不重试、不补STOP；第一例外层真实EOF/退出码和资源证据齐全后才第二例。失败盘、claim与needsRecovery保留，不自动恢复。Guest自然退出、服务清理、原句柄退出、内外EOF和盘释放各自取证，不能互相替代。历史失败、丢失EOF及手工收尾均不改为PASS。首次5秒和后续只观察不是整体恢复时限保证；真实延迟只观察试验另有版本绑定。

当前不是任意payload或任意shell平台。入口需已准备的固定镜像、HAL/runtime、profile及闭包清单；create显式选择GPU模式，resume不可换模式。GPU-off仍链接libvulkan，不等于零库加载。当前配置protected=false且disable-sandbox，normal/no-adbd不证明生产隔离或保护VM。

未验收：Microdroid GLES/ANGLE/AHB、严格外部pending fence、真实显示/输入/音频、通用网络/共享目录、多实例并发、快照迁移、掉电/坏盘恢复、任意故障、长期压力及性能。Android图形成果不能替代Microdroid证据。A暂停，H3及UI延后。

已完成双模式、有限compute、正常/限定异常生命周期、仓库入口集成及r14发布。后续须另定功能范围；不再等待分支选择。回退用普通Git revert及匹配清单引用，不删除或恢复失败盘，不因文档发布切换默认运行入口。
