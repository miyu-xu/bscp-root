# 隔离同步与 ASG 候选

仅发布源码，**不由正式 release selector 启用**。[English](README.md)

本包整理已验证的保守 fence 所有权、严格提交结果、延迟队列拒绝、descriptor
回收、Guest 等待错误与能力门禁，再加入 ASG 退出修复。它不是完整外部 semaphore
桥接实现；Linux 内核 out-fence 与后续 Windows 原生 payload 轮换/decoder 生成器
实验未在此静默合入或启用。不支持的能力仍须拒绝。

ASG 析构原来向有界队列阻塞发送 Exit；消费者已退出而队列满时可能卡住。
修复改用队列 stop，将接收失败明确映射为 Exit，并在每轮读回调前重置 sleeping。
消费者销毁和 GPU 对象生命周期等待仍保留。

## 复现源码覆盖层

使用已有 Python 3 和 Git，不安装软件。在本目录执行：

```sh
python apply_candidate.py --host snapshots/host-before --guest snapshots/guest-before --output /path/to/new-overlay
```

需自行指定新的输出路径，父目录已存在、目标不存在。脚本写入前核验输入哈希，
应用后核验输出，不修改输入树。MANIFEST.json 是逐文件基线合同，不声称适用于任意
上游 HEAD。输出只是部分源码覆盖层，不能独立构建；须配合固定完整源码和工具链，
重新编译受影响 backend/common/address-space/renderer 入口对象，保留配套 strict
Vulkan archive；Guest 编译需 BSCP_REQUIRE_NATIVE_SUBMIT_RESULT=1。不得覆盖运行中部署。

REGRESSION.json 为脱敏实测摘要。组合补丁应用测试不等于本次将整个组合覆盖层重新跑过
三平台 VM；实际运行来自各平台已记录的组件组合分支。ASG 语义一致，Windows 的注释/
格式可能不同。Windows 覆盖三次受控错误退出、正常导出与推理；Linux 覆盖正常导出、
桥接开关推理和小规模吞吐；Mac 覆盖推理与准确报告“不支持”的能力门禁。
不能外推成完整故障矩阵、任意并发或完整外部 semaphore 支持。旧 UNKNOWN 不改写。

tests 中保留生产方法抽取测试与 MessageChannel 回归，需已有项目头文件和工具链；
mock 范围见源码，不能替代真实 Guest。可选 llama 补丁不是核心覆盖层的自动启用项。
第三方源码原许可证头保留，不被仓库自有代码许可证重新授权。

参见[性能与限制](../../docs/LLAMA-PERFORMANCE.zh-CN.md)。
