# Linux AVF Host Bring-up

本文档记录 **Linux 主机** 下 AVF host runtime 的落地形态、使用方法、与 Windows 已实现功能的对齐结果，以及本次改动涉及的关键代码入口。

## 1. 基线

本次整理参考了当前 `doc/` 中的 Windows 侧文档，并以工作区对应 bare repo 的最新提交作为基线：

- `packages/modules/Virtualization.git`: `1e0c1eda7` `bscp: fisrt commit.`
- `frameworks/native.git`: `f2f4d7b472` `bscp: fisrt commit.`
- `external/crosvm.git`: `ceb2ee651` `bscp: fisrt commit.`

目标是：

1. **先对齐 Windows host AVF 已有能力**
2. **不影响 Windows 既有逻辑**
3. **再评估 Linux 相比“完整 AVF / Android 真机语义”还能继续补什么**

---

## 2. 本次 Linux host 对齐完成项

### 2.1 host runtime 控制面不再依赖 Android 内部服务

此前 Linux host 运行 `virtmgr` 时，仍沿用 Android 设备路径中对 `IVirtualizationServiceInternal` 的假设；这会让 host runtime 在关键路径上依赖 Android-only 服务。

现在改为：

- **所有非 Android 目标**（Linux / macOS / Windows）统一走 `host_internal_service`
- Android 设备目标保持原有 `wait_for_interface(...)` 逻辑不变

代码入口：

- `packages/modules/Virtualization/android/virtmgr/src/aidl.rs`
- `packages/modules/Virtualization/android/virtmgr/src/host_internal_service.rs`

### 2.2 Linux 支持 persistent virtmgr service

Windows 侧已有 `VIRTMGR_SERVICE_DIR` 持久服务模式；本次补齐 Linux：

- `libvmclient` 的 Unix spawn 支持读取 `VIRTMGR_SERVICE_DIR`
- 持久模式下使用 **Unix domain socket path** 暴露 `virtmgr`
- 跨命令复用同一 `virtmgr` 实例
- 生成服务状态文件：
  - `virtmgr-service.state`
  - `virtmgr-service.sock`

这样 Linux 现在也具备：

- `vm list` 看到同一个持久 `virtmgr` 管理下的 VM
- `vm console` 重新连回同一个 VM 的 host console 元数据
- wrapper 层提供 `service-status` / `stop-service`

代码入口：

- `packages/modules/Virtualization/libs/libvmclient/src/spawn_unix.rs`
- `packages/modules/Virtualization/libs/libvmclient/src/lib.rs`
- `packages/modules/Virtualization/android/virtmgr/src/non_windows_main.rs`

### 2.3 Linux host 支持 PTY/TTY-backed `vm console`

按本次要求，Linux 不再复用 Windows 的 file-backed console 主路径，而是改成更贴近 Unix 主机习惯的 **PTY/TTY** 流程：

- persistent `virtmgr` 模式下，`vm run-*` 会为 guest console 分配 PTY
- `hostConsoleName` 记录 PTY slave 路径
- `vm console` 在 Linux host 下直接由 `vm` 自身打开 PTY 并做 stdin/stdout 双向转发
- Windows 仍保持原有 file-backed console 逻辑，不受影响

这让 Linux `vm console` 真正变成 tty attach，而不是日志文件回放。

代码入口：

- `packages/modules/Virtualization/android/vm/src/run.rs`
- `packages/modules/Virtualization/android/vm/src/main.rs`
- `packages/modules/Virtualization/libs/libvmclient/src/lib.rs`

### 2.4 Linux host 支持 localhost TCP -> guest vsock ADB bridge

Windows 侧已有 `startHostVsockTcpBridge()` + `vm_shell_windows.ps1 -AutoConnect`。  
本次为 Linux host 补齐：

- `IVirtualMachine.startHostVsockTcpBridge()` 在 **非 Android host** 可用
- `crosvm_unix` 增加 host-side bridge listener
- 桥接模型为：
  - host 监听 `127.0.0.1:<port>`
  - 接入后转发到 guest `vsock:<guest_port>`
- `vm` 在 **非 Android host** 也支持 `--adb-tcp-port`
- 新增 `scripts/vm_shell_linux.sh` 的 `-AutoConnect`

代码入口：

- `packages/modules/Virtualization/android/virtmgr/src/aidl.rs`
- `packages/modules/Virtualization/android/virtmgr/src/crosvm/crosvm_unix.rs`
- `packages/modules/Virtualization/android/vm/src/main.rs`
- `packages/modules/Virtualization/android/vm/src/run.rs`

### 2.5 Linux host 路径映射补齐

Windows 侧已有 host path mapping；Linux host 现在也能使用同一套环境变量映射 APEX / system 运行时树：

- `VIRTMGR_APEX_ROOT`
- `VIRTMGR_SYSTEM_ROOT`
- `VIRTMGR_SYSTEM_EXT_ROOT`
- 可选 `VIRTMGR_MICRODROID_JSON`

代码入口：

- `packages/modules/Virtualization/libs/vmconfig/src/lib.rs`
- `packages/modules/Virtualization/android/virtmgr/src/aidl.rs`

---

## 3. 新增 Linux wrapper / regression 脚本

新增：

- `scripts/vm_linux.sh`
- `scripts/vm_shell_linux.sh`
- `scripts/check_linux_avf_markers.sh`
- `scripts/run_linux_avf_regression.sh`
- `scripts/microdroid_linux_raw.json`

### 3.1 `vm_linux.sh`

命令面与 Windows wrapper 对齐：

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

- 二进制：`out/dist/linux/bin`
- 动态库：`out/dist/linux/lib`
- APEX 树：`out/dist/apex_dir`

### 3.2 `vm_shell_linux.sh`

目前支持：

- `start-microdroid`
- `connect`

当使用 `-AutoConnect` 时，流程是：

1. 调 `vm_linux.sh -Command run-microdroid`
2. 传入 `--adb-tcp-port <port>`
3. 等待 `notifyPayloadReady`
4. `adb connect localhost:<port>`
5. 可选 `adb root`
6. 可选进入 `adb shell`

### 3.3 `run_linux_avf_regression.sh`

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

说明：

- baseline `run-microdroid` / `run-app` 按 smoke 方式运行；persistent `run-microdroid` 则要求 VM 真正到 READY 后自行 detach
- Linux regression 不再依赖 Windows 风格 `virtmgr-trace.log`，而是优先从 `vm-run-microdroid.log` / `guest-log.txt` 提取 marker
- persistent 场景下，CID 会从 run log 回推，而不是强依赖 trace
- regression 会记录初始 repo-local `crosvm` / `virtmgr` PID 集，并在 transient smoke 后清理由本轮新增的进程，避免后续 persistent 场景撞上前一轮 transient VM
- regression 入口会先清理旧的 repo-local regression `virtmgr` / `crosvm` 残留；否则历史失败运行留下的 persistent service 或 orphan `crosvm` 可能长期占住低位 CID（例如 4097-4102）

可选覆盖：

- `--include-run-app`
- `--include-adb-scenario`

---

## 4. Linux 使用方式

## 4.1 前置

先完成 host 构建：

```bash
./build_all.sh
```

Android gfxstream+ANGLE 窗口模式需要带 X11 display backend 的 crosvm：

```bash
ENABLE_GFXSTREAM_ANGLE=1 ./build_all.sh
```

该路径会启用顶层 `gpu,gfxstream,x` crosvm features。Linux host 通常需要
`libx11-dev`、`libxext-dev`、`libwayland-dev`、`wayland-protocols` 等构建依赖。

运行前至少应满足：

- `/dev/kvm` 存在
- 当前用户对 `/dev/kvm` 有读写权限
- `out/dist/linux/bin/{vm,virtmgr,crosvm}` 已生成
- `out/dist/apex_dir` 已就绪

快速检查：

```bash
./scripts/vm_linux.sh -Command validate-prereqs
```

## 4.3 Android gfxstream+ANGLE 实时窗口

启动 Android GPU 模式时必须提供可用 X11 display；如果没有 `DISPLAY`，显式传
`--x-display`：

```bash
DISPLAY=:1 ./scripts/run_android_linux.sh \
  --mode gpu \
  --gpu-guest-angle \
  --mem 8192 \
  --timeout-secs 420 \
  --x-display :1
```

验证分三步：

```bash
DISPLAY=:1 ./scripts/check_android_linux_host_window.sh \
  --log-dir out/dist/logs/android-linux \
  --x-display :1

./scripts/check_android_linux_gfx_markers.sh out/dist/logs/android-linux
./scripts/check_android_linux_gfx_screenshot.sh --log-dir out/dist/logs/android-linux
```

2026-06-26 验证要求：

- X11 host 窗口存在：`0x5400001 "crosvm" 1280x720+14+49`
- host window dump：`out/dist/logs/android-linux/host-window/crosvm-window.xwd`
- host window metrics：`out/dist/logs/android-linux/host-window/crosvm-window.metrics.txt`
- guest 截图：`out/dist/logs/android-linux/adb/gfxstream-angle.png`
- 亮色 Settings 验证指标：host `mean_luma=157.292`、`bright_ratio=0.673819`；
  guest 截图 1280x720 RGBA，785 unique colors，非空 bbox
- Android boot markers、gfxstream host init、guest ANGLE Vulkan markers 全部通过

只看到 guest screencap 或只看到空 host 窗口不算完整通过；必须同时有 host
窗口非黑像素证据和 guest screencap 证据。

`run_android_linux.sh --timeout-secs` 使用 GNU `timeout --foreground`，并将 crosvm
stdin 接到 `/dev/null`。不要去掉这两个行为；否则在交互式终端里 `timeout`
会把 crosvm 放入后台进程组，crosvm 触碰 tty 后会停在 `T` 状态，表现为窗口黑、
串口不增长。

## 4.2 基本命令

```bash
./scripts/vm_linux.sh -Command info
./scripts/vm_linux.sh -Command run-microdroid -KeepTemp
./scripts/vm_linux.sh -Command run-app -KeepTemp
./scripts/vm_linux.sh -Command run -Config ./scripts/microdroid_linux_raw.json -KeepTemp
./scripts/vm_linux.sh -Command create-partition -PartitionPath /tmp/writable.img -PartitionSize 1048576
./scripts/vm_linux.sh -Command create-idsig -OutputPath /tmp/app.idsig
```

## 4.3 persistent service

```bash
./scripts/vm_linux.sh -Command run-microdroid -PersistVirtmgr
./scripts/vm_linux.sh -Command list -PersistVirtmgr
./scripts/vm_linux.sh -Command console -PersistVirtmgr -Cid 2048 -- --read-only --timeout-secs 3
./scripts/vm_linux.sh -Command service-status -PersistVirtmgr
./scripts/vm_linux.sh -Command stop-service -PersistVirtmgr
```

## 4.4 vm_shell 风格 + ADB

```bash
./scripts/vm_shell_linux.sh -Command start-microdroid -AutoConnect -NoShell
./scripts/vm_shell_linux.sh -Command connect -AdbPort 8035
```

## 4.5 Android Cuttlefish guest on Linux crosvm

Linux host 现在也有直接启动 Android Cuttlefish x86_64 产物的脚本路径。该路径优先复用
`/opt/workspace/aosp/out/target/product/vsoc_x86_64` 中已经编译好的 AOSP 输出，不要求清理或全量重编
AOSP。

新增脚本：

- `scripts/create_cf_android_disk.py`
- `scripts/run_android_linux.sh`
- `scripts/check_android_linux_markers.sh`
- `scripts/build_angle_linux.sh`

基本 headless smoke：

```bash
./scripts/run_android_linux.sh --timeout-secs 180
./scripts/check_android_linux_markers.sh out/dist/logs/android-linux
```

当前验证结果：

- Linux `crosvm run` 可启动 Android guest。
- first-stage mount 可找到 Cuttlefish fstab 和聚合 GPT disk。
- system_server 到达 `OnBootPhase_1000`。
- init 触发 `sys.boot_completed=1`。
- `scripts/check_android_linux_markers.sh` 已通过。

关键差异：

- 脚本生成 Cuttlefish 风格 GPT 聚合盘：`misc`、A/B boot/vendor_boot/vbmeta、`super`、`userdata`、
  `frp`、`metadata`。
- `frp` 使用 1 MiB `factory_reset_protected.img`，与 Cuttlefish persistent composite disk 中的
  FRP 分区语义一致。没有该分区时，`PersistentDataBlockService` 会因
  `/dev/block/by-name/frp` 不存在而在 boot phase 500 超时，继而触发 system_server/zygote 重启。
- `external/crosvm/arch/src/android.rs` 需要为 Android DT string properties 写入 trailing NUL；
  AOSP `ReadDtFile()` 会去掉最后一个字节，缺少 NUL 会导致 fstab DT 字符串被截断。

GPU/gfxstream + ANGLE 路径：

```bash
./scripts/build_angle_linux.sh
./scripts/run_android_linux.sh --mode gpu --gpu-guest-angle --mem 8192 --timeout-secs 240
./scripts/check_android_linux_markers.sh out/dist/logs/android-linux
./scripts/check_android_linux_gfx_markers.sh out/dist/logs/android-linux
./scripts/check_android_linux_gfx_screenshot.sh --log-dir out/dist/logs/android-linux --cid 100
```

当前验证结果：

- `scripts/check_android_linux_markers.sh out/dist/logs/android-linux` 已通过。
- init 触发 `sys.boot_completed=1`。
- SurfaceFlinger 完成启动：`SurfaceFlinger: Boot is finished`。
- RenderEngine 使用 guest ANGLE：`ANGLE (NVIDIA, Vulkan 1.3.0 (... NVIDIA GeForce RTX 2060 ...))`。
- gfxstream host 后端初始化成功：`stream_renderer_init Gfxstream initialized successfully!`。
- gfxstream feature 中 `GuestVulkanOnly` 已启用，`VulkanAllocateHostMemory` 已禁用。
- surfaceflinger、bootanimation、Settings、SystemUI、Launcher3、system_server 均创建了 `engine:ANGLE` 的 Vulkan device。
- virtio-gpu 持续收到 scanout update 和 flush，说明 guest composition 产物已提交到 host GPU frontend。
- ADB over AF_VSOCK 可连接 guest `adbd`，`screencap` 可拉回 1280x720 RGBA PNG，像素检查为非空帧。

`--mode gpu` 要求 `ANGLE_RUNTIME_DIR` 指向包含 `libEGL.so` 和 `libGLESv2.so` 的 Linux ANGLE runtime；
默认 staging 目录是 `out/dist/linux/gfx/angle`。`scripts/build_angle_linux.sh` 会从 `~/angle` 构建并
stage ANGLE runtime，同时复制可选的 `libvulkan.so*`、`libvk_swiftshader.so*`、
`libVkICD_mock_icd.so*` 和 `vk_swiftshader_icd.json`。

`--gpu-guest-angle` 是当前通过验证的 Android 图形路径。该模式使用：

- guest bootconfig：`androidboot.hardware.egl=angle`
- crosvm GPU：`backend=gfxstream`
- display：`displays=[[mode=windowed[1280,720],dpi=[320,320],refresh-rate=60]]`
- gfxstream contexts：`gfxstream-vulkan:gfxstream-composer`
- `angle=true,gles=false,vulkan=true,wsi=vk`

`gles=false` 会让 crosvm/gfxstream 进入 guest ANGLE 的 Vulkan-only 组合，避免 direct crosvm
路径里 guest GL pipe 未就绪导致的 `eglMakeCurrent failed` / null context 问题。
`wsi=vk` 是当前 Linux windowed host 显示路径；`egl=true,surfaceless=true,glx=false`
的实验路径能初始化 host gfxstream，但 guest 会卡在早期启动阶段，因此不作为默认。

`--gpu-host-swiftshader` 只在需要强制使用 staged SwiftShader Vulkan ICD 时启用。默认不强制设置
`VK_ICD_FILENAMES`；本机验证使用 NVIDIA host Vulkan ICD 正常通过。

本轮还修复了两处 Linux direct crosvm + gfxstream guest ANGLE blocker：

- `external/crosvm` 允许 `backend=gfxstream,angle=true,gles=false`，并在该组合下设置
  `GuestVulkanOnly=true`、`VulkanAllocateHostMemory=false`。
- gfxstream host Vulkan 不再在 physical-device probe 阶段用 `VkPhysicalDevice` 调用需要
  `VkDevice` 的 `VK_EXT_external_memory_host` 查询；guest 发起
  `vkGetMemoryHostPointerPropertiesEXT` 时返回 `VK_ERROR_FEATURE_NOT_PRESENT`。

这些改动只复用 `/opt/workspace/aosp` 已有 AOSP 产物，没有 clean 或全量重编 AOSP。

`scripts/check_android_linux_gfx_markers.sh` 是 GPU 路径的更强验证入口。它会先运行基础
Android boot marker 检查，再要求：

- SurfaceFlinger boot finished
- RenderEngine 是 ANGLE over Vulkan
- gfxstream 初始化成功
- `GuestVulkanOnly` / `VulkanAllocateHostMemory` feature 状态正确
- surfaceflinger 和 Launcher3 都创建 ANGLE Vulkan device
- host virtio-gpu 看到 scanout update 与 flush
- 不出现已知的 gfxstream/ANGLE/Vulkan 失败：`eglMakeCurrent failed`、`null ctx`、
  `vkGetMemoryHostPointerPropertiesEXT`、Vulkan loader invalid device、SIGSEGV 等

`scripts/check_android_linux_gfx_screenshot.sh` 是当前最强的可视化验证入口。它要求 VM 仍在运行，
会临时启动 `127.0.0.1:<port> -> vsock:<cid>:5555` ADB bridge，执行：

```bash
adb shell screencap -p /data/local/tmp/gfxstream-angle.png
adb pull /data/local/tmp/gfxstream-angle.png out/dist/logs/android-linux/adb/gfxstream-angle.png
```

然后用 Pillow 检查 PNG：

- 尺寸必须为 1280x720
- `bbox` 不能为空
- 颜色数不能是单色
- RGB extrema 不能全为 0

最新通过结果：

```text
size=1280x720
mode=RGBA
bbox=(0, 0, 1280, 720)
unique_colors=7020
mean_rgba=[22.38, 24.0, 56.12, 255.0]
```

---

## 5. 日志与产物

每次 wrapper 运行都会聚合到单独目录，例如：

```text
out/dist/logs/linux-run-microdroid-<timestamp>/
  vm-run-microdroid.log
  virtmgr-trace.log
  vmclient-trace.log
  guest-log.txt
  vm-console.txt
  vm-console-in.txt
  work/
  temp/
```

persistent service 模式下，额外状态位于：

```text
<service-root>/
  virtmgr-service.state
  virtmgr-service.sock
  virtmgr-trace.log
```

注意：

- Linux host 目前很多关键 marker 实际更稳定地出现在 `vm-run-microdroid.log` 中，而不是 Windows 同款 trace 文件
- 当 `guest-log.txt` 为空时，应优先检查 `vm-run-microdroid.log`

---

## 6. 本轮额外 host bring-up 修复

除最初的 runtime split 以外，本轮还补了 Linux host 真运行所需的一批关键修复：

- `frameworks/native/libs/binder/CMakeLists.txt` 纳入 `Threads.cpp` / `file.cpp`，补齐 `android::Thread` 与 `ReadFully` 等运行时符号
- `system/core/libutils/Threads.cpp` 为 GCC/Linux host 增加 `_Nonnull` / `_Nullable` 兼容宏
- `frameworks/native/libs/binder/ProcessState.cpp` 修正 RPC-only host `ProcessState::self()` 初始化，避免 `startThreadPool()` 空对象崩溃
- Unix `libvmclient` 改为和 Windows 一样尊重 `VIRTMGR_PATH`
- unstable binder RPC Unix-domain client 支持 **绝对路径**，不再无条件拼 `/dev/socket/`
- Unix `virtmgr` / `crosvm` host 路径都改为尊重 wrapper 注入的环境变量，而不是硬编码 `/apex/...`
- `build_all.sh` 的 Linux host crosvm feature 集补齐 `composite-disk`，否则 Microdroid composite 映像会被当普通小文件盘处理
- persistent VM 的 crosvm control socket 改为 desktop-host 短路径，规避 Unix socket `ENAMETOOLONG`
- Linux `vm console` 不再依赖外部 `microcom`，而是内置 PTY attach
- desktop host `create_vm_context()` 的 CID / RpcServer 端口重试窗口放大，避免共享环境中低位 CID 被旧 VM 占满时过早失败

---

## 7. 当前语义边界

以下行为按要求维持与 Windows host 一致，不在本次实现范围内：

- permission service：**不实现**
- SELinux host-side parity：**不实现**
- 优先使用真实 vsock；只有难度过高或平台不支持时才保留替代方案

---

## 6. 不影响 Windows 逻辑的边界

本次改动遵守以下边界：

1. Windows 仍保留自己的命名管道 / WHPX / `crosvm_windows` 路径
2. Android 设备目标仍保留对真实 `VirtualizationServiceInternal` / permission / SELinux 的设备语义
3. Linux 新增能力统一以 **`cfg(not(target_os = "android"))`** 或 Unix host 路径实现，不回写 Windows transport 逻辑

---

## 7. 当前 Linux host 已知限制

1. 这仍是 **desktop host runtime**，不是 Android 设备环境；permission / SELinux / 内部系统服务语义不是 1:1 复刻。
2. `service-status` / `stop-service` 是 wrapper 层能力，不是 `vm` 原生命令。
3. host ADB bridge 依赖 guest 里确实有 `adbd` 并监听 `vsock:5555`。
4. 若未使用 wrapper 传入 file-backed console 路径，`vm console` 的跨命令体验可能退回到 `microcom` 模式。

---

## 8. 建议验证顺序

```bash
./scripts/vm_linux.sh -Command validate-prereqs
./scripts/vm_linux.sh -Command info
./scripts/vm_linux.sh -Command run-microdroid -KeepTemp
./scripts/vm_linux.sh -Command run-microdroid -PersistVirtmgr
./scripts/vm_linux.sh -Command list -PersistVirtmgr
./scripts/vm_linux.sh -Command console -PersistVirtmgr -Cid 2048 -- --read-only --timeout-secs 3
./scripts/vm_shell_linux.sh -Command start-microdroid -AutoConnect -NoShell
./scripts/run_linux_avf_regression.sh
```
