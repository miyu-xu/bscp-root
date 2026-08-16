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
5. Check that author and committer identities are approved and that release commit timestamps
   follow the repository's weekend publication policy.

Do not add hosted CI workflow files to this repository. Downstream automation may invoke the same
documented commands without changing the source tree.
