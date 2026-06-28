#!/usr/bin/env python3
"""Merge bootconfig key/value lines into an Android initrd bootconfig trailer."""

from __future__ import annotations

import argparse
import struct
import sys
from pathlib import Path

BOOTCONFIG_MAGIC = b"#BOOTCONFIG\n"


def parse_bootconfig_trailer(data: bytes) -> tuple[bytes, bytes] | None:
    if not data.endswith(BOOTCONFIG_MAGIC):
        return None
    body = data[: -len(BOOTCONFIG_MAGIC)]
    if len(body) < 8:
        raise ValueError("bootconfig trailer too short")
    size = struct.unpack_from("<I", body, len(body) - 8)[0]
    checksum = struct.unpack_from("<I", body, len(body) - 4)[0]
    if size > len(body) - 8:
        raise ValueError(f"invalid bootconfig size {size}")
    bootconfig_bytes = body[len(body) - 8 - size : len(body) - 8]
    if (sum(bootconfig_bytes) & 0xFFFFFFFF) != checksum:
        raise ValueError("bootconfig checksum mismatch")
    prefix = body[: len(body) - 8 - size]
    return prefix, bootconfig_bytes


def merge_bootconfig(existing: bytes, updates: dict[str, str]) -> bytes:
    ordered_keys: list[str] = []
    merged: dict[str, str] = {}
    text = existing.decode("utf-8")
    for raw_line in text.splitlines():
        line = raw_line.strip()
        if not line:
            continue
        key = line.split("=", 1)[0]
        if key not in merged:
            ordered_keys.append(key)
        merged[key] = line
    for key, value in updates.items():
        line = f"{key}={value}"
        if key not in merged:
            ordered_keys.append(key)
        merged[key] = line
    if not ordered_keys:
        return b""
    return ("\n".join(merged[key] for key in ordered_keys) + "\n").encode("utf-8")


def write_bootconfig_trailer(prefix: bytes, bootconfig_bytes: bytes) -> bytes:
    checksum = sum(bootconfig_bytes) & 0xFFFFFFFF
    return (
        prefix
        + bootconfig_bytes
        + struct.pack("<I", len(bootconfig_bytes))
        + struct.pack("<I", checksum)
        + BOOTCONFIG_MAGIC
    )


def patch_initrd(initrd_path: Path, updates: dict[str, str]) -> None:
    data = initrd_path.read_bytes()
    parsed = parse_bootconfig_trailer(data)
    if parsed is None:
        prefix = data
        bootconfig_bytes = b""
    else:
        prefix, bootconfig_bytes = parsed
    merged = merge_bootconfig(bootconfig_bytes, updates)
    initrd_path.write_bytes(write_bootconfig_trailer(prefix, merged))


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--initrd", required=True, type=Path)
    parser.add_argument(
        "--set",
        action="append",
        default=[],
        metavar="KEY=VALUE",
        help="Bootconfig entry to merge (repeatable)",
    )
    args = parser.parse_args()

    updates: dict[str, str] = {}
    for item in args.set:
        if "=" not in item:
            print(f"invalid bootconfig entry: {item}", file=sys.stderr)
            return 2
        key, value = item.split("=", 1)
        updates[key] = value

    if not updates:
        print("no bootconfig updates requested", file=sys.stderr)
        return 2

    patch_initrd(args.initrd, updates)
    return 0


if __name__ == "__main__":
    sys.exit(main())
