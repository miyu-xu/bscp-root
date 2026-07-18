#!/usr/bin/env python3
"""Create a small, versioned Cuttlefish host-device tools bundle."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import shutil
import stat
from pathlib import Path

SCRIPT_FILES = (
    "cf_hvc_bridge.py",
    "cf_host_devices.py",
    "modem_simulator_host.py",
    "modem_simulator_windows.py",
    "sensors_simulator_host.py",
    "cf_hvc_tcp_bridge.ps1",
    "root_canal_stub.py",
    "casimir_stub.py",
)
NATIVE_TOOLS = ("root-canal", "casimir")
TARGETS = ("linux-x86_64", "linux-arm64", "windows-x86_64", "darwin-arm64")


def native_target() -> str:
    system = platform.system().lower()
    machine = platform.machine().lower()
    arch = "arm64" if machine in ("arm64", "aarch64") else "x86_64"
    if system == "darwin":
        return f"darwin-{arch}"
    return f"{system}-{arch}"


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def copy_executable(source: Path, destination: Path) -> None:
    shutil.copy2(source, destination)
    destination.chmod(destination.stat().st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)


def find_native(name: str, search_dirs: list[Path], windows: bool) -> Path | None:
    names = (name + ".exe", name) if windows else (name,)
    for directory in search_dirs:
        for candidate_name in names:
            candidate = directory / candidate_name
            if candidate.is_file():
                return candidate
    return None


def package_target(args: argparse.Namespace, target: str) -> Path:
    script_dir = Path(__file__).resolve().parent
    output = Path(args.output_root).resolve() / target
    bin_dir = output / "bin"
    bin_dir.mkdir(parents=True, exist_ok=True)
    files: dict[str, dict[str, object]] = {}

    for name in SCRIPT_FILES:
        source = script_dir / name
        if not source.is_file():
            raise FileNotFoundError(source)
        destination = bin_dir / name
        copy_executable(source, destination)
        files[name] = {
            "sha256": sha256(destination),
            "kind": "powershell" if destination.suffix == ".ps1" else "python",
        }

    search_dirs = [Path(p).resolve() for p in args.native_bin_dir]
    windows = target.startswith("windows-")
    for name in NATIVE_TOOLS:
        source = find_native(name, search_dirs, windows)
        if source is None:
            if args.require_native:
                raise FileNotFoundError(f"{name} not found for {target}")
            files[name] = {"available": False, "kind": "native"}
            continue
        destination = bin_dir / source.name
        copy_executable(source, destination)
        files[destination.name] = {
            "available": True,
            "sha256": sha256(destination),
            "kind": "native",
        }

    manifest = {
        "schema": 1,
        "target": target,
        "python_minimum": "3.9",
        "protocols": {
            "bluetooth": "H4/TCP",
            "nfc": "NCI/TCP",
            "modem": "binder-rpc-vsock-framed-AT",
            "sensors": "cuttlefish-raw-message-v1",
        },
        "fallbacks": {
            "bluetooth": "root_canal_stub.py (LE-only HCI control plane)",
            "nfc": "casimir_stub.py (NCI 2.0 control plane, no RF tags)",
            "modem": "modem_simulator_host.py",
            "sensors": "sensors_simulator_host.py",
        },
        "files": files,
    }
    (output / "manifest.json").write_text(
        json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    return output


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output-root", default="out/dist/host-tools")
    parser.add_argument("--target", action="append", choices=TARGETS)
    parser.add_argument("--all-targets", action="store_true")
    parser.add_argument("--native-bin-dir", action="append", default=[])
    parser.add_argument("--require-native", action="store_true")
    args = parser.parse_args()
    targets = list(TARGETS) if args.all_targets else (args.target or [native_target()])
    for target in targets:
        print(package_target(args, target))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
