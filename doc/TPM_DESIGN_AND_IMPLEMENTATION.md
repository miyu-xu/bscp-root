# TPM Design and Implementation

Date: 2026-06-05

## 1. Motivation

ChromeOS requires a detectable TPM 2.0 device to boot without triggering security shutdown. The upstream crosvm uses virtio-TPM with external backends (swtpm process or vtpmd D-Bus service), neither of which is available on Windows/WHPX. We built a self-contained, in-process TPM TIS MMIO device with a minimal command backend.

## 2. Architecture

### 2.1 QEMU-Inspired Split Design

```
Guest Kernel (tpm_tis driver)
    |
    | MMIO read/write at 0xFED40000
    v
TpmTisDevice (FIFO frontend)
    |  - TIS register state machine
    |  - Locality support
    |  - 64-byte burst count
    v
TpmBackend trait (pluggable)
    |
    v
MinimalTpm (in-memory backend)
    |  - 23+ TPM2 command handlers
    |  - NV storage in BTreeMap
    |  - File-based persistence
    v
```

### 2.2 Why Not virtio-TPM?

| Aspect | virtio-TPM (upstream) | TIS MMIO (ours) |
|--------|----------------------|-----------------|
| Transport | virtio PCI queue | MMIO at 0xFED40000 |
| Backend | External swtpm or vtpmd | Built-in MinimalTpm |
| Windows support | No (no swtpm on Windows) | Yes |
| ACPI detection | None needed | TPM2 SDT + MSFT0101 DSDT |
| NV persistence | Backend responsibility | File-based in MinimalTpm |

## 3. Frontend: TpmTisDevice

### 3.1 Register Map (locality 0)

| Offset | Register | Purpose |
|--------|----------|---------|
| 0x0000 | ACCESS | Locality request/grant |
| 0x0008 | INT_ENABLE | Interrupt enable |
| 0x0018 | STS | Status (commandReady, dataAvail, expect, burstCount) |
| 0x0024 | DATA_FIFO | 1-byte FIFO for cmd/response |
| 0x0030 | INTERFACE_ID | TIS + FIFO + TPM 2.0 (0x00000030) |
| 0x0F00 | DID_VID | Google VID + device 1 (0x00011AE0) |

### 3.2 FIFO State Machine

```
Idle → [write commandReady] → Ready
     → [write FIFO bytes]    → Reception (STS_EXPECT set)
     → [write tpmGo]         → Execution
     → backend.execute_command()
     → Completion (STS_DATA_AVAIL set)
     → [read FIFO bytes]     → drain response
     → [write commandReady]  → Ready
```

### 3.3 Multi-Command Handling

The kernel may write multiple TPM commands in a single FIFO cycle (e.g., TPM2_GetCapability followed by TPM12_GetCapability during retry). The frontend processes commands sequentially, keeping the response from the last valid command.

### 3.4 ACPI Integration

- **TPM2 SDT**: Required by Linux kernel's `check_acpi_tpm2()` for TPM 2.0 detection
- **DSDT Device**: `MSFT0101` at `_HID`, standard x86 TPM device identifier
- **Memory Resource**: `Memory32Fixed` at 0xFED40000, size 0x5000

## 4. Backend: MinimalTpm

### 4.1 NV Persistence

```
MinimalTpm.nvram_path: Option<PathBuf>
    |
    v
with_nvram_path(path) → load from file on startup
    |
    v
save_nvram() → write to file on: NV_Write, NV_DefineSpace, TPM2_Shutdown
```

Default path: `$TPM_NVRAM_PATH` or `$USERPROFILE/crosvm_tpm_nvram.bin`

**File format** (binary, little-endian):
```
[count: u32][for each space: index:u32, data_size:u16, attributes:u32, data:bytes]
```

### 4.2 Command Handlers

| Category | Commands | Implementation |
|----------|----------|---------------|
| **Probe** | TPM2_Startup, TPM2_SelfTest, TPM2_GetCapability, TPM2_GetRandom | Full |
| **NV Operations** | NV_ReadPublic, NV_Read, NV_Write, NV_DefineSpace, NV_UndefineSpace, NV_WriteLock | Full |
| **PCR** | PCR_Read, PCR_Extend | Minimal (dummy values) |
| **HMAC** | HMAC_Start, HMAC, SequenceUpdate, SequenceComplete | Minimal (dummy keys) |
| **Object** | CreatePrimary, EvictControl, FlushContext, ContextLoad | Minimal (dummy handles) |
| **Hierarchy** | HierarchyControl | Always success |
| **Clear** | Clear, ClearControl, Shutdown, StirRandom | Always success |
| **Default** | Any other command | Success with empty response |

### 4.3 NV Index Handling

Key ChromeOS NV indices:
- **8388613** (0x00800005): Encstateful encryption key (40 bytes)
- **8388612** (0x00800004): Lockbox credentials (variable)

Parsing challenges:
- NV_ReadPublic: NV index is the auth handle at bytes 10-13 (fixed position)
- NV_Read/Write: NV index is within variable-length auth block — fallback to iterating defined spaces
- NV_DefineSpace: Complex command layout with variable-length auth — fallback to pre-creating known indices

### 4.4 Response Codes

| Code | Name | When |
|------|------|------|
| 0x000 | TPM_RC_SUCCESS | Normal success |
| 0x08F | TPM_RC_COMMAND_CODE | Unknown command |
| 0x084 | TPM_RC_VALUE | Invalid parameter |
| 0x100 | TPM_RC_INITIALIZE | Not started |
| 0x14A | TPM_RC_NV_UNINITIALIZED | NV space not defined |
| 0x148 | TPM_RC_NV_LOCKED | NV space is write-locked |

## 5. ChromeOS Boot Interaction

### 5.1 Boot Sequence

```
Kernel: tpm_tis driver probe → TPM2_GetCapability (repeated)
    → TPM detected as MSFT0101:00, 2.0 TPM
    |
Userspace: chromeos_startup
    → TPM2_SelfTest → OK
    → NV_ReadPublic(8388613) → TPM_RC_NV_UNINITIALIZED
    → NV_DefineSpace(8388613, 40) → OK, save_nvram()
    → NV_Write(8388613, key_data) → OK, save_nvram()
    → Format encstateful → mount
    → TPM2_Shutdown → save_nvram()
    → ACPI reboot
    |
Next boot:
    → NV_ReadPublic(8388613) → OK (persisted from file)
    → NV_Read(8388613) → key_data
    → Mount encstateful with key → boot continues
```

### 5.2 Without NV Persistence (Before Fix)

Each crosvm restart wiped all NV data → every boot looked like "first boot" → ChromeOS formatted encstateful and rebooted → infinite loop.

### 5.3 Workaround

`rw` root + `cros_debug` kernel params make ChromeOS write the encryption key to `/mnt/stateful_partition/encrypted.key` on disk instead of TPM NVRAM, bypassing the persistence requirement.

## 6. TPM2_CC_SHUTDOWN Handling

ChromeOS may reboot via direct ACPI reset without clean TPM2_Shutdown. To handle this, NV data is persisted immediately on every write:

```
NV_Write → save_nvram()
NV_DefineSpace → save_nvram()
TPM2_Shutdown → save_nvram()
```

## 7. Future Improvements

### 7.1 TPM 2.0 Compliance Gaps

- No session-based authorization (all commands use password auth)
- No policy enforcement
- No proper key hierarchy (all keys are dummy)
- No real random number generation (returns fixed pattern 0x42)
- PCR values are all zeros (not real measurements)

### 7.2 Potential Enhancements

- Integrate with swtpm for full TPM 2.0 compliance
- Add TPM2_ReadClock, TPM2_GetTime for time-critical ChromeOS features
- Implement TPM2_PolicyXXX commands for proper authorization
- Add virtio-TPM as alternative backend when swtpm is available
- Session-based HMAC for encrypted command/response

## 8. Files

| File | Lines | Purpose |
|------|-------|---------|
| `devices/src/tpm_tis.rs` | 1086 | TpmTisDevice frontend + MinimalTpm backend |
| `x86_64/src/lib.rs` | +14 | MMIO registration, ACPI TPM2 table, NVRAM path |

## 9. Testing

- ChromeOS kernel driver probes TPM via TIS FIFO → detects TPM 2.0
- `TPM ready` log from chromeos_startup confirms userspace access
- NV Read/Write/Define tested via ChromeOS encstateful encryption
- NV persistence tested across VM reboots
- Fallback path (no TPM_NVRAM_PATH env vars) tested on Windows
