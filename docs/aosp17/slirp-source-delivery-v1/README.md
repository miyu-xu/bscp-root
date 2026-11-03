# Windows libslirp corresponding-source candidate

This is a new candidate, not the old Google prebuilt and not yet a promoted runtime.
The old accepted binary and its evidence remain unchanged.

`INDEX.json` records every archived source file, the source lock, exact library
hashes, recursive PE imports and the limited component test results.
`libslirp-superbuild-source.tar.gz` contains the build source including the
upstream Meson dependency sources and package caches. Its 2,874 files were read
back from the archive and individually checked. `notices/` preserves 12 source
license/author texts without editing their content.

The runtime pair is `candidate/libslirp-0.dll` and `candidate/libintl-8.dll`.
The former includes static GLib 2.70.5 and uses the upstream proxy-libintl
translation fallback. Do not substitute the old prebuilt's iconv/GLib source
version claims for this new build. The upstream GLib 2.68.3 build failure is
retained; no warning suppression or local runtime patch was used to bypass it.

## Rebuilding

Use the fixed compiler/tool identities recorded in `SLIRP-BUILD-TOOLS.json` and
the configure/build receipts. The current compiler is the existing MinGW GCC
15.2.0 toolchain; it is not installed by this package. The Python scripts are
the exact build-instance records and contain that instance's paths. For another
build root, unpack `source/` and use the corresponding local paths:

```text
python -m mesonbuild.mesonmain setup BUILD SOURCE --buildtype=release --wrap-mode=nodownload --prefix INSTALL
ninja -C BUILD -j2 subprojects/libslirp/libslirp-0.dll
```

Set `CC`, `CXX`, `NINJA` and `PATH` to the recorded compiler, Ninja and Git shell;
set `PYTHONPATH` to the task-local Meson installation. No global installation is
required. Keep both generated DLLs together. Source changes and relinking are
possible with this source/build closure; byte-identical output across different
toolchains, paths or timestamps is not asserted.

The ABI/lifecycle tests cover 26 shared C/Rust fields, invalid-version rejection
and 100 disabled-network creation/cleanup loops. They do not establish network
forwarding or VM capability. The separate v1828 acceptance receipts, when
complete, are the authority for the new runtime's VM results. Old v1777 results
must never be relabelled as results for this library.
