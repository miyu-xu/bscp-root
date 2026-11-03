> Historical preparation record, not current release status. AOSP 17 is now published and the default. See [current deployment guide](SUPPORTED-DEPLOYMENT-CONDITIONS-v1830.md). Personal environment fields below are redacted presentation values, not executable configuration.

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

- MoltenVK provenance recovered: the original sibling source repository is clean at `3afb1b3be8f508cee8ed67aad4bd2f3088826b79` (`v1.4.1-55-g3afb1b3b`). Its existing Package/Latest macOS dylib SHA256 `768325c65cdb6b6c9ce814dcce5d962d6e675135e6d48700f8488b54ac2fbdf8` exactly matches the accepted runtime. All seven ExternalRevisions dependencies, including nested SPIRV-Headers, match clean actual HEADs. Eight exact Git source archives and 14 matching notice files have been collected; see MOLTENVK-EXACT-SOURCE-INDEX.json and MOLTENVK-RECOVERED-PROVENANCE.json. This is a retained development commit, not the official v1.4.2 release binary. The old comparison failure remains valid historical evidence. No binary replacement or new VM claim was made; the recovered current tree/build-output match is not a fabricated historical build attestation.
- libslirp-0.dll exactly matches the recorded immutable Google prebuilt object. Its upstream build recipe uses floating libslirp/MSYS2 inputs and statically links glib/iconv. The record does not establish all corresponding source versions/notice obligations. The libslirp-rs MIT notice cannot be applied to the whole DLL. Recover the upstream prebuilt's corresponding source/notice set, or build a pinned dependency closure and repeat affected Windows validation.
- Read-only PE analysis now identifies libslirp's version literal as `4.6.1.15-20e2`, GLib compilation paths as `2.68.3`, and `_libiconv_version` as `0x110` (1.16). These narrow the source search but do not establish packaging patches or complete corresponding source. The Google object's public HEAD metadata provides no source metadata. See LIBSLIRP-BINARY-VERSION-EVIDENCE.json and LIBSLIRP-PROVIDER-HEAD.json.
- Collected Rust/native texts and file-level headers are preserved. RUNTIME-FILE-INVENTORY.cdx.json is deliberately a file inventory, not a claim that every transitive license obligation is resolved.

## Publication gate

User login was confirmed by a supplied screenshot of github.com/miyu-xu/bscp-root/releases/new. The user subsequently navigated elsewhere. Supported live browser validation/upload remains pending; the old screenshot must not be used for clicks. No credential store probing, new token, ACL change, default switch or binary substitution has occurred. Do not promote the default until installation scope and remaining third-party provenance/notice gaps above have been resolved. r14 remains the working rollback default.


## Source-closed Windows replacement, v1830

The original libslirp prebuilt source gap was not waived. A new candidate was built from official libslirp v4.6.1 (cceced0cce9d578ed01db68edf31af78eb66f858), GLib 2.70.5 and the fixed dependency closure in slirp-source-delivery-v1/INDEX.json. Its libslirp SHA256 is 9dc628bfc6ecb1f6d183bfaa276f95ff5e21e4df948d487a09896e1160e97f7f. The required companion libintl-8.dll SHA256 is 67886605af8b9d556949e0b758e1c5ad4b308c42850424fca33e6a36c5bcbbe0. The exact source archive has 2,874 files and 12 collected notice texts; its SHA256 is 01da5677eafb2017b68607f2ccb1ce19071b9c72474e72c956b72de70f7059c9.

The new flat runtime directory is an ordinary inherited-permission child of the already approved private root: assets/host-source-closed-v1828. No ACL was changed. The same EXE/renderer/Guest bytes were copied; only the library pair and runtime paths changed. C/Rust ABI/lifecycle and 55 entry tests passed. Four real GPU-off/on initial/healthy-resume cases passed with original stable identities, real exits/EOF and released storage. The two original fault cases passed resource verification with task UNKNOWN and resources REAPED, retained claims and disks. See WINDOWS-FINAL-RESULT-v1830.json (SHA256 d239e4dfa35305f1a1468452d46d0b490812f80e618b3c523fe3e67234a1003f), 195 rechecked evidence files, no live related process at final observation.

This closes the replacement library's tested dependency/VM gate for the admitted Windows deployment. It does not establish arbitrary-root installation, network forwarding, independent CLI reconnection, or change the old prebuilt's incomplete provenance. Old assets/evidence remain untouched. The new runtime archive and source archive must be distributed together with their notices; the old Windows runtime archive must not be presented as the new candidate.
