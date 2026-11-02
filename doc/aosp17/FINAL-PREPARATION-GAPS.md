# Final preparation gaps (not a release approval)

## Verified

- Frozen Windows/Linux/Mac runtime file inventory matches the saved acceptance closures.
- Three renderer original build outputs and Linux/Mac Binder main libraries match the accepted runtime hashes (NATIVE-OUTPUT-BINDING.json).
- Current renderer/native source trees were archived with per-file hashes and their original CMake source roots. These are current snapshots, not invented build-time attestations.
- Exact x86_64 and arm64 APEX notices were extracted after checking their frozen APEX hashes. Guest notices were already inside the APEX.
- Windows relocated Rust Host builds offline and --help exits 0. Its new binary is NOT substituted for the VM-tested binary.

## Installation decision

The existing accepted binaries can be retained for the three explicitly admitted deployments. This is the narrow supported route already checked by verify_installed.py. It is not an arbitrary-machine installer.

A general installer is not just archive extraction: Windows Initial currently pins runtime-private-v1550 and its reviewed private ACL; Mac profiles and dylib load locations refer to the prior volume and OpenSSL location. Existing binaries/profile checks do not prove a relocated deployment. Do not remove the root gate, mutate ACLs, or patch/sign dylibs merely to claim portability.

For a portable release, implement a user-selected installation-root admission contract preserving existing owner/ACL and handle rules, and validate a genuinely relocated instance. Creation of any new protected runtime root requires an exact directory/owner/access plan before execution. Keeping only the three admitted deployments is an alternative release scope that must be explicitly labelled and must not be described as general installation support.

## Third-party source/notice gaps

- MoltenVK old runtime log identifies 1.4.2. The official v1.4.2 macOS archive digest was verified, but its dylib hash differs from the accepted retained dylib. No binary was replaced. A tag/license text alone is not proof of the retained binary's exact build source. Preserve MOLTENVK-ORIGIN-CHECK.json and obtain its original build/package provenance, or rebuild a pinned official source and repeat affected Mac validation.
- libslirp-0.dll exactly matches the recorded immutable Google prebuilt object. Its upstream build recipe uses floating libslirp/MSYS2 inputs and statically links glib/iconv. The record does not establish all corresponding source versions/notice obligations. The libslirp-rs MIT notice cannot be applied to the whole DLL. Recover the upstream prebuilt's corresponding source/notice set, or build a pinned dependency closure and repeat affected Windows validation.
- Collected Rust/native texts and file-level headers are preserved. RUNTIME-FILE-INVENTORY.cdx.json is deliberately a file inventory, not a claim that every transitive license obligation is resolved.

## Publication gate

GitHub Release upload authentication remains pending. No credential store probing, new token, ACL change, default switch or binary substitution has occurred. Even after login, do not promote the default until the installation scope and remaining third-party provenance/notice gaps above have been resolved. r14 remains the working rollback default.
