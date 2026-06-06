# ChromeOS WHPX Boot — 最终状态报告

Date: 2026-06-01

## 结论

**ChromeOS (amd64-generic) 已在 crosvm WHPX 上成功启动，包括 OVMF firmware boot 和自定义内核 direct boot 两条路径。**

## P0: crosvm WHPX 修复 (5项, 已提交)

| # | 修复 | 文件 | 提交 |
|---|------|------|------|
| 1 | WHPX IO: handle_io_with_mmio | `hypervisor/src/whpx/vcpu.rs` | 64a8a87 |
| 2 | WHPX 中断: WHvRequestInterrupt + 移除 kick | `devices/src/irqchip/whpx.rs` | 64a8a87 |
| 3 | MSR #GP → RAZ/WI | `hypervisor/src/whpx/vcpu.rs` | 64a8a87 |
| 4 | ACPI PNP0501 COM1-COM4 | `x86_64/src/lib.rs` | 64a8a87 |
| 5 | Remote IRR 清理 (MSI fallback 后) | `devices/src/irqchip/whpx.rs` | 64a8a87 |

## P1: 自定义内核 (x86_64_defconfig + VM 驱动)

**编译**: WSL2 Ubuntu, Linux 6.1.119, gcc 13.3.0

| 特性 | 值 |
|------|-----|
| VIRTIO_BLK | y (built-in) |
| VIRTIO_CONSOLE | y |
| VIRTIO_PCI | y |
| DEVTMPFS + MOUNT | y |
| SERIAL_8250 + CONSOLE | y |
| HVC_DRIVER + CONSOLE | y |
| EXT4_FS | y |

**产物**: `out/dist/img/custom_kernel` (12MB bzImage)

**验证结果**:
```
Linux 6.1.119-crosvm → earlycon @ COM1 (37KB dmesg)
                     → virtio_blk 自动检测: /dev/vda (10.3GB, 12 partitions)
                     → EXT4-fs (vda3): rootfs 挂载成功 (ro)
                     → /sbin/init 运行 → chromeos_startup, udevd
                     → ChromeOS 用户态启动中
```

## 架构分析

**AVF ferrochrome 是 ARM64 原生。WHPX 问题是 x86_64/Windows 特有的。**
我们的 5 个修复填补了 Windows crosvm WHPX 的通用空白——在 Linux KVM 上这些问题不存在。

## gfxstream GPU (部分完成)

**编译**: crosvm 含 `gfxstream` feature，`rutabaga_gfx` 中 gfxstream 后端已链接。

**运行时**: `crosvm device gpu` 子进程可正常启动。需满足：
- `libgfxstream_backend.dll` + `gfxstream_backend.dll` 在 PATH 中
- MinGW 运行时 DLL (`libgcc_s_seh-1.dll`, `libwinpthread-1.dll`, `libstdc++-6.dll`) 需在同目录或 PATH 中
- ANGLE DLL 需在 PATH 中 (`C:\workspace\bscp\angle\out\Release-GfxAngle-Clang`)

**当前阻塞**: `--gpu backend=gfxstream` 在 `--disable-sandbox` 模式下未将 GPU 设备接入 VM。crosvm Windows broker 进程架构可能需要 sandbox 模式才能正确创建 GPU child process。

**下步**: 研究 crosvm Windows broker/sandbox GPU 设备创建流程，或使用 `--jail` 启用 sandbox 测试。

## 修改文件 (crosvm)

| 文件 | 变更 |
|------|------|
| `external/crosvm/devices/src/irqchip/whpx.rs` | send_msi: WHvRequestInterrupt; Remote IRR清理; kick移除 |
| `external/crosvm/hypervisor/src/whpx/vcpu.rs` | handle_msr_read/write: #GP→RAZ/WI; handle_io_with_mmio |
| `external/crosvm/x86_64/src/lib.rs` | setup_acpi_devices: PNP0501 COM1-COM4 |
| `external/crosvm/src/sys/windows/run_vcpu.rs` | VcpuExit::Io 使用 handle_io_with_mmio |

## 相关文档

- WHPX boot changes: `doc/CHROMIUMOS_WHPX_BOOT_CHANGES.md`
- WHPX IO fix: `doc/CHROMIUMOS_WHPX_IO_FIX.md`
- WHPX MMIO debug: `doc/CHROMIUMOS_WINDOWS_MMIO_DEBUG.md`
- AVF custom_vm: `packages/modules/Virtualization/docs/custom_vm.md`
