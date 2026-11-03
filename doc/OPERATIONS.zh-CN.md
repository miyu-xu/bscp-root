# 运行操作

先做[部署校验](DEPLOYMENT.zh-CN.md)，使用已安装包记录的准入平台控制器及私有状态根。Windows 发布选择器使用 `releases/aosp17/windows-v1830/scripts/microdroid.py`，Linux/Mac 使用 `releases/aosp17/controllers/<platform>/microdroid.py`。未经验证不能用通用开发 wrapper 替代冻结验收控制器。

用匹配控制器的 `--help` 查看实际接口。安装校验器调用只读的 `check --gpu on` 和 `check --gpu off`，不启动 VM。真实生命周期命令需要原实例/配置所有权和稳定进程句柄；不存在脱离私有配置的通用一键启动命令。

保留日志、原句柄、claim 和磁盘身份。超时/断连后业务结果保持 UNKNOWN，不自动重试 ADD1。发出终止信号不等于进程退出；只有观测真实退出/EOF和磁盘释放后才能判定资源 REAPED。不得按进程名批量杀进程、复用裸 PID、升级信号或删除其他实例端点凑通过。

健康同盘重启验证保留值 118，不授权重置失败实例。GPU-on 需要配套图形 Guest 和完整 renderer 依赖；GPU-off 单独验证。Host 创建设备不等于 Guest 硬件枚举或 VADD 通过。

参阅[故障排查](TROUBLESHOOTING.zh-CN.md)、[发布边界](RELEASE.zh-CN.md)及归档的[超时证据](history/avf-timeout-evidence.zh-CN.md)。

[English](OPERATIONS.md)
