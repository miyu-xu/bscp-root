# HD feature branch

[简体中文](HD_FEATURE.zh-CN.md) | English

The HD product integration is isolated on `hd-feature`. The main branch contains neither its
repository entry nor its product-specific graphics, packaging, or runtime hooks.

Initialize the workspace with the manifest repository's `hd-feature` branch. That manifest adds
the `hd` repository and selects matching component feature branches. On Windows, run
`build_hd.bat`; it first builds the Microdroid-first platform baseline, then builds, stages, and
audits the HD workspace and UI artifacts.

The Cuttlefish import helper on this branch converts already-built compatibility artifacts into HD
guest staging. It does not download, sign, certify, or publish Android images. Production releases
must still satisfy the security and regression gates documented by the HD repository.
