# Limited-deployment reference packages

The [current release](https://github.com/miyu-xu/bscp-root/releases/tag/bscp-aosp17-20261102) is AOSP17. These packages support existing validated deployments only. They do not provision accounts, protected directories, ACLs, drivers or hypervisor access. They are not relocatable merely because public examples omit private paths.

| Platform | Validated environment and required dependencies |
| --- | --- |
| Windows x86_64 | Existing admitted account/runtime root; build 26200, RTX2060 driver 32.0.16.1088, WHPX; matching v1830 Host, DLL closure and x86 Guest |
| Linux x86_64 | Existing admitted layout/KVM access; recorded kernel 7.0.0-38-generic and NVIDIA ICD/system libraries; matching Linux Host and x86 Guest |
| macOS arm64 | Existing admitted volume layout; Apple M1, macOS 26.5.2/25F84, signed crosvm, frozen Vulkan loader/MoltenVK/OpenSSL and arm64 Guest |

An arm64 source target does not establish Linux arm64 runtime acceptance. Windows independent CLI reconnect is disabled. See [operations](OPERATIONS.md) before any start, restart or cleanup.

After [download verification](RELEASE.md), use the original private deployment configuration containing `BINDING.json`, `profile.json`, and `closure.json`. It must match the admitted installed files and state root; it is not a template for new admission. In a checkout of this repository, the following is read-only and starts no VM:

```sh
python3 -B releases/aosp17/verify_installed.py --platform linux --deployment-config "$BSCP_DEPLOYMENT_CONFIG"
python3 -B releases/aosp17/verify_installed.py --platform macos --deployment-config "$BSCP_DEPLOYMENT_CONFIG"
```

```powershell
python -B releases/aosp17/verify_installed.py --platform windows --deployment-config "$env:BSCP_DEPLOYMENT_CONFIG"
```

Run only the line for your actual host. Set the environment variable to the original private configuration directory first. Missing configuration fails closed. The standalone documentation ZIP does not contain all sibling production controllers, so run this verifier from the repository checkout, not from that ZIP alone. Do not publish the private binding or infer permissions from a successful hash check.

[简体中文](DEPLOYMENT.zh-CN.md)
