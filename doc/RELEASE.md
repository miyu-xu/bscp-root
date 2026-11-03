# Download, verification, upgrade and rollback

The default is [AOSP17](https://github.com/miyu-xu/bscp-root/releases/tag/bscp-aosp17-20261102). Use [releases/default.json](../releases/default.json) for asset identities and the [manifest repository](https://github.com/miyu-xu/bscp-manifest-repo). The release tag is a frozen source point; later main commits maintain documentation and selectors. Do not treat a later documentation commit as a new runtime build.

Download the selected assets without executing them. Original `SHA256SUMS` covers the original 24 archives; `license-asset-v1837.json` covers the additional license archive. The v1838 documentation ZIP has its own checksum and a filename/hash-only `release-assets-public.json` for the original 28 assets. That index intentionally does not cover later-added documentation files. Use each revision's corresponding checksum; do not assume a mixed download directory is one new release.

Read-only verification using the matching index and downloaded files:

```sh
python3 -B releases/aosp17/verify_release_assets.py --directory "$BSCP_DOWNLOAD_DIR" --manifest "$BSCP_ASSET_INDEX"
```

Set both variables first; Windows may use `python` and PowerShell `$env:` variables. A hash pass establishes identity, not safe deployment or complete privacy removal. Older public attachments and Git history still contain environment/build identity. Clean replacements are being prepared; old assets have not been removed. Third-party example addresses, regular-expression syntax and legal author names must not be mistaken for deployment identities.

Never blindly materialize Guest absolute symlinks on the host or extract over live/failed instance data. Preserve third-party notices. Follow [deployment conditions](DEPLOYMENT.md), and retain Host/Guest/configuration/evidence identities together.

## r14 rollback

Retained branch: `bscp-android-15.0.0_r14` in both repositories. Root commit: `43e4786a48c142b4c8b6f4bfb0db38cd32b20e42`; manifest commit: `89c0365a4dff5c0e52a5386877592a2f6fd45548`. Use a separate source workspace and the retained r14 runtime closure; do not replace only a Guest or DLL. Stop the owned current instance through its normal controller and verify actual resource release before any switch. Preserve failed disks/claims and UNKNOWN task state. Do not replay business requests as a rollback procedure.

Repository history and frozen evidence are historical facts, not current installation instructions. No force-push/history rewrite or default-branch setting change is part of this documentation revision.

[简体中文](RELEASE.zh-CN.md)
