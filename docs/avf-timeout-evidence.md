# AVF timeout and shutdown evidence

The macOS wrapper and regression watchdog convert exit 137/143 to allowed timeout
124 only when that invocation's watchdog successfully sent TERM after its timer
completed. A child exit 137/143 or an early external TERM/KILL remains a failure.
The existing allowance for a real timeout is unchanged. Timer/allocation failure
must not create timeout evidence; temporary receipt directories are removed on
normal return. Abrupt termination of the wrapper itself can leave a small temporary
directory, as no new process-wide cleanup trap is installed.

This is conservative evidence, not perfect signal causality: at the deadline,
external and watchdog signals may race. If the watchdog is reaped before recording
its successful signal, retain 137/143 and fail closed. PID selection, TERM/KILL
scope and escalation timing are unchanged; this does not add process-group cleanup.

`python3 -m unittest discover -s tests -p 'test_timeout_provenance.py' -v`
checks early exits/signals, actual deadlines, successful completion, failed timers,
allocation failure, deadline races, and receipt-recording races using shell mocks.
This is not native macOS/HVF or Android guest validation.

## Acceptance evidence must stay separate

- Ready markers establish that the payload reached Ready at least once.
- Only 124 converted here from 137/143 carries this watchdog deadline/TERM receipt.
  A child returning 124 directly is still allowed by existing policy and gains no
  new provenance guarantee. Neither establishes clean guest shutdown.
- A transient vm client returns success after `DeathReason::Shutdown`; the producer
  derives that from crosvm's successful exit unless a specific failure reason wins.
  Capture `VM ended: Shutdown`, its matching run/CID and the actual command status.
- Persistent vm success means Ready and detach, intentionally leaving the VM alive.
- `stop-service`, cleanup, or `Running: 0` do not independently prove process exit.
  The service status uses registration; removal of state makes it report zero.
- Native shutdown acceptance additionally needs correlated crosvm exit/wait evidence
  and independent absence of the tracked VM/service process identities. A PSCI-specific
  claim needs guest/backend shutdown evidence, not merely a Ready message.

The existing macOS regression's PSCI comment/message must not be interpreted as a
shutdown gate. A timeout-plus-Ready run can still pass its boot/Ready checks. This
change does not introduce a shutdown requirement or alter scenario policy.
