# Windows Host-local pipe descriptor contract — open gate

The byte transport and zero-descriptor legacy framing are implemented separately. They do not provide descriptor capability and must not be advertised as full Host compatibility.

## Existing protocol and concrete gap

The r14 local pipe envelope contains little-endian 32-bit payload size and handle count, followed by 64-bit **receiver-process** handle values and payload. The sender duplicates a handle into the receiver and sends the resulting numeric value. Existing code narrows that value to an int and accepts values without provenance.

A valid handle number, object type, SID, or authenticated peer process does not prove that a particular receiver handle was created by this transfer. An attacker-controlled numeric value could select an existing receiver handle. Rejecting an invalid packet and closing that value could close an unrelated receiver resource. This is not repaired by widening the integer or calling GetHandleInformation. The new zero-FD reader rejects handleCount != 0 before reading/adopting/closing any supplied value and poisons the connection; it does not claim those sender-created target handles were reclaimed.

## Maintainable proposed Host-only successor (not implemented or enabled)

Use a versioned sender-handle pull protocol. The receiver already holds the original, supervisor-owned peer process HANDLE with PROCESS_DUP_HANDLE. After authenticating the complete request, it calls DuplicateHandle with that source process and the source handle value, targeting itself with inherit=false. Only this newly returned local handle is adopted/closed. Never use DUPLICATE_CLOSE_SOURCE, resolve a wire PID with OpenProcess, or fall back to receiver-handle interpretation.

The sender must retain an independent original-object duplicate until an acknowledgment covers the complete transfer. Partial-frame, partial-adoption and acknowledgment-loss rules must preserve ownership: receiver closes only copies it created; sender closes its retained source copies only after acknowledgment or confirmed peer termination. An incomplete business reply remains UNKNOWN and is not retried. A bounded transfer count/size and explicit version negotiation must reject mixed peers. This requires a Host-local protocol compatibility decision and real multi-process tests before enablement; it does not require Guest Binder or gfxstream wire changes.

A stable process HANDLE prevents PID replacement, but does not by itself prevent sender handle-number reuse after sending. The retained sender-owned duplicate plus transfer lifetime/ack is therefore mandatory. An authenticated SID alone is insufficient for this transfer contract.

## Implemented cancellation boundary

Each overlapped request owns its event and original pipe HANDLE until GetOverlappedResult reports actual completion. CancelIoEx targets that exact OVERLAPPED once. It is not evidence of completion and has no hard completion-time guarantee. Pending stack/buffer resources are not freed to manufacture bounded cleanup.

Official references:
- https://learn.microsoft.com/en-us/windows/win32/api/handleapi/nf-handleapi-duplicatehandle
- https://learn.microsoft.com/en-us/windows/win32/api/ioapiset/nf-ioapiset-cancelioex
- https://learn.microsoft.com/en-us/windows/win32/fileio/canceling-pending-i-o-operations

No Guest protocol change, security configuration change, VM run, or descriptor protocol enablement is included in these component results.
