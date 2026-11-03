# 下载、校验、升级与回滚

默认发布为 [AOSP17](https://github.com/miyu-xu/bscp-root/releases/tag/bscp-aosp17-20261102)。资产身份见 [releases/default.json](../releases/default.json)，源码入口见[清单仓库](https://github.com/miyu-xu/bscp-manifest-repo)。发布 tag 是冻结源码点，后续 main 提交维护文档和选择器；文档提交不能冒充新运行包构建。

下载选定资产，不自动执行。原始 `SHA256SUMS` 覆盖最初 24 个归档，`license-asset-v1837.json` 覆盖额外许可归档。v1838 文档 ZIP 有独立校验文件，其中 `release-assets-public.json` 是原始 28 附件的文件名/哈希索引，不覆盖后续新增文档。必须使用对应修订的校验和，不能把混合下载目录视为一个新版本。

使用匹配索引与下载文件进行只读校验：

```sh
python3 -B releases/aosp17/verify_release_assets.py --directory "$BSCP_DOWNLOAD_DIR" --manifest "$BSCP_ASSET_INDEX"
```

先设置两个环境变量；Windows 可用 `python` 和 PowerShell `$env:` 变量。哈希通过仅证明身份，不证明可安全部署或已完全脱敏。旧公开附件及 Git 历史仍有环境/构建身份信息；正在准备干净替代物，旧资产尚未移除。第三方示例地址、正则语法及合法作者署名不能误当作部署身份删除。

不要在宿主盲目创建 Guest 绝对符号链接，也不要解压覆盖活实例或失败实例数据。保留第三方许可，遵守[部署条件](DEPLOYMENT.zh-CN.md)，成套保留 Host/Guest/配置/证据身份。

## r14 回滚

两仓库保留分支 `bscp-android-15.0.0_r14`。根仓库提交 `43e4786a48c142b4c8b6f4bfb0db38cd32b20e42`，清单提交 `89c0365a4dff5c0e52a5386877592a2f6fd45548`。使用独立源码工作区及完整保留的 r14 运行闭包，不能仅换 Guest 或 DLL。切换前用原控制器正常停止本实例，并验证真实资源释放。失败磁盘/claim 与 UNKNOWN 状态必须保留；回滚不是重放业务请求的理由。

仓库历史和冻结证据保留事实，不是当前安装指南。本轮文档修订不强推改写历史，也不更改默认分支设置。

[English](RELEASE.md)
