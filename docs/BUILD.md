# Source synchronization and builds

AOSP 17 is the default; the manifest pins individual projects, not a common branch name for every repository. The full snapshot has 1087 official projects plus the BSCP root. A smaller Guest build closure is not proof that the full tree has synchronized.

Start from the fresh-workspace commands in [README](../README.md). Record the manifest repository commit and `repo manifest -r -o resolved-manifest.xml`. On an existing tree, inspect local branches, tracked and untracked changes first; do not use destructive reset/clean to force synchronization.

The published runtime was built from the source closures and tool/configuration records indexed under [AOSP17 evidence](aosp17/). The generic `build_all.sh`/`build_all.bat` wrappers are development entry points; running them alone does not reproduce or validate the frozen reference package. Follow the platform-specific source closure, exact dependency locks, native library recipes and Guest inputs. Do not mix r14 libraries with AOSP17.

Configuration and build scripts can download dependencies and create large output trees. Review their arguments before running. Keep outputs isolated, budget disk/RAM, retain r14 assets and build architectures serially when memory is limited. Do not automatically install drivers, change ACLs, enable hypervisors or edit network settings as part of a documentation command.

Source/provenance checks from this checkout, without starting VMs:

```sh
python3 -B releases/aosp17/verify_snapshot.py
python3 -B releases/aosp17/verify_host_sources.py
python3 -B releases/aosp17/test_public_verifiers.py
```

On Windows use `python` instead of `python3`. The last command creates retained test fixtures. These checks are not native compilation or VM acceptance. The GitHub AOSP17 workflow runs source/provenance checks on three operating systems; older statements that this repository has no GitHub Actions are obsolete.

[简体中文](BUILD.zh-CN.md)
