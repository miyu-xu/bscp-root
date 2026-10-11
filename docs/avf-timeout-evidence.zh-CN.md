# AVF 超时与关机证据

macOS 包装器和回归脚本只有在本次 watchdog 的计时成功结束，且确实成功发送
TERM 后，才把退出码 137/143 转为允许的超时码 124。子进程自行 exit 137/143、
提前收到外部 TERM/KILL 仍然失败。真实超时的放行策略不变。计时或临时目录创建
失败不得产生超时证据；正常返回时清除临时凭据目录。不新增全局 cleanup trap，
因此包装器自身被强制终止仍可能留下小型临时目录。

这不是完美的信号归因：期限附近外部信号可能与 watchdog 竞争。若 watchdog 在
记录成功发信号前被回收，则保留 137/143，保守失败。PID 选择、TERM/KILL 范围与
升级延迟不变，不新增进程组清理。

运行 `python3 -m unittest discover -s tests -p 'test_timeout_provenance.py' -v`。
测试使用 shell mock，覆盖提前退出/信号、真实到期、提前成功、timer 失败、临时
目录创建失败、期限竞态与记录竞态；不代表原生 macOS/HVF 或 Android 客体验证。

## 验收时分别记录

- Ready 仅证明载荷曾进入 Ready。
- 只有本层从 137/143 转换出的 124 带有本次 watchdog 到期/TERM 凭据。子命令
  原样返回的 124 仍按既有策略放行，不新增来源证明；两者均不证明正常关机。
- 瞬态 vm 客户端仅在 `DeathReason::Shutdown` 后成功返回；producer 根据 crosvm
  成功退出判为 Shutdown，但明确故障原因优先。需保存本次运行/CID 对应的
  `VM ended: Shutdown`、实际命令退出码及 crosvm 的退出/wait 证据。
- 持久 vm 成功返回表示 Ready 后 detach，按设计仍运行。
- stop-service、cleanup、`Running: 0` 均不独立证明进程退出。后者依赖服务注册
  状态；状态文件被删除后会显示零。
- 原生关机验收还需独立核实被跟踪的 VM/服务进程身份已消失；PSCI 结论另需客体/
  backend 关机证据，不能以 Ready 替代。

现有 macOS 回归的 PSCI 注释/提示不是关机门禁；Ready 后超时仍可通过启动检查。
本修改不新增关机要求，也不改变场景策略。
