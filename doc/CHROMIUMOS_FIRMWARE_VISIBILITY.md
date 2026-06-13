# ChromiumOS 固件可见性（下一步计划结果）

Date: 2026-05-29

## 目标

在 WHPX IO 修复后，确认 OVMF 是否执行、是否看到 ChromeOS 磁盘、是否进入 depthcharge/vboot。

## 结论摘要

| 检查项 | 结果 |
|--------|------|
| WHPX IO 风暴 | 已修复（见 `CHROMIUMOS_WHPX_IO_FIX.md`） |
| OVMF 串口 `ttyS0` (RELEASE) | **0 字节**（Debian 发布版 OVMF 通常不向 COM1 打日志） |
| OVMF DEBUG + serial | **5617 字节** — 越过 PEI ASSERT，PlatformPei + CpuMpPei 后 **挂起**（未进 BDS） |
| OVMF debugcon `0x402` | **0 字节**（DEBUG_ON_SERIAL_PORT 构建不写 debugcon） |
| GPT 磁盘 | **正常**：含 `EFI-SYSTEM`（~128MB）、`KERN-A`、`ROOT-A` |
| `bootindex=1` | direct crosvm 脚本已有；**virtmgr 已补** `bootindex=1` |
| depthcharge 控制台 | 设计为 UEFI 链 + 可能帧缓冲；需 GPU 或 DEBUG 固件 |

## 固件日志：串口 vs debugcon

标准 OVMF（含 Debian `ovmf` 包）行为（edk2 `OvmfPkg/README`）：

- **默认**：调试信息写到 **ISA debugcon 端口 `0x402`**，不是 `0x3F8`。
- **串口**：需编译时 `-D DEBUG_ON_SERIAL_PORT`，且 QEMU/crosvm 需捕获 serial。
- **RELEASE 构建**：**关闭**所有 DebugLib 输出 → debugcon 与 serial 均为空是预期现象。

crosvm 捕获方式：

```text
--serial type=file,path=debugcon.log,hardware=debugcon,num=3,debugcon_port=402
--serial type=file,path=serial.log,hardware=serial,num=1,earlycon=true
```

`scripts/run_crosvm_chromeos_firmware.ps1` 默认启用 debugcon（`-NoDebugcon` 可关）。

## 磁盘与启动顺序

`scripts/inspect_chromeos_gpt.ps1` 对 R150 `amd64-generic_test_image.bin` 可见：

| Index | Name | 说明 |
|-------|------|------|
| 12 | EFI-SYSTEM | UEFI / depthcharge（FAT） |
| 2 | KERN-A | ChromeOS 内核分区 |
| 3 | ROOT-A | rootfs |

`bootindex=1` 应让 OVMF 优先从该 virtio block 启动。

## depthcharge 控制台路径

ChromeOS test image 的 **EFI-SYSTEM** 为 FAT + **SYSLINUX** → **depthcharge**（UEFI）。分区起始可见 `SYSLINUX` / `EFI-SYSTEM FAT16`（`scripts/inspect_chromeos_efi.ps1`）。镜像内亦含 `Boot error` 静态字符串（SYSLINUX 失败提示；VM 内是否出现需 GPU/串口确认）。

可能输出路径：

1. **UEFI 图形** — OVMF Boot Manager / depthcharge UI（需 `--gpu backend=gfxstream`）
2. **串口** — 内核阶段 `console=ttyS0`；固件阶段未必用 COM1
3. **vboot 验签失败** — 可能无输出即挂起

## DEBUG OVMF 结果（2026-05-29）

构建：`scripts/build_ovmf_debug.ps1`（WSL）或 GitHub Actions `build-ovmf-debug.yml`  
产物：`out/dist/firmware/OVMF_DEBUG.fd`（4MB，sha256 `7e4fc0b2…`）

测试：

```powershell
.\scripts\run_crosvm_chromeos_firmware.ps1 -Firmware ovmf-debug -TimeoutSecs 180
.\scripts\classify_ovmf_debug_log.ps1 -LogDir out\dist\logs\cros-fw-ovmf-debug
```

串口末尾（`guest-serial-num1.txt`）：

```text
Loading PEIM at 0x0000082CB40 EntryPoint=0x0000082FDB2 PcdPeim.efi
...
ASSERT [PeiCore] IoLibGcc.c(211): (Port & 3) == 0
```

**分类（修复前）**：`reached_pei` → **`pei_assert_io_alignment`**  
固件在 PEI 早期因 **未对齐的 IO 端口访问** 触发 ASSERT，未进入 DXE/BDS。

**根因（2026-05-29）**：crosvm PCI root 仿真为 i440 (`0x1237`) 且无 ICH9 LPC `00:1f.0`。OVMF `AcpiTimerLibConstructor` 读 LPC PMBASE 得 `0xFFFFFFFF` → ACPI timer IO `0x6`（未对齐）→ `IoRead32(0x6)` ASSERT。

**修复**：`external/crosvm/devices/src/pci/pci_root.rs`
- Root bridge → Q35 MCH `0x29c0`
- 桩 ICH9 LPC `00:1f.0`（`0x2918`），PMBASE `0x601`，ACPI_CNTL `0x80`（与 crosvm `pm_iobase=0x600` 一致）

**分类（修复后）**：`reached_dxe`（启发式匹配 `DxeIpl.efi`）；串口末尾停在 **CpuMpPei**（`AP Loop Mode is 1`，`BootCpuCount=0`），180s 无新输出 → **下一阻塞点：CpuMpPei / AP bring-up（WHPX 单 vCPU）**。

```text
Loading PEIM at 0x000CFF4F000 EntryPoint=0x000CFF54780 CpuMpPei.efi
...
CpuMpPei: 5-Level Paging = 0
```

**下一阻塞点**：CpuMpPei 在 WHPX 下单 vCPU 环境挂起（可能缺 fw_cfg CPU 数、APIC/INIT-SIPI 或 MP 表）。Release OVMF 可能静默挂在同一点。

## 推荐下一步（按优先级）

1. **调查 CpuMpPei 挂起**（PEI ASSERT 已修复）  
   - 对比 `--cpus 1` vs 多 CPU；是否需启用 fw_cfg（Windows 默认 `fw_cfg_enable: false`）  
   - WHPX LAPIC/INIT-SIPI 与 OVMF `BootCpuCount=0` 路径

2. **带 gfxstream 的 crosvm**（越过 CpuMpPei 后的 UI 路径）  
   ```powershell
   .\scripts\build_chromeos_firmware_gfx.ps1   # ENABLE_GFXSTREAM_ANGLE=1 + in-repo aemu
   .\scripts\run_chromeos_firmware_gfx.ps1 -TimeoutSecs 180
   ```
   当前主机探测（2026-05-29）：`crosvm.exe` **未**带 `gfxstream` feature（`unknown variant gfxstream, expected 2D`），`out\dist\windows\bin\gfxstream_backend.dll` 不存在。gfxstream CMake 在 Windows 上还可能需要 `external/libdrm`（见 `scripts/probe_gfxstream.ps1`）。

2. **自编译 DEBUG OVMF**（已完成）  
   ```powershell
   .\scripts\build_ovmf_debug.ps1
   .\scripts\fetch_ovmf_debug.ps1 -Build
   ```

3. **virtmgr 端到端**（bootindex 已对齐；待 CpuMpPei 挂起解决后再测）  
   ```powershell
   .\scripts\run_chromeos_firmware.ps1
   ```

## 脚本索引

| 脚本 | 用途 |
|------|------|
| `scripts/build_ovmf_debug.sh` / `.ps1` | WSL/CI 构建 OVMF_DEBUG.fd |
| `scripts/fetch_ovmf_debug.ps1` | 获取/构建 DEBUG 固件 |
| `scripts/classify_ovmf_debug_log.ps1` | 解析串口启动阶段 |
| `scripts/run_crosvm_chromeos_firmware.ps1` | direct crosvm OVMF+磁盘 |
| `scripts/run_chromeos_firmware.ps1` | virtmgr 路径 |
| `scripts/build_chromeos_firmware_gfx.ps1` | 启用 gfxstream 重建 crosvm |
| `scripts/run_chromeos_firmware_gfx.ps1` | OVMF+磁盘+gfxstream |
| `scripts/probe_gfxstream.ps1` | 检测 gfxstream 是否可用 |
| `scripts/inspect_chromeos_gpt.ps1` | 列出 GPT / EFI 分区 |
| `scripts/run_crosvm_ovmf_min_repro.ps1` | 最小 OVMF-only 复现 |

## 成功标准（本阶段）

- [x] 明确发布版 OVMF 为何无串口/debugcon 输出  
- [x] 确认镜像含 EFI-SYSTEM / KERN-A / ROOT-A  
- [x] virtmgr 与 direct crosvm 均带 `bootindex=1`  
- [x] 通过 DEBUG OVMF 获得**可观测**固件进展（SEC/PEI；阻塞于 PEI IO 对齐 ASSERT）
- [ ] 越过 PEI 进入 BDS / 磁盘枚举 / depthcharge
