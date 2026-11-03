> Historical preparation record, not current release status. AOSP 17 is now published and the default. See [current deployment guide](SUPPORTED-DEPLOYMENT-CONDITIONS-v1830.md). Personal environment fields below are redacted presentation values, not executable configuration.

# AOSP 17 publication readiness

The preparation branch is not the default release. `main/default.xml` still selects r14; `releases/default.json` remains unpublished. Do not interpret source/provenance CI as another VM acceptance run.

## Existing deployment verification

Run on the corresponding admitted machine, from the repository root:

```
python -B releases/aosp17/verify_installed.py --platform windows
python3 -B releases/aosp17/verify_installed.py --platform linux
python3 -B releases/aosp17/verify_installed.py --platform macos
```

Choose only the matching platform command. These commands verify the exact saved profile, closure, Host identity and GPU on/off asset contracts. They do not install, start a VM, change ACLs or admit a new state root. All three existing deployments passed this check during publication preparation.

Frozen runtime archives preserve the accepted binaries. They are not yet general-purpose installers: platform dependencies, admitted root paths and deployment configuration must remain explicit. In particular, the Windows original-handle controller has a fixed approved runtime root. Copying files elsewhere does not authorize that location or prove a new installation.

## Source and build provenance

The preparation branch includes 5,193 verified source files across Windows Host/crosvm and Linux/Mac Host closures. Windows sealed snapshots are checked against their original accepted source seals. Linux/Mac current Cargo closure snapshots are labelled current snapshots; they are not retroactively claimed as complete build-time attestations.

The portable Windows Host build replaces absolute Cargo dependency paths with relative paths and includes native sibling build inputs. It uses the existing locked dependencies offline and process-local tool configuration. Its new binary is a reproducibility candidate, not a replacement for the accepted VM-tested binary. Original failures and each build receipt remain preserved.

## Promotion gates still open

- Finish portable build recipes, native/renderer source correspondence and dependency notices.
- Finish an installation/profile admission route appropriate to the actual supported deployment; do not broaden security policy implicitly.
- Upload real Host/Guest assets through an authenticated supported GitHub Release session and verify their downloadable hashes.
- Publish the final code revision before pinning it in the default manifest; verify remote refs and exact-commit CI.

The prepared manifest currently pins an earlier preparation commit and must be updated after the final code revision is remotely available. No force push or GitHub default-branch setting change is required.

## Preserved capability limits

Windows independent persistent CLI reconnect remains disabled. Mac historical ext4-specific 70-pass/1-fail evidence is retained and not relabelled. Fault business outcomes remain UNKNOWN even when resources are REAPED. UI/input, A and H3 remain outside this release's verification scope.

r14 code `43e4786a48c142b4c8b6f4bfb0db38cd32b20e42` and manifest `89c0365a4dff5c0e52a5386877592a2f6fd45548` remain available as rollback references. Publication must preserve their admitted runtime assets and user changes.
