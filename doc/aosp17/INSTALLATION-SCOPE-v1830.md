# Installation scope of the current release candidate

The original repository deployment instructions describe a workspace checkout
and platform build. They do not establish that a downloaded frozen runtime is
an arbitrary-machine installer. This distinction must remain visible in the
Release description and default manifest documentation.

## What is actually verified

- Windows: the new 15-file flat Host directory is
  `E:/bscp/out/aosp17-windows-formal-v1461/runtime-private-v1550/assets/host-source-closed-v1828`.
  It inherits the already approved private-root permissions; no ACL was changed.
  The new library pair, unchanged EXEs/renderer and matching Guest passed four
  normal and two fault cases. A new versioned state root was used; old instances
  were not reused or overwritten.
- Linux/Mac: previously recorded admitted deployments and frozen artifacts
  remain unchanged. Their existing path checks are not a relocation test.

## What is not provided

The Windows controller still admits a precise protected root. Mac profiles and
dylib linkage still include existing volume/OpenSSL paths. The runtime archives
are reference deployment artifacts, not arbitrary-root installers. A portable
Rust Host build/`--help` result does not change this limitation.

A release using these archives must explicitly say that they support the
recorded deployment conditions, retain the exact source/notice bundles, and
must not advertise general extraction-and-run installation. The default-source
manifest and reference-binary support scope are separate claims.

If arbitrary-root installation is required before publication, the remaining
work is an installation-root admission implementation and affected native
loader relocation with actual VM revalidation. It must preserve owner/ACL,
original process identity, lease, EOF and failure rules. No new protected
directory, modified security descriptor, UAC setting, credential, or ad-hoc
library substitution is authorized by this document. The current work has not
performed those actions.

The old runtime packages and r14 remain available for rollback. No main/default
manifest entry or system running profile has been changed by v1830 preparation.
