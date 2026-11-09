# llama.cpp：Host / Microdroid 实测对比

[English](LLAMA-PERFORMANCE.md) · [候选源码](../experiments/aosp17-sync-asg/README.zh-CN.md)

## 已验证能力

Linux/KVM、Windows/WHPX、macOS/HVF 均实际运行过 Microdroid Vulkan 推理。
独立发布验收还覆盖有限离屏像素读回、coherent buffer 工作负载、GPU开关、同盘重启
和有界失败收尾。这不等于任意 Vulkan、UI/输入或跨机器自动推理；coherent 也不免除
CPU/GPU 同步，更不是跨主机缓存一致性。发布包仍限已准入部署环境。

## 历史同机 Vulkan 配对基线

下表不是最新 ASG 构建，也不是操作系统性能排名。每侧三个进程轮次、每轮每项三次，
n=9；单位 tokens/s，均值±合并样本标准差，标准差不是置信区间。
pp512=处理512 token提示，tg128=生成128 token，二者均不是TTFT。

| 平台／硬件 | Host pp512 | Guest pp512 | Host tg128 | Guest tg128 |
|---|---:|---:|---:|---:|
| Linux/KVM，i7-12700KF、RTX2060 6GB、32GiB | 1616.96 ± 4.52 | 1125.59 ± 21.44 | 257.12 ± 3.86 | 178.64 ± 11.61 |
| Windows/WHPX，i5-12400F、RTX2060、32GiB | 1623.47 ± 9.60 | 1136.83 ± 7.61 | 270.59 ± 1.96 | 228.97 ± 9.49 |
| macOS/HVF，Apple M1、8GiB | 784.05 ± 4.85 | 296.24 ± 2.72 | 60.28 ± 1.16 | 9.13 ± 0.03 |

统一模型 Qwen2.5-0.5B-Instruct GGUF Q4_K_M，上游revision
`9217f5db79a29953eb74d5343926648285ec7e67`，491400032字节，SHA256
`74a4da8c9fdbcd15bd1f6d01d621410d31c6fc00986f5eb687824e7b93d7a9db`。
元数据为630167424参数，llama-bench标为qwen2 1B。
llama.cpp提交 `fac3a894fc04901c3e5abaff457e024c6b52e04f`，另有core16/诊断补丁
`9418ef8dada0ee31eec7f607d0a35a5cd851a4d45c6affa4982acc9db982a3ef`。
线程1、batch512、ubatch128、请求GPU层99、FA关闭、两侧FP16运算关闭、保留预热。
Guest为SDK37 normal/noADB、1 vCPU；x86内存4096MiB、Mac2048MiB。
Linux驱动580.178.04。部分历史OS/驱动完整版本字符串未保留于本摘要，不能视为逐位环境复现规范。

Linux CPU集合2/4/6/8，Windows原Job同集合，Mac未绑核；未修改频率/电源策略。
原生GCC/MSVC/Apple Clang与Guest NDK/bionic不同，差异不能全部归因于虚拟化。
Mac为MoltenVK上的Vulkan，不是llama原生Metal后端；使用含vsock修复的隔离调试crosvm
`d78d357e0c4779c603dac1fa94e8c783bab374d013535d4ede0242deb9053c67`，不是发布运行时。
Mac模型准备约208–212秒，吞吐排除VM启动、传输、校验。这组旧基线早于后续调优，
不能冒充最新候选的性能；不同硬件之间也不能直接解释为平台开销。

## CPU观察（采样设计不同）

| 硬件／采样 | Host pp512 | Guest pp512 | Host tg128 | Guest tg128 |
|---|---:|---:|---:|---:|
| Linux i7-12700KF，早期n=5 | 102.59 ± 0.05 | 93.60 ± 0.03 | 33.15 ± 0.07 | 31.91 ± 0.13 |
| Windows i5-12400F，n=9 | 90.14 ± 0.50 | 87.18 ± 0.94 | 32.26 ± 1.20 | 31.79 ± 0.44 |
| Mac M1，事后特征匹配n=9 | 73.06 ± 0.04 | 66.91 ± 0.11 | 55.49 ± 0.06 | 51.70 ± 0.12 |

同模型、单线程、ngl=0。Linux是早期未绑核的五次批次，不是后续受控GPU序列。
Mac原生CPU关闭FP16特征以匹配Guest，新Host样本对照旧Guest样本，并非同期交替。
编译器、运行库和调度差异仍在，不能作为统一跨平台排名。

## 最新ASG回归单列

Windows三次真实受控错误退出、正常导出和9条推理记录通过；Linux真实导出及桥接
开关18条推理记录通过；Mac9条推理和准确“不支持”能力查询通过。logits与各自已有
回归基线逐字节一致，不意味着CPU与GPU彼此完全一致。

Linux新桥接关/开顺序样本（每组内部3次）：pp512为1059.26±4.15／1058.83±3.81，
tg128为218.01±0.64／209.64±13.09。开启组194.52、217.18、217.21，慢样本保留。
此测试不是ASG修复前后对照，不能宣称无回退。Mac本轮没有新的吞吐结论。
详见[脱敏回归数据](../experiments/aosp17-sync-asg/REGRESSION.json)。

## Android真机仅为原生测试

Pixel7、API35、arm64、Mali-G710，同模型哈希，五次样本；同pp512/tg128及线程/
batch/ubatch，CPU ngl0、GPU ngl99，无固定频率：

| 原生后端 | pp512 tokens/s | tg128 tokens/s |
|---|---:|---:|
| CPU | 42.2227 ± 0.5176 | 28.0427 ± 0.5649 |
| Vulkan默认FP16 | 64.7743 ± 0.0888 | 34.5737 ± 0.1312 |
| Vulkan关闭F16 | 82.8302 ± 0.0828 | 34.3895 ± 0.1123 |

CPU内嵌提交unknown、GPU为1b61d01，尚未证明来自同一干净构建，不能推断虚拟化开销。
原生外部semaphore真实导出/信号/等待三次通过；手机Microdroid GPU及同机Host/Guest
配对未运行，仍缺匹配且受支持的图形部署链，没有替换手机系统组件。

严格CPU/GPU logits容差(atol=rtol=1e-4)失败。生成token或top1相同不能替代该检查。
将同一量化权重展开为F32参考（不是恢复原始未量化权重）表明激活量化显著影响差异。
关闭整数点积后最大残差约0.0075–0.0137，采样top10顺序相同，但严格差异仍在；
TTFT由约0.55–0.60秒增至3.13–3.14秒。未变更默认精度或放宽阈值。

## 证据与限制

[公开汇总数据](benchmarks/llama-20261109.json)来自受控记录
BSCP-llama-controlled-evaluation-20261103.json第26版：Linux第1–130行、
Windows第2973–3202行、Mac第10695–10910行。公开内容是精选摘要，不声称私人日志可下载。
最新v2176报告SHA256为
`23ddfe70034a43301e350c84143e66eba9b7139839c3af6732ed8f8b56ba178d`。
实验外部同步默认门禁仍关闭，Mac SYNC_FD不支持。历史UNKNOWN、Mac ext4 70通过/1失败
保留，Windows独立CLI重连仍禁用。本次源码/文档更新不替换正式二进制、tag、冻结
发布清单或旧附件。
