# 故障排查

| 现象 | 首先检查 | 不应采取的动作 |
| --- | --- | --- |
| 私有绑定缺失或哈希不符 | 已安装 profile/closure 与配套发布索引 | 用占位符替代或放宽准入 |
| Host 库/模块找不到 | 同版本递归库闭包及支持布局 | 混入 r14 DLL 或修改二进制字节 |
| Guest 未 READY | 原 Guest/控制器日志及 Host/Guest 配对 | 把进程创建当 READY |
| GPU 枚举为零 | 配套图形 Guest、正常启动配置及正式 loader 证据 | 用直接探针替代正式加载成功 |
| 超时/断连 | 原任务状态、稳定句柄及退出/EOF证据 | 重放业务或把 UNKNOWN 改 PASS |
| Permission denied | 区分工具政策、OS 身份/ACL、产品 API 错误 | 盲改 ACL/UAC/网络或换身份 |
| 文档乱码 | 读取 UTF-8 原始字节，比对远端与源文件 | 经旧控制台代码页重新编码 |

保留首个错误及原始证据，wrapper 异常不一定是根因。报告平台、精确产物哈希、命令、退出/EOF 状态及脱敏错误类别，不公开个人路径或凭据。Mac 历史 ext4 失败及 Windows 独立 CLI 重连禁用仍是明确限制，不是绕过检查的操作建议。

[English](TROUBLESHOOTING.md)
