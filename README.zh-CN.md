# BSCP — AOSP 17

AOSP 17 是 Linux/KVM、macOS/HVF 和 Windows/WHPX 上 Microdroid 算力流程的默认源码基线。已发布二进制是**限定部署条件的参考运行包**，不是新机器通用安装器。必须保留原有私有准入配置及匹配的 Host/Guest 产物；改名或替换路径不代表支持新账户、新目录。

- [文档入口](doc/README.zh-CN.md)：中英文导航及当前/历史边界。
- [下载、校验与回滚](doc/RELEASE.zh-CN.md)。
- [部署条件](doc/DEPLOYMENT.zh-CN.md)与[运行操作](doc/OPERATIONS.zh-CN.md)。
- [架构](doc/ARCHITECTURE.zh-CN.md)、[源码与构建](doc/BUILD.zh-CN.md)、[故障排查](doc/TROUBLESHOOTING.zh-CN.md)。

## 当前能力边界

分平台记录的验收覆盖 GPU 开/关、VADD、离屏/coherent、健康同盘重启，以及有界超时/断连回收。资源 REAPED 不会把 UNKNOWN 业务改成成功。Windows 独立 CLI 重连仍禁用；Mac 历史 ext4 结果保留 70 通过、1 失败。A/H3、真实画面/输入及任意新机器安装均不属于本发布验收。构建或源码 CI 通过不等于 VM 测试通过。

`main` 选择 AOSP 17，`bscp-android-15.0.0_r14` 保留用于回滚。不要通过重置已有工作区或覆盖实例磁盘切换版本。

## 源码与校验

以下命令仅用于新工作区，会初始化元数据并下载源码：

```sh
repo init -u https://github.com/miyu-xu/bscp-manifest-repo.git -b main
repo sync
```

这不是运行包安装命令。请查阅固定的[清单](https://github.com/miyu-xu/bscp-manifest-repo/blob/main/aosp17.xml)、[发布选择器](releases/default.json)和[构建说明](doc/BUILD.zh-CN.md)，并保存解析到的清单提交用于复现。

## 许可与贡献

BSCP 原创部分采用 [PolyForm Noncommercial 1.0.0](LICENSE)，属于源码可用，不宣称是 OSI 认可的开源软件。商业使用须取得[单独授权](COMMERCIAL_LICENSING.md)。第三方组件保留原许可，参阅[许可政策](LICENSE_POLICY.md)、[贡献说明](CONTRIBUTING.md)及[安全说明](SECURITY.md)。

[English](README.md)
