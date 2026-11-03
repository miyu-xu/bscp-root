# Runtime operations

First follow [deployment verification](DEPLOYMENT.md). Use the admitted platform controller and private state root recorded with the installed package. For Windows the published selector uses `releases/aosp17/windows-v1830/scripts/microdroid.py`; Linux and Mac use `releases/aosp17/controllers/<platform>/microdroid.py`. Do not substitute the generic development wrapper for a frozen acceptance controller without validation.

Run the matching controller's `--help` to inspect its current interface. The installed verifier invokes its read-only `check --gpu on` and `check --gpu off`; it does not start a VM. Actual lifecycle commands require original instance/configuration ownership and stable process handles. There is no universal copy/paste start command independent of private deployment configuration.

Preserve logs, original handles, claims and disk identities. A timeout or disconnect leaves the business outcome UNKNOWN; do not retry ADD1 automatically. A sent termination signal is not proof of exit. Observe actual exit/EOF and disk release before declaring resources REAPED. Do not kill by process name, reuse a bare PID, escalate signals, or remove a foreign endpoint to make a test pass.

Healthy same-disk restart tests verify retained value 118. They do not authorize resetting failed instance state. GPU-on requires the matching graphics Guest and complete renderer dependencies; GPU-off is a separate case. Host device creation alone is not Guest hardware enumeration or VADD success.

See [troubleshooting](TROUBLESHOOTING.md), [release boundaries](RELEASE.md) and the archived [timeout evidence](history/avf-timeout-evidence.md).

[简体中文](OPERATIONS.zh-CN.md)

Guest Vulkan offscreen PIXEL, VADD and coherent tests are already verified in the admitted AOSP17 profiles; see [exact scope and evidence](MICRODROID.md#8-verified-microdroid-vulkan-offscreen-rendering). GPU-off must reject GPU work.
