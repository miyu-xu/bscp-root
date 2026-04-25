# macOS AVF Host Bring-up

本文档记录 **macOS（Apple Silicon）主机** 下 AVF host runtime 的构建、运行与回归入口。目标是让 macOS 与现有 Windows / Linux host workflow 对齐，同时让 crosvm 在 macOS 上默认走 **HVF**。

## 1. 结论

本轮补齐后，macOS host 的主入口与 Linux / Windows 保持一致：

- 统一构建入口仍是仓库根 `./build_all.sh`
- 构建产物收集到 `out/dist/macos`
- `scripts/vm_macos.sh` 提供与 Linux/Windows 对齐的 `vm` wrapper
- `scripts/vm_shell_macos.sh` 提供 `vm_shell` 风格入口
- `scripts/run_macos_avf_regression.sh` 提供主机侧回归路径
- `external/crosvm` 在 macOS 默认启用 `hvf`
- macOS 的 `crosvm` 启动不再停留在 `src/sys/macos.rs` 的 stub，而是复用 `src/crosvm/sys/linux.rs` 中现有的 HVF runtime 主路径

## 2. 前置条件

当前 macOS host 路径面向：

1. **Apple Silicon (`arm64`)**
2. **Hypervisor.framework 可用**（`sysctl -n kern.hv_support` 为 `1`）
3. 已安装 `cmake` 与可工作的 `ninja`
4. 已选择完整 Xcode toolchain（建议：`sudo xcode-select -s /Applications/Xcode.app/Contents/Developer`）
5. 已完成 host build，且 `out/dist/macos/bin/{vm,virtmgr,crosvm}` 与 `out/dist/macos/lib/libbinder-rpc.dylib` 已生成
6. `build_all.sh` 已对 `out/dist/macos` 下的 host binaries / dylibs 完成 ad-hoc 签名；其中 `crosvm` 会额外带上 Hypervisor entitlement
7. `out/dist/apex_dir` 已就绪
8. `out/dist/apex_dir/apex/com.android.virt/etc/fs/microdroid_kernel` 必须是 **arm64 guest kernel**；如果这里仍是 Linux/x86 的 `bzImage`，HVF 启动会在 crosvm 侧失败并报 `kernel could not be loaded: invalid magic number`

快速检查：

```bash
./scripts/vm_macos.sh -Command validate-prereqs
```

## 3. 构建

从仓库根执行：

```bash
chmod +x build_all.sh
./build_all.sh
```

如果已经有一份 **arm64** 的 host APEX tree，可在构建时一并收进 `out/dist/apex_dir`：

```bash
MACOS_AVF_APEX_TREE_SOURCE=/path/to/arm64/apex_tree ./build_all.sh
```

macOS 分支的默认行为：

- `RUST_TARGET` 默认 `aarch64-apple-darwin`
- `binder-rpc` 产物按 `.dylib` 收集
- `build_all.sh` 会通过 `xcrun` 固定 Xcode 的 `clang` / `clang++` 与 macOS SDK，并在使用 Ninja generator 时避开 `depot_tools` 提供的 stub `ninja`
- `build_all.sh` 会对 `out/dist/macos/lib` 与 `out/dist/macos/bin` 中收集到的 Mach-O 产物做最终 ad-hoc 签名，避免 macOS `taskgated` 以 `Code Signature Invalid` 杀掉 `vm` / `virtmgr`
- 如果设置了 `MACOS_AVF_APEX_TREE_SOURCE`，`build_all.sh` 会调用 `scripts/prepare_host_apex_tree.sh` 把外部 APEX tree 刷到 `out/dist/apex_dir`，并在 Darwin 下强制校验 `microdroid_kernel` 为 arm64
- crosvm 默认 feature 集为：
  - `hvf`
  - `default-no-sandbox`
  - `config-file`
  - `qcow`
  - `balloon`
  - `android-sparse`
  - `composite-disk`
  - `tokio`
- macOS 上的 `crosvm` 默认 async executor 为 `tokio`，避免沿用 Linux `Fd/Epoll` executor 路径

构建完成后主要产物位于：

- `out/dist/macos/bin`
- `out/dist/macos/lib`

## 4. Wrapper 入口

新增脚本：

- `scripts/vm_macos.sh`
- `scripts/vm_shell_macos.sh`
- `scripts/check_macos_avf_markers.sh`
- `scripts/run_macos_avf_regression.sh`
- `scripts/microdroid_macos_raw.json`

### 4.1 `vm_macos.sh`

支持的 `-Command`：

- `validate-prereqs`
- `run-microdroid`
- `run-app`
- `run`
- `info`
- `list`
- `console`
- `check-feature-enabled`
- `create-partition`
- `create-idsig`
- `service-status`
- `stop-service`

默认运行时路径：

- 二进制：`out/dist/macos/bin`
- 动态库：`out/dist/macos/lib`
- APEX 树：`out/dist/apex_dir`
- persistent service：`out/dist/logs/macos-virtmgr-service`

如需先把外部 APEX tree 收进默认 dist 目录，可单独执行：

```bash
./scripts/prepare_host_apex_tree.sh \
  --source-root /path/to/arm64/apex_tree \
  --target-root ./out/dist/apex_dir \
  --force \
  --expect-kernel-arch arm64
```

如果 arm64 的 `com.android.virt` 展开树不在默认 `out/dist/apex_dir`，可通过下面任一方式覆盖：

- `MACOS_AVF_APEX_TREE_ROOT=/path/to/apex_tree ./scripts/vm_macos.sh ...`
- `./scripts/vm_macos.sh -ApexTreeRoot /path/to/apex_tree ...`
- `./scripts/vm_shell_macos.sh -ApexTreeRoot /path/to/apex_tree ...`

### 4.2 `vm_shell_macos.sh`

支持：

- `start-microdroid`
- `connect`

`-AutoConnect` 的流程与 Linux 一致：

1. 启动 `run-microdroid`
2. 传入 `--adb-tcp-port <port>`
3. 等待 `notifyPayloadReady`
4. `adb connect localhost:<port>`
5. 可选 `adb root`
6. 可选进入 `adb shell`

### 4.3 `run_macos_avf_regression.sh`

默认回归覆盖：

1. `validate-prereqs`
2. `info`
3. `create-partition`
4. `create-idsig`
5. baseline `run-microdroid`
6. persistent `run-microdroid`
7. `vm list`
8. `vm console`
9. `service-status`
10. `stop-service`

可选覆盖：

- `--include-run-app`
- `--include-adb-scenario`

## 5. 常用命令

```bash
./scripts/vm_macos.sh -Command info
./scripts/vm_macos.sh -Command run-microdroid -KeepTemp
./scripts/prepare_host_apex_tree.sh --source-root /path/to/arm64/apex_tree --target-root ./out/dist/apex_dir --force --expect-kernel-arch arm64
./scripts/vm_macos.sh -ApexTreeRoot /path/to/arm64/apex_tree -Command run-microdroid -KeepTemp
./scripts/vm_macos.sh -Command run-app -KeepTemp
./scripts/vm_macos.sh -Command run -Config ./scripts/microdroid_macos_raw.json -KeepTemp
./scripts/vm_macos.sh -Command create-partition -PartitionPath /tmp/writable.img -PartitionSize 1048576
./scripts/vm_macos.sh -Command create-idsig -OutputPath /tmp/app.idsig
```

persistent service：

```bash
./scripts/vm_macos.sh -Command run-microdroid -PersistVirtmgr
./scripts/vm_macos.sh -Command list -PersistVirtmgr
./scripts/vm_macos.sh -Command console -PersistVirtmgr -Cid 2048
./scripts/vm_macos.sh -Command service-status -PersistVirtmgr
./scripts/vm_macos.sh -Command stop-service -PersistVirtmgr
```

`vm_shell` 风格 + ADB：

```bash
./scripts/vm_shell_macos.sh -Command start-microdroid -AutoConnect -NoShell
./scripts/vm_shell_macos.sh -Command connect -AdbPort 8035
```

## 6. 日志与产物

每次 wrapper 运行会聚合到单独目录，例如：

```text
out/dist/logs/macos-run-microdroid-<timestamp>/
  vm-run-microdroid.log
  virtmgr-trace.log
  vmclient-trace.log
  guest-log.txt
  vm-console.txt
  vm-console-in.txt
  work/
  temp/
```

persistent service 额外状态：

```text
<service-root>/
  virtmgr-service.state
  virtmgr-trace.log
```

## 7. 当前语义边界

1. macOS host 路径当前面向 **HVF + Apple Silicon**，不是 Intel + HAXM 路径。
2. 仍是 **desktop host runtime**，不是 Android 设备环境；permission / SELinux / 系统服务语义不做 1:1 复刻。
3. `vm console` 复用 Unix host 的 PTY attach 路径。
   - 在 persistent virtmgr 场景下，如果调用侧没有本地 tty（例如 regression 脚本），`vm console`
     会自动退化为 **output-only attach**，这样非交互回归也能验证 console 通路。
4. `run_macos_avf_regression.sh` 会主动清理 repo-local `virtmgr` / `crosvm` 残留，避免 CID 与控制 socket 冲突。
5. guest 侧较新的 `IVirtualMachineService` 事务（如 `registerGuestAgent`）已在 desktop host
   runtime 上做兼容处理，不再因为 host/guest AIDL 版本差异阻断 `notifyPayloadStarted` /
   `notifyPayloadReady`。
6. macOS/HVF 目前要求 **arm64 Microdroid guest artifacts**。现有 `out/dist/apex_dir` 如果来自 x86 host 产物（例如 `microdroid_kernel` 被识别为 x86 `bzImage`），需要先替换为 arm64 `com.android.virt` APEX 展开树，否则 `run-microdroid` / `run-app` / `run` 会在 wrapper 前置检查阶段直接拒绝执行。
6. 下列 Linux-only / 当前未实现能力会在 macOS/HVF 路径显式拒绝，而不是静默降级：
   - Wayland devices
   - SCSI devices
   - pmem / pmem-ext2
   - shared directories（FS/9p）
   - vhost-scmi
   - vhost-user-fs / vhost-user-vsock / vhost-user console
   - VFIO passthrough
   - `devices` 子进程模式 / `start_devices`

## 8. 依赖版本说明

为兼容当前仓库使用的 Cargo 工具链，本轮额外收敛了 macOS host 的依赖版本：

- `command-fds` 钉到 `0.3.0`
- `applevisor-sys` 钉到 `1.0.0`

这样可避免拉取需要 Cargo edition 2024 的更新版本，保证现有 host 工具链下仍能构建。
