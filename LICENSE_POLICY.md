# BSCP License Policy

[简体中文](LICENSE_POLICY.zh-CN.md) | English

This policy explains how licensing is applied across the BSCP multi-repository workspace. It does
not replace the controlling license text.

## BSCP-original material

Unless a file or directory states otherwise, original orchestration code, scripts, tests,
configuration, and documentation tracked by this root repository are available under
`PolyForm-Noncommercial-1.0.0`. The controlling terms are linked from [LICENSE](LICENSE).

The license permits use, modification, and distribution for permitted noncommercial purposes.
Any purpose outside those permissions requires a separate written commercial license from the
applicable BSCP copyright holder before use begins.

Because the license restricts commercial fields of use, BSCP-original material under these terms
is source-available and must not be represented as OSI-approved open-source software.

## Workspace license map

| Material | Controlling terms |
| --- | --- |
| Files tracked directly by `bscp-root` | PolyForm Noncommercial 1.0.0 unless another notice applies |
| Manifest repository's original files | Its own top-level BSCP license and policy |
| `packages/modules/Virtualization`, `frameworks/native`, `system/core`, and other AOSP projects | Their AOSP, Apache-2.0, NOTICE, GPL, or file-level terms |
| `external/crosvm` | Its BSD license and bundled third-party licenses |
| `hardware/google/gfxstream` and `hardware/google/aemu` | Their Apache-2.0 and bundled third-party licenses |
| Optional `hd` repository | Its repository license and third-party notices; a root license does not relicense it |
| OVMF firmware under `prebuilts/firmware` | Upstream EDK II/OVMF terms, including `BSD-2-Clause-Patent` and applicable component notices |
| Generated AOSP images, APEX files, toolchains, and packaged dependencies | The licenses and notices of their source projects; they are not relicensed by the packager |

Directories managed as independent Git repositories are separate works with separate license
files. Including them in one manifest, build, archive, VM image, or distribution does not erase
their original notices or convert them to the root license.

## Commercial licensing

Commercial permission applies only to rights the identified BSCP copyright holder can license. A
commercial distribution must also satisfy every third-party license, attribution, source-offer,
patent, trademark, export, and binary-notice obligation that applies to its contents.

See [Commercial Licensing](COMMERCIAL_LICENSING.md) for the request process. Permission is valid
only when the copyright holder provides explicit written authorization identifying the licensed
version, scope, and licensee. Repository access, issue discussion, technical assistance, or silence
does not grant commercial rights.

## Contributions and dual licensing

Copyright does not transfer merely because a contribution is submitted or merged. A contributor
must have the right to submit the material and agrees that it is distributed under the applicable
repository license. If the project owner needs authority to offer that contribution under a
separate commercial license, the contributor must sign an additional contributor or copyright
agreement that expressly grants that authority.

To preserve a coherent commercial-licensing path, maintainers should not merge third-party
contributions into BSCP-original material until the required rights are documented. This does not
apply to upstream code already incorporated under its existing Apache, BSD, GPL, MIT, or other
license.

## Previous releases

Adding or changing a license does not revoke permissions already granted for earlier material, and
it cannot retroactively restrict third-party code already available under an irrevocable license.
Every release must retain the exact license and notice set that applies to that release.

## Release checklist

- Classify every tracked file as BSCP-original, third-party, generated, or unknown.
- Preserve file-level copyright and SPDX headers.
- Include the root license, this policy, commercial-licensing notice, and all third-party notices.
- Generate an SBOM and license inventory for source and binary distributions.
- Verify that prebuilt firmware, AOSP images, APEX content, and host tools carry redistributable
  notices and corresponding source or source offers when required.
- Obtain written commercial permission before any commercial use of BSCP-original material.
- Have qualified counsel review the release when legal ownership or license compatibility is
  uncertain.
