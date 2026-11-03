# Contributing

[简体中文](CONTRIBUTING.zh-CN.md) | English

Use the manifest checkout so every file is committed to its owning repository. Keep changes small,
explain security and portability effects, and include the validation commands and results in the
commit message or review description.

Before committing:

1. Confirm `repo status` contains no generated files or unrelated changes.
2. Run formatters and focused tests for every modified component.
3. Run root script syntax checks and the relevant platform prerequisite/regression wrapper.
4. Update both English and Simplified Chinese documents when behavior or operations change.
5. Keep legitimate author attribution while excluding private deployment identities from published material.

The existing AOSP17 GitHub workflow checks source/provenance on three platforms; it does not run VM acceptance. Review any new workflow permissions separately. AOSP17 is default; retain r14 rollback, bilingual docs/ navigation, reachable pinned commits and exact asset checksums. Do not rewrite frozen evidence or treat a documentation commit as a new binary acceptance run.

## Contribution licensing

You must have the right to submit every contribution. Unless a file is governed by a compatible
third-party license, contributions to BSCP-original material are distributed under the repository
license. Submission does not transfer copyright or automatically permit the project owner to offer
your contribution under a separate commercial license. Maintainers may require a separate written
contributor agreement before merging material that must remain commercially licensable. See the
[license policy](LICENSE_POLICY.md).
