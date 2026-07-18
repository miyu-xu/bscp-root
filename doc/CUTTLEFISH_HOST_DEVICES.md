# Cuttlefish 跨平台宿主设备

本文说明 Linux、Windows 和 macOS 共用的 Cuttlefish 宿主设备实现。Android 镜像保持纯 ARM64；宿主工具按宿主 OS/CPU 单独打包，不把宿主可执行文件混入 Android 镜像。

## 已实现范围

1. `cf_hvc_bridge.py`：HVC 与 TCP 双向桥接。Linux/macOS 使用文件或 FIFO，Windows 使用命名管道，并支持断线重连。
2. Bluetooth：RootCanal 提供 H4/TCP，桥接到 Android `hvc5`。
3. NFC：Casimir 提供 NCI/TCP，桥接到 Android `hvc12`。
4. Modem：`modem_simulator_host.py` 在 Linux 使用 AF_VSOCK、macOS 使用兼容 UDS、Windows 使用命名管道，复用同一套 framed-AT 状态机。
5. Sensors：`sensors_simulator_host.py` 实现 Cuttlefish RawMessage framing、传感器枚举以及加速度计/陀螺仪/磁力计基础数据，连接 `hvc13`。
6. `cf_host_devices.py`：统一拉起、端口探活、ready JSON、日志、子进程故障传播以及退出清理。

## 跨平台 Stub 回退

没有原生 RootCanal/Casimir 时，三端默认自动使用以下 Python stub：

- `root_canal_stub.py`：H4/TCP、确定性 BD_ADDR、最小 LE-only HCI 控制面。Android 可以完成控制器枚举和初始化，但不提供真实扫描、广播、配对或 ACL 对端。
- `casimir_stub.py`：NCI 2.0 reset/init/config、RF discovery 控制面。Android 可以完成 NFCC 初始化，但不会生成标签、读卡结果或 RF peer。
- `modem_simulator_host.py`：framed-AT modem stub。
- `sensors_simulator_host.py`：加速度计、陀螺仪、磁力计 stub。

`host-devices-ready.json` 的 `backends` 字段会逐设备标记 `native`、`stub` 或 `disabled`。使用 `cf_host_devices.py --require-native` 可禁止 Bluetooth/NFC 回退，适合正式验收。

## 运行入口

- Linux：`scripts/run_android_linux.sh` 与 `scripts/start_cf_host_daemons.sh`
- Windows：`scripts/run_android_windows_gfxstream_angle.ps1`
- macOS：`scripts/run_android_macos.sh`

macOS 示例：

```bash
scripts/run_android_macos.sh --work-dir out/runtime/android-macos
```

按需关闭设备：

```bash
scripts/run_android_macos.sh --no-bluetooth --no-nfc
```

宿主服务写入 `<log-dir>/host-devices/`，就绪状态写入 `<work-dir>/host-devices-ready.json`。任何已启动子进程提前退出都会令统一 supervisor 返回失败。

## 打包

生成四个轻量目录：

```bash
python3 scripts/package_cf_host_tools.py \
  --output-root out/dist/host-tools \
  --all-targets
```

目标为 `linux-x86_64`、`linux-arm64`、`windows-x86_64`、`darwin-arm64`。每个目录都有带 SHA-256 的 `manifest.json`。

RootCanal 与 Casimir 是宿主原生程序，必须为每个目标提供对应二进制。正式发布时使用严格模式，缺少任一引擎即失败：

```bash
python3 scripts/package_cf_host_tools.py \
  --output-root out/dist/host-tools \
  --target darwin-arm64 \
  --native-bin-dir /path/to/darwin-arm64/bin \
  --require-native
```

当前源码树没有 RootCanal/Casimir 源码或预编译文件时，非严格打包会包含跨平台 stub，并在 manifest 中把两个 native 文件标记为 `available: false`。stub 可用于启动和控制面调试，但不等同于可发布的 Bluetooth/NFC 完整无线功能。

## 镜像要求

纯 ARM64 Android 17 镜像可直接用于后续调试，只要 guest 配置保留 Cuttlefish 的 HVC 编号与 modem binder-rpc-vsock 端口。宿主设备实现不要求把 x86_64 文件聚合进镜像；若 guest 端删改了 `hvc5`、`hvc12`、`hvc13` 或 modem bootconfig，才需要重新打包镜像。

## 快速验证

```bash
python3 -m py_compile \
  scripts/cf_hvc_bridge.py \
  scripts/cf_host_devices.py \
  scripts/modem_simulator_host.py \
  scripts/sensors_simulator_host.py
python3 -m unittest -v tests.test_cf_host_devices
bash -n scripts/run_android_macos.sh scripts/start_cf_host_daemons.sh
```

macOS 可先用 `scripts/run_android_macos.sh --work-dir <dir> --dry-run` 检查最终 crosvm 命令，不复制大镜像、不产生额外磁盘副本。
