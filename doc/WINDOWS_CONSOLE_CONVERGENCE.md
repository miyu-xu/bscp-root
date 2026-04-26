# Windows VM Console Convergence

> Date: 2026-04-25
> Status: Phase A + Phase B implemented — bidirectional named pipe console (PH2-05)
> Target: Windows WHPX host

## Current State

Windows VM console is **bidirectional via named pipes**: console output is
streamed through `\\.\pipe\virtmgr_console_{cid}` and console input through
`\\.\pipe\virtmgr_console_input_{cid}`, providing near-PTY interactive behavior
without modifying the crosvm serial backend.

Compare:
- **Linux / macOS**: PTY/TTY interactive — `vm console` attaches to the guest's
  serial console with full bidirectional I/O.
- **Windows**: Bidirectional named pipe console — `read_console()` and
  `write_console()` provide real-time I/O; no terminal emulation.

## Root Cause

crosvm on Windows creates a **file-backed console** because:

1. No ConPTY in the crosvm Windows backend (serial console uses `File`).
2. virtmgr's `getConsoleOutput` AIDL method previously returned the console output file via
   `ParcelFileDescriptor` — read-only.
3. `stdin` for the VM on Windows was provided as a separate inherited handle
   (console input pipe), not a PTY master.

## Architecture Overview

```
Bidirectional Named Pipes (Phase A + Phase B — implemented):

   crosvm (Windows guest serial output)
     └── WriteFile → \\.\pipe\virtmgr_console_{cid}
                       └── console_pipe_read_loop() (background thread)
                             └── Arc<Mutex<Vec<u8>>> buffer
                                   └── read_console(max_len) → Vec<u8>

   write_console(data) → WriteFile → \\.\pipe\virtmgr_console_input_{cid}
                                       └── ReadFile → crosvm (Windows guest serial input)
```

Key insight: Windows `CreateFile` can open named pipes by path, so crosvm's
existing `type=file,path=\\.\pipe\...` serial backend works without any
modification to the crosvm Rust codebase.

## Phase A: Named Pipe Output Console (已实现)

### Implementation

Phase A is implemented in `crosvm_windows.rs`:

1. **Named pipe creation** (`create_console_named_pipe()`):
   - Creates `\\.\pipe\virtmgr_console_{cid}` via `CreateNamedPipeW`
   - Returns pipe path and server-side `OwnedHandle`
   - 64KB buffer for inbound data (guest→host console output)

2. **Reader thread** (`start_console_pipe()` on `VmInstance`):
   - Spawns `console_pipe_read_loop()` in a dedicated thread
   - Accepts client connection (crosvm opens the pipe via `type=file,path=`)
   - Reads console output in blocking mode, appending to shared `Arc<Mutex<Vec<u8>>>`

3. **Console serial arg override** (`run_vm()`):
   - When pipe path is available, replaces `type=file,path=<tempfile>` with `type=file,path=\\.\pipe\...`
   - Crosvm opens the named pipe client end and writes serial output to it

4. **API methods on `VmInstance`**:
   - `read_console(max_len: usize) -> Vec<u8>` — reads buffered console output

### What Phase A enables

- `vm console` can read guest console output in real-time
- Console output is buffered in-memory rather than only written to a file

## Phase B: Named Pipe Input Console (已实现)

Phase B adds **bidirectional I/O** by creating a second named pipe dedicated to
console input. This avoids the complexity of full ConPTY integration (which
would require crosvm serial backend refactoring).

### Implementation

Phase B extends `crosvm_windows.rs` with:

1. **Input pipe creation** (`create_console_input_pipe()`):
   - Creates `\\.\pipe\virtmgr_console_input_{cid}` via `CreateNamedPipeW`
   - Uses `PIPE_ACCESS_OUTBOUND` — virtmgr writes keyboard input, crosvm reads it
   - 64KB buffer for outbound data
   - Returns pipe path and server-side `OwnedHandle`

2. **Connection thread** (`start_console_pipe()`):
   - Spawns a thread that calls `ConnectNamedPipe` on the input pipe
   - Passes the raw handle (`isize`, `Copy`) to the thread while keeping
     the `OwnedHandle` on `VmInstance` for later `WriteFile` calls
   - Handles `ERROR_PIPE_CONNECTED` (535) for already-connected clients
   - Thread stays alive (sleeps in 1-hour intervals) to keep the pipe server end open

3. **`write_console(data)` — fully implemented**:
   - Acquires the `console_input_pipe` handle from `VmInstance`
   - Calls `WriteFile` to forward data to the input pipe
   - Crosvm reads from the pipe as standard serial input
   - Returns `io::ErrorKind::NotConnected` if pipe is unavailable
   - Returns `io::ErrorKind::BrokenPipe` on write failure

4. **Serial arg construction** (`run_vm()`):
   - Appends `,input=\\.\pipe\virtmgr_console_input_{cid}` to the serial argument
   - Falls back to `console_in_fd` (file path) when pipe is unavailable

### Architecture

```
Phase A: Output pipe (Phase A — 已实现)
┌─────────────────┐     WriteFile      ┌──────────────────────────────┐
│  crosvm.exe     │ ──────────────────→│ \\.\pipe\virtmgr_console_X  │
│  (serial out)   │                    │  (CreateNamedPipeW Inbound)  │
└─────────────────┘                    └──────────┬───────────────────┘
                                                  │ ConnectNamedPipe
                                                  ↓
                                          console_pipe_read_loop()
                                          (background thread)
                                                  │
                                                  ↓
                                          Arc<Mutex<Vec<u8>>>
                                          (shared buffer)
                                                  │
                                                  ↓
                                          read_console(max_len)
                                          → Vec<u8>

Phase B: Input pipe (Phase B — 已实现)
                                          write_console(data)
                                                  │
                                                  ↓
                                          Arc<Mutex<OwnedHandle>>
                                          (input pipe write end)
                                                  │ WriteFile
                                                  ↓
┌─────────────────┐      ReadFile      ┌──────────────────────────────┐
│  crosvm.exe     │ ←──────────────────│ \\.\pipe\virtmgr_console_   │
│  (serial in)    │                    │       input_X                │
│  via ,input=... │                    │  (CreateNamedPipeW Outbound) │
└─────────────────┘                    └──────────────────────────────┘
```

### Key Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Two pipes vs one ConPTY** | Two named pipes | Avoids crosvm serial backend changes; uses existing `type=file,path=` |
| **Pipe direction** | Output=PIPE_ACCESS_INBOUND, Input=PIPE_ACCESS_OUTBOUND | Correct flow: guest writes→host reads output; host writes→guest reads input |
| **OwnedHandle lifetime** | Stored on `VmInstance`, raw handle passed to connection thread | `OwnedHandle` is `!Clone`; raw handle (`isize`) is `Copy` |
| **Connection thread** | Thread stays alive with 1h sleeps | Keeps named pipe server end open for crosvm |
| **No ConPTY** | Not implemented | Avoids `CreatePseudoConsole` + crosvm startup handle injection (5-7 day task) |

### What Phase B enables

- Full bidirectional console I/O between host and guest
- `vm_shell_windows.ps1` can use `read_console()` / `write_console()` for interactive sessions
- Near-PTY experience without modifying crosvm serial backend
- Terminal emulation (escape sequences, resize) is not provided but raw byte transport works

### Known Limitations

| Limitation | Impact | Workaround |
|-----------|--------|------------|
| No terminal emulation | Escape sequences pass through as raw bytes | Client-side terminal handling |
| No resize events | Guest has fixed terminal size | Can be added via virtio-console resize |
| No AIDL readConsole/writeConsole | Not exposed via RPC yet | Direct `VmInstance` methods available |
| No PowerShell wrapper | `vm_shell_windows.ps1` not updated for interactive mode | Can be done as follow-up |

### Effort

| Step | Status | Effort | Risk |
|------|--------|--------|------|
| Phase A: Output pipe creation + reader thread | ✅ Complete | 1 day | Low |
| Phase A: `run_vm()` pipe path integration | ✅ Complete | 0.5 day | Low |
| Phase A: `read_console()` API | ✅ Complete | 0.5 day | Low |
| Phase B: Input pipe creation (`PIPE_ACCESS_OUTBOUND`) | ✅ Complete | 0.5 day | Low |
| Phase B: Connection thread with raw handle sharing | ✅ Complete | 0.5 day | Low |
| Phase B: `write_console()` implementation | ✅ Complete | 0.5 day | Low |
| Phase B: `run_vm()` serial arg with `,input=` | ✅ Complete | 0.5 day | Low |

## ConPTY Alternative (Not Implemented)

Windows 10 1809+ includes the **ConPTY API** (`CreatePseudoConsole`,
`ClosePseudoConsole`) that provides:

- A PTY-like pipe pair: one end acts as a terminal (owned by a console window),
  the other acts as a pipe to the child process (crosvm).
- Bidirectional I/O: ConPTY handles `stdin` input and `stdout`/`stderr` output
  through a single `HPCON` handle.
- Terminal emulation — escape sequences, resize, etc.

This was considered but **not chosen** because:
1. Requires crosvm Windows serial backend refactoring to accept pre-opened handles
2. 5-7 day implementation effort vs. 1-2 day bidirectional named pipe approach
3. The bidirectional named pipe approach achieves the same functional result
   for raw byte transport

### ConPTY Architecture (for reference)

```
Proposed (ConPTY — not implemented):
   vm_shell.ps1
     └── CreatePseudoConsole → HPCON
           ├── ConPTR (input pipe) ← crosvm stdin
           └── ConPTY (output pipe) → crosvm stdout/stderr
     └── GetConsoleWindow → attach to ConPTY for interactive mode
```

### ConPTY Implementation (if needed in future)

ConPTY integration would require changes to `crosvm_windows.rs`:
1. Create `HPCON` via `CreatePseudoConsole` instead of named pipes
2. Set crosvm's `STARTUPINFOEX.hStdInput`/`hStdOutput` to ConPTY pipe handles
3. Store `HPCON` handle in `VmInstance` for `write_console()`/`read_console()`

This is deferred indefinitely since the named pipe approach addresses the core
requirement without crosvm serial backend changes.

## Dependencies

| Depends on | Why | Status |
|-----------|-----|--------|
| Windows named pipe support | `CreateNamedPipeW` / `ConnectNamedPipe` / `WriteFile` / `ReadFile` | ✅ Available on all Windows versions |
| crosvm `type=file,path=` serial backend | Accepts named pipe paths since `CreateFile` can open `\\.\pipe\...` | ✅ Used as-is, no changes needed |
| `std::os::windows::io` | `OwnedHandle`, `RawHandle`, `FromRawHandle` for safe handle management | ✅ Standard library |

## Future Work

| Item | Effort | Priority | Notes |
|------|--------|----------|-------|
| AIDL `readConsole`/`writeConsole` methods | 1-2 days | P2 | Expose via virtmgr RPC |
| `vm_shell_windows.ps1` interactive console | 1-2 days | P2 | PowerShell wrapper using `read_console()`/`write_console()` |
| Terminal emulation (escape sequences, resize) | 3-5 days | P3 | Full PTY parity with Linux/macOS |
