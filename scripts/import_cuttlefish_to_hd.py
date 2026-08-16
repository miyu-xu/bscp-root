#!/usr/bin/env python3
"""Convert an x86_64 Cuttlefish release into HD Guest staging.

The importer is deliberately offline. It consumes an image ZIP, build metadata
from either a matching target-files ZIP or OTA metadata, and an already-built
x86_64 HD sensor injector. It does not download, sign, publish, certify, or
launch a Guest.
"""

from __future__ import annotations

import argparse
import contextlib
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import stat
import struct
import sys
import tempfile
import time
import uuid
import zipfile
import zlib


SCHEMA_VERSION = 1
ANDROID_SPARSE_MAGIC = 0xED26FF3A
SPARSE_CHUNK_RAW = 0xCAC1
SPARSE_CHUNK_FILL = 0xCAC2
SPARSE_CHUNK_DONT_CARE = 0xCAC3
SPARSE_CHUNK_CRC32 = 0xCAC4
BOOT_MAGIC = b"ANDROID!"
VENDOR_BOOT_MAGIC = b"VNDRBOOT"
PAGE_SIZE_V4 = 4096
SECTOR_SIZE = 512
GPT_ENTRY_SIZE = 128
GPT_ENTRY_COUNT = 128
GPT_ALIGNMENT_LBA = 2048
ANDROID_BASIC_DATA_GUID = uuid.UUID("EBD0A0A2-B9E5-4433-87C0-68B6B72699C7")
IMPORT_NAMESPACE = uuid.UUID("62f59139-a36a-5ca4-8f5a-27909f5e8219")
MAX_TEXT_FILE = 4 * 1024 * 1024
MAX_IMAGE_FILE = 32 * 1024 * 1024 * 1024
COPY_CHUNK = 8 * 1024 * 1024
METADATA_PARTITION_SIZE = 64 * 1024 * 1024
LZ4_LEGACY_MAGIC = b"\x02\x21\x4c\x18"
MAX_RAMDISK_SIZE = 512 * 1024 * 1024
SUPPORTED_RELEASES = {("15", "35"), ("17", "37")}

REQUIRED_IMAGES = (
    "boot.img",
    "vendor_boot.img",
    "vbmeta.img",
    "vbmeta_system.img",
    "super.img",
    "userdata.img",
)
OPTIONAL_IMAGES = (
    "init_boot.img",
    "vbmeta_vendor_dlkm.img",
    "vbmeta_system_dlkm.img",
    "misc.img",
    "factory_reset_protected.img",
    "metadata.img",
)
FSTAB_NAMES = (
    "fstab.cf.f2fs.hctr2",
    "fstab.cf.ext4.hctr2",
    "fstab.cf.f2fs.cts",
    "fstab.cf.ext4.cts",
    "fstab.cutf_cvm",
)


class ImportFailure(RuntimeError):
    """Stable importer failure surfaced to automation."""

    def __init__(self, code: str, message: str):
        super().__init__(message)
        self.code = code


class EventLog:
    def __init__(self) -> None:
        self._events: list[dict[str, object]] = []

    def emit(self, name: str, status: str, **fields: object) -> None:
        event: dict[str, object] = {
            "schema_version": SCHEMA_VERSION,
            "sequence": len(self._events) + 1,
            "event": name,
            "status": status,
        }
        event.update(fields)
        self._events.append(event)

    def write(self, path: Path) -> None:
        with path.open("w", encoding="utf-8", newline="\n") as stream:
            for event in self._events:
                stream.write(json.dumps(event, sort_keys=True, separators=(",", ":")))
                stream.write("\n")


def fail(code: str, message: str) -> None:
    raise ImportFailure(code, message)


def align_up(value: int, alignment: int) -> int:
    return ((value + alignment - 1) // alignment) * alignment


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        while chunk := stream.read(COPY_CHUNK):
            digest.update(chunk)
    return digest.hexdigest()


def copy_stream(source, destination) -> int:
    total = 0
    while chunk := source.read(COPY_CHUNK):
        destination.write(chunk)
        total += len(chunk)
    return total


def normalized_zip_name(info: zipfile.ZipInfo) -> str:
    raw = info.filename.replace("\\", "/")
    path = PurePosixPath(raw)
    if path.is_absolute() or any(part in ("", ".", "..") for part in path.parts):
        fail("CFI_ARCHIVE_UNSAFE", f"unsafe ZIP member path: {info.filename!r}")
    unix_mode = info.external_attr >> 16
    if stat.S_ISLNK(unix_mode):
        fail("CFI_ARCHIVE_UNSAFE", f"symbolic-link ZIP member is not allowed: {raw}")
    return str(path)


def archive_members_by_basename(archive: zipfile.ZipFile) -> dict[str, zipfile.ZipInfo]:
    selected: dict[str, zipfile.ZipInfo] = {}
    for info in archive.infolist():
        if info.is_dir():
            continue
        normalized = normalized_zip_name(info)
        name = PurePosixPath(normalized).name
        if name not in REQUIRED_IMAGES and name not in OPTIONAL_IMAGES:
            continue
        if info.file_size <= 0 or info.file_size > MAX_IMAGE_FILE:
            fail("CFI_ARCHIVE_LIMIT", f"invalid declared size for {normalized}: {info.file_size}")
        if name in selected:
            fail("CFI_ARCHIVE_AMBIGUOUS", f"duplicate critical image basename in ZIP: {name}")
        selected[name] = info
    return selected


def extract_release_images(image_zip: Path, destination: Path) -> dict[str, Path]:
    try:
        archive = zipfile.ZipFile(image_zip)
    except (OSError, zipfile.BadZipFile) as error:
        fail("CFI_INPUT_INVALID", f"cannot open image ZIP: {error}")
    with archive:
        members = archive_members_by_basename(archive)
        missing = sorted(set(REQUIRED_IMAGES) - members.keys())
        if missing:
            fail("CFI_MISSING_IMAGE", f"image ZIP is missing: {', '.join(missing)}")
        destination.mkdir(parents=True)
        extracted: dict[str, Path] = {}
        for name, info in sorted(members.items()):
            output = destination / name
            with archive.open(info) as source, output.open("wb") as target:
                actual = copy_stream(source, target)
            if actual != info.file_size:
                fail("CFI_ARCHIVE_TRUNCATED", f"short extraction for {name}")
            extracted[name] = output
        return extracted


def read_zip_text(archive: zipfile.ZipFile, info: zipfile.ZipInfo) -> str:
    if info.file_size <= 0 or info.file_size > MAX_TEXT_FILE:
        fail("CFI_ARCHIVE_LIMIT", f"invalid text member size: {info.filename}")
    with archive.open(info) as stream:
        data = stream.read(MAX_TEXT_FILE + 1)
    if len(data) > MAX_TEXT_FILE:
        fail("CFI_ARCHIVE_LIMIT", f"text member exceeds limit: {info.filename}")
    try:
        return data.decode("utf-8")
    except UnicodeDecodeError as error:
        fail("CFI_TEXT_INVALID", f"non-UTF-8 target-files member {info.filename}: {error}")


def parse_properties(text: str) -> dict[str, str]:
    properties: dict[str, str] = {}
    for raw in text.splitlines():
        line = raw.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, value = line.split("=", 1)
        properties[key.strip()] = value.strip()
    return properties


def read_metadata_file(path: Path) -> str:
    try:
        data = path.read_bytes()
    except OSError as error:
        fail("CFI_INPUT_INVALID", f"cannot read OTA metadata: {error}")
    if not data or len(data) > MAX_TEXT_FILE:
        fail("CFI_ARCHIVE_LIMIT", f"invalid OTA metadata size: {len(data)}")
    try:
        return data.decode("utf-8")
    except UnicodeDecodeError as error:
        fail("CFI_TEXT_INVALID", f"non-UTF-8 OTA metadata: {error}")


def inspect_ota_metadata(path: Path, image_zip: Path) -> tuple[dict[str, str], str]:
    metadata = parse_properties(read_metadata_file(path))
    fingerprint = metadata.get("post-build", "")
    sdk = metadata.get("post-sdk-level", "")
    incremental = metadata.get("post-build-incremental", "")
    device = metadata.get("pre-device", "")
    match = re.match(r"^([^/]+)/([^/]+)/([^:]+):([^/]+)/([^/]+)/([^:]+):([^/]+)/(.+)$", fingerprint)
    if match is None:
        fail("CFI_BUILD_MISMATCH", f"invalid OTA post-build fingerprint: {fingerprint!r}")
    _, product, fingerprint_device, release, build_id, fingerprint_incremental, build_type, tags = (
        match.groups()
    )
    if (release, sdk) not in SUPPORTED_RELEASES:
        fail(
            "CFI_BUILD_MISMATCH",
            f"unsupported Cuttlefish release/SDK: release={release!r} sdk={sdk!r}",
        )
    if "x86_64" not in device or "x86_64" not in fingerprint_device:
        fail("CFI_BUILD_MISMATCH", f"expected x86_64 OTA metadata, got device={device!r}")
    if build_type != "userdebug":
        fail("CFI_BUILD_MISMATCH", f"only a userdebug Cuttlefish build is accepted, got {build_type!r}")
    if not incremental or incremental != fingerprint_incremental:
        fail("CFI_BUILD_MISMATCH", "OTA incremental build identifiers do not match")
    if incremental not in image_zip.name:
        fail(
            "CFI_BUILD_MISMATCH",
            f"image ZIP name does not contain OTA build {incremental}: {image_zip.name}",
        )
    properties = {
        "ro.build.version.release": release,
        "ro.build.version.sdk": sdk,
        "ro.product.cpu.abi": "x86_64",
        "ro.build.type": build_type,
        "ro.debuggable": "1",
        "ro.product.name": product,
        "ro.build.product": fingerprint_device,
        "ro.build.fingerprint": fingerprint,
        "ro.build.id": build_id,
        "ro.build.version.incremental": incremental,
        "ro.build.tags": tags,
        "ro.build.version.security_patch": metadata.get("post-security-patch-level", ""),
    }
    return properties, product


def inspect_target_files(target_zip: Path) -> tuple[str, str, dict[str, str], str]:
    try:
        archive = zipfile.ZipFile(target_zip)
    except (OSError, zipfile.BadZipFile) as error:
        fail("CFI_INPUT_INVALID", f"cannot open target-files ZIP: {error}")
    with archive:
        fstabs: dict[str, zipfile.ZipInfo] = {}
        property_infos: list[zipfile.ZipInfo] = []
        for info in archive.infolist():
            if info.is_dir():
                continue
            normalized = normalized_zip_name(info)
            path = PurePosixPath(normalized)
            if path.name in FSTAB_NAMES and "VENDOR" in path.parts:
                if path.name in fstabs:
                    fail("CFI_ARCHIVE_AMBIGUOUS", f"duplicate target fstab: {path.name}")
                fstabs[path.name] = info
            if path.name == "build.prop" and any(
                part in ("SYSTEM", "VENDOR", "PRODUCT") for part in path.parts
            ):
                property_infos.append(info)
        selected_fstab = next((name for name in FSTAB_NAMES if name in fstabs), None)
        if selected_fstab is None:
            fail("CFI_MISSING_FSTAB", "target-files ZIP has no supported Cuttlefish vendor fstab")
        properties: dict[str, str] = {}
        for info in property_infos:
            properties.update(parse_properties(read_zip_text(archive, info)))
        release = properties.get("ro.build.version.release", "")
        sdk = properties.get("ro.build.version.sdk", "")
        abi_values = " ".join(
            properties.get(key, "")
            for key in ("ro.product.cpu.abi", "ro.product.cpu.abilist", "ro.bionic.arch")
        )
        build_type = properties.get("ro.build.type", "")
        debuggable = properties.get("ro.debuggable", "")
        product = properties.get("ro.product.name", properties.get("ro.build.product", "unknown"))
        if release != "15" or sdk != "35":
            fail("CFI_BUILD_MISMATCH", f"expected Android 15/SDK 35, got release={release!r} sdk={sdk!r}")
        if "x86_64" not in abi_values:
            fail("CFI_BUILD_MISMATCH", f"expected x86_64 target-files, got ABI properties {abi_values!r}")
        if build_type != "userdebug" and debuggable != "1":
            fail("CFI_BUILD_MISMATCH", "only a userdebug/debuggable Cuttlefish build is accepted")
        fstab_text = read_zip_text(archive, fstabs[selected_fstab])
        return selected_fstab, fstab_text, properties, product


def filter_fstab(source: str) -> str:
    output: list[str] = []
    for raw in source.splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        columns = line.split()
        if len(columns) != 5:
            fail("CFI_FSTAB_INVALID", f"unsupported fstab line: {raw}")
        mount_point = columns[1]
        if not mount_point.startswith("/") or "/" in mount_point.lstrip("/"):
            continue
        flags = set(columns[4].split(","))
        if "recoveryonly" in flags and "first_stage_mount" not in flags:
            continue
        output.append("\t".join(columns))
    if not output:
        fail("CFI_FSTAB_INVALID", "filtered fstab is empty")
    return "\n".join(output) + "\n"


def parse_boot_image(path: Path) -> tuple[bytes, bytes]:
    with path.open("rb") as stream:
        header = stream.read(PAGE_SIZE_V4)
        if len(header) < 1584 or header[:8] != BOOT_MAGIC:
            fail("CFI_BOOT_FORMAT", f"invalid Android boot image: {path.name}")
        kernel_size, ramdisk_size = struct.unpack_from("<II", header, 8)
        header_size = struct.unpack_from("<I", header, 20)[0]
        header_version = struct.unpack_from("<I", header, 40)[0]
        if header_version not in (3, 4) or not 1580 <= header_size <= PAGE_SIZE_V4:
            fail("CFI_BOOT_FORMAT", f"unsupported boot header v{header_version} size={header_size}")
        kernel_offset = align_up(header_size, PAGE_SIZE_V4)
        ramdisk_offset = align_up(kernel_offset + kernel_size, PAGE_SIZE_V4)
        stream.seek(kernel_offset)
        kernel = stream.read(kernel_size)
        stream.seek(ramdisk_offset)
        ramdisk = stream.read(ramdisk_size)
    if len(kernel) != kernel_size or len(ramdisk) != ramdisk_size:
        fail("CFI_BOOT_FORMAT", f"truncated Android boot image: {path.name}")
    return kernel, ramdisk


def parse_init_boot_image(path: Path) -> bytes:
    kernel, ramdisk = parse_boot_image(path)
    if kernel:
        fail("CFI_BOOT_FORMAT", "init_boot.img unexpectedly contains a kernel")
    return ramdisk


def parse_vendor_boot_image(path: Path) -> tuple[bytes, bytes]:
    with path.open("rb") as stream:
        header = stream.read(PAGE_SIZE_V4)
        if len(header) < 2128 or header[:8] != VENDOR_BOOT_MAGIC:
            fail("CFI_BOOT_FORMAT", "invalid vendor_boot.img")
        header_version = struct.unpack_from("<I", header, 8)[0]
        page_size = struct.unpack_from("<I", header, 12)[0]
        vendor_ramdisk_size = struct.unpack_from("<I", header, 24)[0]
        header_size = struct.unpack_from("<I", header, 2096)[0]
        dtb_size = struct.unpack_from("<I", header, 2100)[0]
        table_size = struct.unpack_from("<I", header, 2112)[0]
        table_entries = struct.unpack_from("<I", header, 2116)[0]
        table_entry_size = struct.unpack_from("<I", header, 2120)[0]
        bootconfig_size = struct.unpack_from("<I", header, 2124)[0]
        if (
            header_version != 4
            or page_size < 2048
            or page_size > 65536
            or page_size & (page_size - 1)
        ):
            fail("CFI_BOOT_FORMAT", f"expected vendor boot v4, got v{header_version} page={page_size}")
        if header_size < 2128 or header_size > 65536:
            fail("CFI_BOOT_FORMAT", f"invalid vendor boot v4 header size: {header_size}")
        expected_table_size = table_entries * table_entry_size
        if table_size != expected_table_size:
            fail(
                "CFI_BOOT_FORMAT",
                "vendor ramdisk table size does not match its entry count and size",
            )
        ramdisk_offset = align_up(header_size, page_size)
        dtb_offset = align_up(ramdisk_offset + vendor_ramdisk_size, page_size)
        table_offset = align_up(dtb_offset + dtb_size, page_size)
        bootconfig_offset = align_up(table_offset + table_size, page_size)
        stream.seek(ramdisk_offset)
        vendor_ramdisk = stream.read(vendor_ramdisk_size)
        stream.seek(bootconfig_offset)
        bootconfig = stream.read(bootconfig_size)
    if len(vendor_ramdisk) != vendor_ramdisk_size or len(bootconfig) != bootconfig_size:
        fail("CFI_BOOT_FORMAT", "truncated vendor_boot.img")
    if not vendor_ramdisk:
        fail("CFI_BOOT_FORMAT", "vendor_boot.img has an empty vendor ramdisk")
    return vendor_ramdisk, bootconfig


def decompress_lz4_block(source: bytes, limit: int = COPY_CHUNK) -> bytes:
    output = bytearray()
    cursor = 0
    while cursor < len(source):
        token = source[cursor]
        cursor += 1
        literal_size = token >> 4
        if literal_size == 15:
            while True:
                if cursor >= len(source):
                    fail("CFI_RAMDISK_FORMAT", "truncated LZ4 literal length")
                extension = source[cursor]
                cursor += 1
                literal_size += extension
                if extension != 255:
                    break
        if cursor + literal_size > len(source):
            fail("CFI_RAMDISK_FORMAT", "truncated LZ4 literals")
        output.extend(source[cursor : cursor + literal_size])
        cursor += literal_size
        if len(output) > limit:
            fail("CFI_ARCHIVE_LIMIT", "LZ4 block exceeds decompression limit")
        if cursor == len(source):
            break
        if cursor + 2 > len(source):
            fail("CFI_RAMDISK_FORMAT", "truncated LZ4 match offset")
        offset = struct.unpack_from("<H", source, cursor)[0]
        cursor += 2
        if offset == 0 or offset > len(output):
            fail("CFI_RAMDISK_FORMAT", "invalid LZ4 match offset")
        match_size = token & 0x0F
        if match_size == 15:
            while True:
                if cursor >= len(source):
                    fail("CFI_RAMDISK_FORMAT", "truncated LZ4 match length")
                extension = source[cursor]
                cursor += 1
                match_size += extension
                if extension != 255:
                    break
        match_size += 4
        if len(output) + match_size > limit:
            fail("CFI_ARCHIVE_LIMIT", "LZ4 block exceeds decompression limit")
        for _ in range(match_size):
            output.append(output[-offset])
    return bytes(output)


def decompress_legacy_lz4(source: bytes) -> bytes:
    if not source.startswith(LZ4_LEGACY_MAGIC):
        return source
    output = bytearray()
    cursor = len(LZ4_LEGACY_MAGIC)
    while cursor < len(source):
        if cursor + 4 > len(source):
            if any(source[cursor:]):
                fail("CFI_RAMDISK_FORMAT", "truncated legacy LZ4 block header")
            break
        encoded_size = struct.unpack_from("<I", source, cursor)[0]
        cursor += 4
        if encoded_size == 0:
            break
        uncompressed = bool(encoded_size & 0x80000000)
        block_size = encoded_size & 0x7FFFFFFF
        if block_size == 0 or block_size > COPY_CHUNK or cursor + block_size > len(source):
            fail("CFI_RAMDISK_FORMAT", "invalid legacy LZ4 block size")
        block = source[cursor : cursor + block_size]
        cursor += block_size
        output.extend(block if uncompressed else decompress_lz4_block(block))
        if len(output) > MAX_RAMDISK_SIZE:
            fail("CFI_ARCHIVE_LIMIT", "vendor ramdisk exceeds decompression limit")
    return bytes(output)


def fstabs_from_newc(source: bytes) -> dict[str, tuple[str, str]]:
    fstabs: dict[str, tuple[str, str]] = {}
    cursor = source.find(b"070701")
    while cursor >= 0 and cursor + 110 <= len(source):
        if source[cursor : cursor + 6] != b"070701":
            break
        try:
            fields = [
                int(source[cursor + 6 + index * 8 : cursor + 14 + index * 8], 16)
                for index in range(13)
            ]
        except ValueError:
            fail("CFI_RAMDISK_FORMAT", "invalid newc header")
        file_size = fields[6]
        name_size = fields[11]
        name_start = cursor + 110
        name_end = name_start + name_size
        if name_size < 1 or name_end > len(source):
            fail("CFI_RAMDISK_FORMAT", "invalid newc member name")
        raw_name = source[name_start:name_end]
        if raw_name[-1:] != b"\0":
            fail("CFI_RAMDISK_FORMAT", "unterminated newc member name")
        try:
            name = raw_name[:-1].decode("utf-8")
        except UnicodeDecodeError as error:
            fail("CFI_TEXT_INVALID", f"non-UTF-8 newc member name: {error}")
        data_start = align_up(name_end, 4)
        data_end = data_start + file_size
        if data_end > len(source):
            fail("CFI_RAMDISK_FORMAT", f"truncated newc member: {name}")
        if name == "TRAILER!!!":
            cursor = source.find(b"070701", align_up(data_end, 4))
            continue
        path = PurePosixPath(name)
        if path.is_absolute() or any(part in ("", ".", "..") for part in path.parts):
            fail("CFI_ARCHIVE_UNSAFE", f"unsafe vendor ramdisk member path: {name!r}")
        if path.name in FSTAB_NAMES:
            try:
                text = source[data_start:data_end].decode("utf-8")
            except UnicodeDecodeError as error:
                fail("CFI_TEXT_INVALID", f"non-UTF-8 vendor ramdisk fstab: {error}")
            existing = fstabs.get(path.name)
            if existing is not None and existing[1] != text:
                fail("CFI_ARCHIVE_AMBIGUOUS", f"conflicting vendor ramdisk fstab: {path.name}")
            if existing is None:
                fstabs[path.name] = (name, text)
        cursor = align_up(data_end, 4)
    return fstabs


def inspect_vendor_ramdisk_fstab(vendor_ramdisk: bytes) -> tuple[str, str]:
    fstabs = fstabs_from_newc(decompress_legacy_lz4(vendor_ramdisk))
    selected = next((name for name in FSTAB_NAMES if name in fstabs), None)
    if selected is None:
        fail("CFI_MISSING_FSTAB", "vendor_boot ramdisk has no supported Cuttlefish fstab")
    path, text = fstabs[selected]
    return selected, text


def validate_x86_kernel(kernel: bytes) -> None:
    if len(kernel) < 0x206 or kernel[0x202:0x206] != b"HdrS":
        fail("CFI_BUILD_MISMATCH", "boot.img kernel is not an x86 Linux boot image")


def validate_sensor_injector(path: Path) -> None:
    try:
        with path.open("rb") as stream:
            header = stream.read(64)
    except OSError as error:
        fail("CFI_INPUT_INVALID", f"cannot read sensor injector: {error}")
    if len(header) < 20 or header[:5] != b"\x7fELF\x02" or header[5] != 1:
        fail("CFI_BUILD_MISMATCH", "sensor injector must be a little-endian ELF64 executable")
    machine = struct.unpack_from("<H", header, 18)[0]
    if machine != 62:
        fail("CFI_BUILD_MISMATCH", f"sensor injector must target x86_64 (ELF machine 62), got {machine}")


def cpio_newc_entry(name: str, data: bytes, mode: int, inode: int) -> bytes:
    encoded_name = name.encode("utf-8") + b"\0"
    fields = (
        inode,
        mode,
        0,
        0,
        2 if stat.S_ISDIR(mode) else 1,
        0,
        len(data),
        0,
        0,
        0,
        0,
        len(encoded_name),
        0,
    )
    header = b"070701" + b"".join(f"{value:08x}".encode("ascii") for value in fields)
    result = bytearray(header)
    result.extend(encoded_name)
    result.extend(b"\0" * ((-len(result)) % 4))
    result.extend(data)
    result.extend(b"\0" * ((-len(result)) % 4))
    return bytes(result)


def make_fstab_cpio(fstab_name: str, fstab_data: bytes) -> bytes:
    paths: list[tuple[str, bytes, int]] = [
        (".", b"", stat.S_IFDIR | 0o755),
        ("first_stage_ramdisk", b"", stat.S_IFDIR | 0o755),
        ("first_stage_ramdisk/system", b"", stat.S_IFDIR | 0o755),
        ("first_stage_ramdisk/system/etc", b"", stat.S_IFDIR | 0o755),
        ("system", b"", stat.S_IFDIR | 0o755),
        ("system/etc", b"", stat.S_IFDIR | 0o755),
    ]
    for name in dict.fromkeys((fstab_name, "fstab.cutf_cvm")):
        for prefix in ("", "first_stage_ramdisk/", "first_stage_ramdisk/system/etc/", "system/etc/"):
            paths.append((prefix + name, fstab_data, stat.S_IFREG | 0o644))
    archive = bytearray()
    for inode, (name, data, mode) in enumerate(paths, start=1):
        archive.extend(cpio_newc_entry(name, data, mode, inode))
    archive.extend(cpio_newc_entry("TRAILER!!!", b"", stat.S_IFREG, len(paths) + 1))
    archive.extend(b"\0" * ((-len(archive)) % 512))
    return bytes(archive)


def merge_bootconfig(original: bytes, fstab_name: str, android_release: str) -> bytes:
    try:
        text = original.rstrip(b"\0").decode("utf-8")
    except UnicodeDecodeError as error:
        fail("CFI_BOOT_FORMAT", f"vendor bootconfig is not UTF-8: {error}")
    suffix = fstab_name.removeprefix("fstab.")
    required = [
        "androidboot.slot_suffix=_a",
        "androidboot.force_normal_boot=1",
        "androidboot.verifiedbootstate=orange",
        "androidboot.hardware=cutf_cvm",
        "androidboot.selinux=permissive",
        "androidboot.serialno=CUTTLEFISHCVD01",
        "androidboot.lcd_density=320",
        "androidboot.setupwizard_mode=DISABLED",
        "androidboot.enable_bootanimation=1",
        f"androidboot.fstab_suffix={suffix}",
        "androidboot.boot_devices=pci0000:00/0000:00:03.0",
        "androidboot.hypervisor.version=crosvm",
        "androidboot.hypervisor.vm.supported=1",
        "androidboot.hypervisor.protected_vm.supported=0",
        "androidboot.modem_simulator_ports=9697",
        "androidboot.vendor.apex.com.android.hardware.keymint=com.android.hardware.keymint.rust_nonsecure",
        "androidboot.vendor.apex.com.android.hardware.gatekeeper=com.android.hardware.gatekeeper.nonsecure",
    ]
    if android_release == "17":
        # Match Android 17's official Cuttlefish guest_swiftshader profile on
        # HD Windows: hvc console, 4915 MiB DDR accounting, a 2D virtio-gpu,
        # Guest ANGLE/Pastel, ranchu composer, and insecure guest key services.
        # This avoids requiring a compatible host gfxstream renderer merely to
        # complete boot; hardware gfxstream remains a separately certified
        # runtime capability.
        required.extend(
            (
                "androidboot.console=hvc0",
                "androidboot.serialconsole=1",
                "androidboot.ddr_size=4915MB",
                "androidboot.hypervisor.version=cf-crosvm",
                "androidboot.enable_confirmationui=1",
                "androidboot.wifi_mac_prefix=5554",
                "androidboot.hw_timeout_multiplier=3",
                "androidboot.wifi_impl=virt_wifi",
                "androidboot.openthread_node_id=1",
                "androidboot.cpuvulkan.version=4206592",
                "androidboot.hardware.gralloc=minigbm",
                "androidboot.hardware.hwcomposer=ranchu",
                "androidboot.hardware.hwcomposer.display_finder_mode=drm",
                "androidboot.hardware.hwcomposer.display_framebuffer_format=rgba",
                "androidboot.hardware.egl=angle",
                "androidboot.hardware.vulkan=pastel",
                "androidboot.hardware.gltransport=virtio-gpu-asg",
                "androidboot.opengles.version=196609",
                "androidboot.vendor.apex.com.android.hardware.secure_element=com.android.hardware.secure_element",
                "androidboot.vendor.apex.com.android.hardware.strongbox=none",
                "androidboot.vendor.apex.com.android.hardware.graphics.composer=com.android.hardware.graphics.composer.ranchu",
                "androidboot.vendor.apex.com.google.emulated.camera.provider.hal=com.google.emulated.camera.provider.hal",
            )
        )
    order: list[str] = []
    values: dict[str, str] = {}
    for raw in [*text.splitlines(), *required]:
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        key = line.split("=", 1)[0]
        if key not in values:
            order.append(key)
        values[key] = line
    return ("\n".join(values[key] for key in order) + "\n").encode("utf-8")


def write_initrd(
    output: Path,
    base_ramdisks: list[bytes],
    vendor_ramdisk: bytes,
    fstab_name: str,
    fstab_text: str,
    vendor_bootconfig: bytes,
    android_release: str,
) -> None:
    bootconfig = merge_bootconfig(vendor_bootconfig, fstab_name, android_release)
    checksum = sum(bootconfig) & 0xFFFFFFFF
    with output.open("wb") as stream:
        for ramdisk in base_ramdisks:
            stream.write(ramdisk)
        stream.write(vendor_ramdisk)
        stream.write(make_fstab_cpio(fstab_name, fstab_text.encode("utf-8")))
        stream.write(bootconfig)
        stream.write(struct.pack("<II", len(bootconfig), checksum))
        stream.write(b"#BOOTCONFIG\n")


def sparse_expanded_size(path: Path) -> int | None:
    with path.open("rb") as stream:
        header = stream.read(28)
    if len(header) < 4 or struct.unpack_from("<I", header)[0] != ANDROID_SPARSE_MAGIC:
        return None
    if len(header) != 28:
        fail("CFI_SPARSE_FORMAT", f"truncated sparse header: {path.name}")
    _, major, _, file_header_size, chunk_header_size, block_size, total_blocks, _, _ = struct.unpack(
        "<IHHHHIIII", header
    )
    if major != 1 or file_header_size < 28 or chunk_header_size < 12 or block_size == 0:
        fail("CFI_SPARSE_FORMAT", f"unsupported sparse header: {path.name}")
    return block_size * total_blocks


def expand_sparse_image(source: Path, destination: Path) -> Path:
    expanded_size = sparse_expanded_size(source)
    if expanded_size is None:
        return source
    with source.open("rb") as src, destination.open("w+b") as dst:
        header = src.read(28)
        _, major, _, file_header_size, chunk_header_size, block_size, total_blocks, total_chunks, _ = struct.unpack(
            "<IHHHHIIII", header
        )
        if major != 1:
            fail("CFI_SPARSE_FORMAT", f"unsupported sparse major version {major}")
        if file_header_size > 28:
            src.read(file_header_size - 28)
        logical_blocks = 0
        for _ in range(total_chunks):
            chunk_header = src.read(chunk_header_size)
            if len(chunk_header) != chunk_header_size:
                fail("CFI_SPARSE_FORMAT", f"truncated sparse chunk in {source.name}")
            chunk_type, _, chunk_blocks, total_size = struct.unpack_from("<HHII", chunk_header)
            data_size = total_size - chunk_header_size
            output_size = chunk_blocks * block_size
            if chunk_type == SPARSE_CHUNK_RAW:
                if data_size != output_size:
                    fail("CFI_SPARSE_FORMAT", f"invalid raw sparse chunk in {source.name}")
                remaining = data_size
                while remaining:
                    data = src.read(min(COPY_CHUNK, remaining))
                    if not data:
                        fail("CFI_SPARSE_FORMAT", f"truncated raw sparse data in {source.name}")
                    dst.write(data)
                    remaining -= len(data)
            elif chunk_type == SPARSE_CHUNK_FILL:
                if data_size != 4:
                    fail("CFI_SPARSE_FORMAT", f"invalid fill sparse chunk in {source.name}")
                pattern = src.read(4)
                repeated = pattern * (COPY_CHUNK // 4)
                remaining = output_size
                while remaining:
                    chunk = repeated[: min(len(repeated), remaining)]
                    dst.write(chunk)
                    remaining -= len(chunk)
            elif chunk_type == SPARSE_CHUNK_DONT_CARE:
                if data_size != 0:
                    fail("CFI_SPARSE_FORMAT", f"invalid hole sparse chunk in {source.name}")
                dst.seek(output_size, os.SEEK_CUR)
            elif chunk_type == SPARSE_CHUNK_CRC32:
                if data_size != 4 or chunk_blocks != 0:
                    fail("CFI_SPARSE_FORMAT", f"invalid CRC sparse chunk in {source.name}")
                src.read(4)
            else:
                fail("CFI_SPARSE_FORMAT", f"unknown sparse chunk 0x{chunk_type:04x}")
            logical_blocks += chunk_blocks
        if logical_blocks != total_blocks:
            fail("CFI_SPARSE_FORMAT", f"sparse block count mismatch in {source.name}")
        dst.truncate(expanded_size)
    return destination


def write_zeros(path: Path, size: int) -> None:
    with path.open("wb") as stream:
        stream.truncate(size)


def copy_image_to_disk(source: Path, destination) -> None:
    zero = b"\0" * COPY_CHUNK
    with source.open("rb") as stream:
        while chunk := stream.read(COPY_CHUNK):
            if chunk == zero[: len(chunk)]:
                destination.seek(len(chunk), os.SEEK_CUR)
            else:
                destination.write(chunk)


def gpt_entry(label: str, first_lba: int, last_lba: int, source_digest: str) -> bytes:
    entry = bytearray(GPT_ENTRY_SIZE)
    entry[0:16] = ANDROID_BASIC_DATA_GUID.bytes_le
    entry[16:32] = uuid.uuid5(IMPORT_NAMESPACE, f"{source_digest}:{label}").bytes_le
    struct.pack_into("<QQ", entry, 32, first_lba, last_lba)
    encoded = label.encode("utf-16-le")
    if len(encoded) > 72:
        fail("CFI_DISK_LAYOUT", f"partition label is too long: {label}")
    entry[56 : 56 + len(encoded)] = encoded
    return bytes(entry)


def gpt_header(
    current_lba: int,
    backup_lba: int,
    first_usable_lba: int,
    last_usable_lba: int,
    disk_guid: uuid.UUID,
    entries_lba: int,
    entries_crc: int,
) -> bytes:
    header = bytearray(92)
    header[:8] = b"EFI PART"
    struct.pack_into("<II", header, 8, 0x00010000, len(header))
    struct.pack_into("<QQQQ", header, 24, current_lba, backup_lba, first_usable_lba, last_usable_lba)
    header[56:72] = disk_guid.bytes_le
    struct.pack_into("<QIII", header, 72, entries_lba, GPT_ENTRY_COUNT, GPT_ENTRY_SIZE, entries_crc)
    crc_copy = bytearray(header)
    struct.pack_into("<I", crc_copy, 16, 0)
    struct.pack_into("<I", header, 16, zlib.crc32(crc_copy) & 0xFFFFFFFF)
    return bytes(header)


def protective_mbr(disk_sectors: int) -> bytes:
    mbr = bytearray(SECTOR_SIZE)
    mbr[0x1BE + 4] = 0xEE
    struct.pack_into("<II", mbr, 0x1BE + 8, 1, min(disk_sectors - 1, 0xFFFFFFFF))
    mbr[0x1FE:0x200] = b"\x55\xaa"
    return bytes(mbr)


def build_aggregate(partitions: list[tuple[str, Path]], output: Path, source_digest: str) -> list[str]:
    layout: list[tuple[str, Path, int, int]] = []
    next_lba = GPT_ALIGNMENT_LBA
    for label, path in partitions:
        first_lba = align_up(next_lba, GPT_ALIGNMENT_LBA)
        size_lba = max(1, align_up(path.stat().st_size, SECTOR_SIZE) // SECTOR_SIZE)
        size_lba = align_up(size_lba, GPT_ALIGNMENT_LBA)
        last_lba = first_lba + size_lba - 1
        layout.append((label, path, first_lba, last_lba))
        next_lba = last_lba + 1
    entries_size = GPT_ENTRY_COUNT * GPT_ENTRY_SIZE
    entries_lbas = align_up(entries_size, SECTOR_SIZE) // SECTOR_SIZE
    backup_entries_lba = align_up(layout[-1][3] + 1, GPT_ALIGNMENT_LBA)
    disk_sectors = backup_entries_lba + entries_lbas + 1
    backup_header_lba = disk_sectors - 1
    entries = bytearray(entries_size)
    for index, (label, _, first_lba, last_lba) in enumerate(layout):
        start = index * GPT_ENTRY_SIZE
        entries[start : start + GPT_ENTRY_SIZE] = gpt_entry(label, first_lba, last_lba, source_digest)
    entries_crc = zlib.crc32(entries) & 0xFFFFFFFF
    disk_guid = uuid.uuid5(IMPORT_NAMESPACE, f"{source_digest}:disk")
    primary = gpt_header(1, backup_header_lba, 34, backup_entries_lba - 1, disk_guid, 2, entries_crc)
    backup = gpt_header(
        backup_header_lba,
        1,
        34,
        backup_entries_lba - 1,
        disk_guid,
        backup_entries_lba,
        entries_crc,
    )
    with output.open("w+b") as disk:
        disk.write(protective_mbr(disk_sectors))
        disk.write(primary)
        disk.seek(2 * SECTOR_SIZE)
        disk.write(entries)
        for _, path, first_lba, last_lba in layout:
            disk.seek(first_lba * SECTOR_SIZE)
            copy_image_to_disk(path, disk)
            disk.seek((last_lba + 1) * SECTOR_SIZE)
        disk.seek(backup_entries_lba * SECTOR_SIZE)
        disk.write(entries)
        disk.seek(backup_header_lba * SECTOR_SIZE)
        disk.write(backup)
        disk.truncate(disk_sectors * SECTOR_SIZE)
    return [label for label, _, _, _ in layout]


def prepare_partitions(images: dict[str, Path], scratch: Path) -> list[tuple[str, Path]]:
    if "misc.img" not in images:
        images["misc.img"] = scratch / "misc.img"
        write_zeros(images["misc.img"], 64 * 1024 * 1024)
    if "factory_reset_protected.img" not in images:
        images["factory_reset_protected.img"] = scratch / "factory_reset_protected.img"
        write_zeros(images["factory_reset_protected.img"], 1024 * 1024)
    if "metadata.img" not in images:
        images["metadata.img"] = scratch / "metadata.img"
        write_zeros(images["metadata.img"], METADATA_PARTITION_SIZE)
    mapping: list[tuple[str, str]] = [
        ("misc", "misc.img"),
        ("boot_a", "boot.img"),
        ("boot_b", "boot.img"),
    ]
    if "init_boot.img" in images:
        mapping.extend((("init_boot_a", "init_boot.img"), ("init_boot_b", "init_boot.img")))
    mapping.extend((("vendor_boot_a", "vendor_boot.img"), ("vendor_boot_b", "vendor_boot.img")))
    mapping.extend((("vbmeta_a", "vbmeta.img"), ("vbmeta_b", "vbmeta.img")))
    mapping.extend((("vbmeta_system_a", "vbmeta_system.img"), ("vbmeta_system_b", "vbmeta_system.img")))
    for name, label in (
        ("vbmeta_vendor_dlkm.img", "vbmeta_vendor_dlkm"),
        ("vbmeta_system_dlkm.img", "vbmeta_system_dlkm"),
    ):
        if name in images:
            mapping.extend(((f"{label}_a", name), (f"{label}_b", name)))
    mapping.extend(
        (
            ("super", "super.img"),
            ("userdata", "userdata.img"),
            ("frp", "factory_reset_protected.img"),
            ("metadata", "metadata.img"),
        )
    )
    raw_paths: dict[str, Path] = {}
    for _, image_name in mapping:
        if image_name not in raw_paths:
            raw_paths[image_name] = expand_sparse_image(images[image_name], scratch / f"raw-{image_name}")
    return [(label, raw_paths[name]) for label, name in mapping]


def file_record(role: str, path: Path, guest_root: Path) -> dict[str, object]:
    return {
        "role": role,
        "relative_path": path.relative_to(guest_root).as_posix(),
        "size": path.stat().st_size,
        "sha256": sha256_file(path),
    }


def import_release(
    image_zip: Path,
    target_files_zip: Path | None,
    ota_metadata: Path | None,
    sensor_injector: Path | None,
    output_dir: Path,
) -> Path:
    if target_files_zip is None and ota_metadata is None:
        fail("CFI_ARGUMENT_INVALID", "either --target-files-zip or --ota-metadata is required")
    inputs = [("image ZIP", image_zip)]
    if target_files_zip is not None:
        inputs.append(("target-files ZIP", target_files_zip))
    if ota_metadata is not None:
        inputs.append(("OTA metadata", ota_metadata))
    if sensor_injector is not None:
        inputs.append(("sensor injector", sensor_injector))
    for label, path in inputs:
        if not path.is_file():
            fail("CFI_INPUT_INVALID", f"{label} is not a regular file: {path}")
    if sensor_injector is not None:
        validate_sensor_injector(sensor_injector)
    if output_dir.exists():
        fail("CFI_OUTPUT_EXISTS", f"refusing to replace existing output: {output_dir}")
    output_dir.parent.mkdir(parents=True, exist_ok=True)
    events = EventLog()
    started = time.monotonic()
    events.emit("cuttlefish.import", "started")
    image_digest = sha256_file(image_zip)
    source_digests = {"image_zip_sha256": image_digest}
    if target_files_zip is not None:
        source_digests["target_files_zip_sha256"] = sha256_file(target_files_zip)
    if ota_metadata is not None:
        source_digests["ota_metadata_sha256"] = sha256_file(ota_metadata)
    source_payload = json.dumps(source_digests, sort_keys=True, separators=(",", ":")).encode("utf-8")
    source_manifest_digest = hashlib.sha256(source_payload).hexdigest()
    staging = Path(tempfile.mkdtemp(prefix=f".{output_dir.name}.staging-", dir=output_dir.parent))
    try:
        scratch = staging / "scratch"
        guest = staging / "guest"
        scratch.mkdir()
        guest.mkdir()
        boundary = time.monotonic()
        images = extract_release_images(image_zip, scratch / "images")
        vendor_ramdisk, vendor_bootconfig = parse_vendor_boot_image(images["vendor_boot.img"])
        if target_files_zip is not None:
            fstab_name, fstab_source, properties, product = inspect_target_files(target_files_zip)
            if ota_metadata is not None:
                ota_properties, ota_product = inspect_ota_metadata(ota_metadata, image_zip)
                for key in ("ro.build.version.release", "ro.build.version.sdk"):
                    if properties.get(key) != ota_properties.get(key):
                        fail("CFI_BUILD_MISMATCH", f"target-files and OTA metadata disagree on {key}")
                if product != ota_product:
                    fail("CFI_BUILD_MISMATCH", "target-files and OTA metadata products do not match")
        else:
            assert ota_metadata is not None
            properties, product = inspect_ota_metadata(ota_metadata, image_zip)
            fstab_name, fstab_source = inspect_vendor_ramdisk_fstab(vendor_ramdisk)
        events.emit(
            "cuttlefish.source.verify",
            "succeeded",
            duration_ms=int((time.monotonic() - boundary) * 1000),
            product=product,
        )
        boundary = time.monotonic()
        kernel, boot_ramdisk = parse_boot_image(images["boot.img"])
        validate_x86_kernel(kernel)
        base_ramdisks: list[bytes] = []
        if "init_boot.img" in images:
            init_ramdisk = parse_init_boot_image(images["init_boot.img"])
            if init_ramdisk:
                base_ramdisks.append(init_ramdisk)
        if boot_ramdisk:
            base_ramdisks.append(boot_ramdisk)
        if not base_ramdisks:
            fail("CFI_BOOT_FORMAT", "neither init_boot.img nor boot.img contains an init ramdisk")
        fstab_text = filter_fstab(fstab_source)
        android_release = properties.get("ro.build.version.release", "")
        kernel_path = guest / "kernel"
        kernel_path.write_bytes(kernel)
        fstab_path = guest / "android_fstab.dt"
        fstab_path.write_text(fstab_text, encoding="utf-8", newline="\n")
        initrd_path = guest / "initrd_android.img"
        write_initrd(
            initrd_path,
            base_ramdisks,
            vendor_ramdisk,
            fstab_name,
            fstab_text,
            vendor_bootconfig,
            android_release,
        )
        events.emit(
            "cuttlefish.boot.extract",
            "succeeded",
            duration_ms=int((time.monotonic() - boundary) * 1000),
        )
        boundary = time.monotonic()
        partitions = prepare_partitions(images, scratch)
        rootfs_path = guest / "aggregate_android.img"
        partition_labels = build_aggregate(partitions, rootfs_path, source_manifest_digest)
        events.emit(
            "cuttlefish.disk.assemble",
            "succeeded",
            duration_ms=int((time.monotonic() - boundary) * 1000),
            partition_count=len(partition_labels),
        )
        files = [
            file_record("kernel", kernel_path, guest),
            file_record("initrd", initrd_path, guest),
            file_record("rootfs", rootfs_path, guest),
            file_record("android_fstab", fstab_path, guest),
        ]
        if sensor_injector is not None:
            sensor_path = guest / "hd-sensor-injector"
            shutil.copyfile(sensor_injector, sensor_path)
            with contextlib.suppress(OSError):
                sensor_path.chmod(0o755)
            files.append(file_record("sensor-injector", sensor_path, guest))
        production_blockers = [
            "unsigned_staging",
            "hd_guest_profile_not_yet_verified",
            "real_guest_gate_not_run",
            "host_certification_not_issued",
        ]
        if android_release == "17":
            production_blockers.append("hd_runtime_android17_split_sensors_profile_not_integrated")
        elif android_release != "15":
            production_blockers.append("hd_runtime_android_version_not_supported")
        if sensor_injector is None:
            production_blockers.append("sensor_injector_missing")
        manifest_source = dict(source_digests)
        manifest_source.update(
            {
                "source_manifest_digest": source_manifest_digest,
                "product": product,
                "android_release": android_release,
                "android_sdk": properties.get("ro.build.version.sdk", ""),
                "build_fingerprint": properties.get("ro.build.fingerprint", ""),
                "build_incremental": properties.get("ro.build.version.incremental", ""),
                "security_patch": properties.get("ro.build.version.security_patch", ""),
                "fstab_member": fstab_name,
                "fstab_source": "target-files" if target_files_zip is not None else "vendor_boot.img",
            }
        )
        manifest = {
            "schema_version": SCHEMA_VERSION,
            "kind": "cuttlefish-hd-guest-staging",
            "production_ready": False,
            "production_blockers": production_blockers,
            "source": manifest_source,
            "architecture": "x86_64",
            "guest_profile": (
                {
                    "hvc_layout": "cuttlefish-android17-split-sensors-v1",
                    "virtio_console_count": 20,
                    "sensors_control_hvc": 18,
                    "sensors_data_hvc": 19,
                }
                if android_release == "17"
                else {
                    "hvc_layout": "cuttlefish-android15-single-sensors-v1",
                    "virtio_console_count": 19,
                    "sensors_hvc": 13,
                }
            ),
            "partition_labels": partition_labels,
            "files": files,
        }
        (staging / "import-manifest-v1.json").write_text(
            json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8", newline="\n"
        )
        publish_request = {
            "schema_version": SCHEMA_VERSION,
            "command": "xtask publish-bundle",
            "kind": "guest",
            "input_root": "guest",
            "platform": "android",
            "architecture": "x86_64",
            "source_manifest_digest": source_manifest_digest,
            "files": {record["role"]: record["relative_path"] for record in files},
            "executable_roles": ["sensor-injector"] if sensor_injector is not None else [],
            "capabilities_requiring_real_guest_validation": [
                (
                    "android-15.0.0_r14"
                    if android_release == "15"
                    else f"android-{android_release}-build-{properties.get('ro.build.version.incremental', 'unknown')}"
                ),
                "hd-guest-profile-v2",
                "hd-device-bridge-v2",
                (
                    "cuttlefish-hvc-android17-split-sensors-v1"
                    if android_release == "17"
                    else "cuttlefish-hvc-android15-single-sensors-v1"
                ),
            ],
            "note": "Do not publish these capability claims until the converted Guest passes the HD real-guest and device-profile gates.",
        }
        (staging / "publish-request-v1.json").write_text(
            json.dumps(publish_request, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
            newline="\n",
        )
        shutil.rmtree(scratch)
        events.emit(
            "cuttlefish.import",
            "succeeded",
            duration_ms=int((time.monotonic() - started) * 1000),
            source_manifest_digest=source_manifest_digest,
        )
        events.write(staging / "import-events.jsonl")
        staging.replace(output_dir)
        return output_dir
    except Exception:
        shutil.rmtree(staging, ignore_errors=True)
        raise


def make_boot_image(kernel: bytes, ramdisk: bytes) -> bytes:
    header = bytearray(PAGE_SIZE_V4)
    header[:8] = BOOT_MAGIC
    struct.pack_into("<IIII", header, 8, len(kernel), len(ramdisk), 0, 1584)
    struct.pack_into("<I", header, 40, 4)
    result = bytearray(header)
    result.extend(kernel)
    result.extend(b"\0" * ((-len(result)) % PAGE_SIZE_V4))
    result.extend(ramdisk)
    return bytes(result)


def make_vendor_boot_image(vendor_ramdisk: bytes, bootconfig: bytes, page_size: int = PAGE_SIZE_V4) -> bytes:
    header = bytearray(align_up(2128, page_size))
    header[:8] = VENDOR_BOOT_MAGIC
    struct.pack_into("<IIIII", header, 8, 4, page_size, 0, 0, len(vendor_ramdisk))
    struct.pack_into("<I", header, 2096, 2128)
    table_entry = b"T" * 108
    struct.pack_into("<III", header, 2112, len(table_entry), 1, len(table_entry))
    struct.pack_into("<I", header, 2124, len(bootconfig))
    result = bytearray(header)
    result.extend(vendor_ramdisk)
    result.extend(b"\0" * ((-len(result)) % page_size))
    result.extend(table_entry)
    result.extend(b"\0" * ((-len(result)) % page_size))
    result.extend(bootconfig)
    return bytes(result)


def make_legacy_lz4_uncompressed(payload: bytes) -> bytes:
    result = bytearray(LZ4_LEGACY_MAGIC)
    for offset in range(0, len(payload), COPY_CHUNK):
        block = payload[offset : offset + COPY_CHUNK]
        result.extend(struct.pack("<I", len(block) | 0x80000000))
        result.extend(block)
    result.extend(struct.pack("<I", 0))
    return bytes(result)


def make_sparse_image(payload: bytes, block_size: int = 4096) -> bytes:
    padded = payload + b"\0" * ((-len(payload)) % block_size)
    blocks = len(padded) // block_size
    header = struct.pack(
        "<IHHHHIIII", ANDROID_SPARSE_MAGIC, 1, 0, 28, 12, block_size, blocks, 1, 0
    )
    chunk = struct.pack("<HHII", SPARSE_CHUNK_RAW, 0, blocks, 12 + len(padded))
    return header + chunk + padded


def make_elf64_x86() -> bytes:
    header = bytearray(64)
    header[:6] = b"\x7fELF\x02\x01"
    struct.pack_into("<H", header, 16, 2)
    struct.pack_into("<H", header, 18, 62)
    return bytes(header)


def expect_failure(expected_code: str, operation) -> None:
    try:
        operation()
    except ImportFailure as error:
        if error.code != expected_code:
            fail(
                "CFI_SELF_CHECK_FAILED",
                f"expected {expected_code}, got {error.code}: {error}",
            )
        return
    fail("CFI_SELF_CHECK_FAILED", f"expected failure {expected_code} was not raised")


def self_check() -> None:
    with tempfile.TemporaryDirectory(prefix="hd-cuttlefish-import-self-check-") as raw:
        root = Path(raw)
        image_zip = root / "aosp_cf_x86_64_phone-img-test.zip"
        target_zip = root / "aosp_cf_x86_64_phone-target_files-test.zip"
        sensor = root / "sensor-injector"
        sensor.write_bytes(make_elf64_x86())
        kernel = bytearray(4096)
        kernel[0x202:0x206] = b"HdrS"
        base_cpio = make_fstab_cpio("fstab.cutf_cvm", b"")
        fstab = "\n".join(
            (
                "system /system erofs ro wait,logical,first_stage_mount",
                "/dev/block/by-name/userdata /data f2fs noatime wait,check,formattable",
            )
        )
        vendor_cpio = make_fstab_cpio("fstab.cf.f2fs.hctr2", fstab.encode("utf-8"))
        with zipfile.ZipFile(image_zip, "w", allowZip64=True) as archive:
            archive.writestr("boot.img", make_boot_image(bytes(kernel), b""), zipfile.ZIP_STORED)
            archive.writestr("init_boot.img", make_boot_image(b"", base_cpio), zipfile.ZIP_STORED)
            archive.writestr(
                "vendor_boot.img",
                make_vendor_boot_image(
                    make_legacy_lz4_uncompressed(vendor_cpio),
                    b"androidboot.hardware=cutf_cvm\n",
                    page_size=2048,
                ),
                zipfile.ZIP_STORED,
            )
            for name in ("vbmeta.img", "vbmeta_system.img"):
                archive.writestr(name, b"X" * 4096, zipfile.ZIP_STORED)
            archive.writestr("super.img", make_sparse_image(b"super"), zipfile.ZIP_STORED)
            archive.writestr("userdata.img", make_sparse_image(b"userdata"), zipfile.ZIP_STORED)
        props = "\n".join(
            (
                "ro.build.version.release=15",
                "ro.build.version.sdk=35",
                "ro.product.cpu.abi=x86_64",
                "ro.build.type=userdebug",
                "ro.debuggable=1",
                "ro.product.name=aosp_cf_x86_64_phone",
                "ro.build.fingerprint=test/cuttlefish:15/test:userdebug/test-keys",
            )
        )
        with zipfile.ZipFile(target_zip, "w", allowZip64=True) as archive:
            archive.writestr("SYSTEM/build.prop", props, zipfile.ZIP_STORED)
            archive.writestr("VENDOR/etc/fstab.cf.f2fs.hctr2", fstab, zipfile.ZIP_STORED)
        wrong_version_zip = root / "wrong-version-target_files.zip"
        with zipfile.ZipFile(wrong_version_zip, "w", allowZip64=True) as archive:
            archive.writestr(
                "SYSTEM/build.prop",
                props.replace("ro.build.version.sdk=35", "ro.build.version.sdk=34"),
                zipfile.ZIP_STORED,
            )
            archive.writestr("VENDOR/etc/fstab.cf.f2fs.hctr2", fstab, zipfile.ZIP_STORED)
        expect_failure("CFI_BUILD_MISMATCH", lambda: inspect_target_files(wrong_version_zip))
        output = root / "output"
        import_release(image_zip, target_zip, None, sensor, output)
        required = (
            output / "guest/kernel",
            output / "guest/initrd_android.img",
            output / "guest/aggregate_android.img",
            output / "guest/android_fstab.dt",
            output / "guest/hd-sensor-injector",
            output / "import-manifest-v1.json",
            output / "publish-request-v1.json",
            output / "import-events.jsonl",
        )
        if not all(path.is_file() and path.stat().st_size > 0 for path in required):
            fail("CFI_SELF_CHECK_FAILED", "self-check output is incomplete")
        with (output / "guest/aggregate_android.img").open("rb") as stream:
            stream.seek(SECTOR_SIZE)
            if stream.read(8) != b"EFI PART":
                fail("CFI_SELF_CHECK_FAILED", "self-check aggregate has no GPT header")
        manifest = json.loads((output / "import-manifest-v1.json").read_text(encoding="utf-8"))
        if manifest["production_ready"] is not False or len(manifest["files"]) != 5:
            fail("CFI_SELF_CHECK_FAILED", "self-check manifest contract mismatch")
        ota_metadata = root / "aosp_cf_x86_64_phone-ota_metadata-test.txt"
        ota_metadata.write_text(
            "\n".join(
                (
                    "post-build=generic/aosp_cf_x86_64_only_phone/vsoc_x86_64_only:17/TEST/15885347:userdebug/test-keys",
                    "post-build-incremental=15885347",
                    "post-sdk-level=37",
                    "post-security-patch-level=2026-06-05",
                    "pre-device=vsoc_x86_64_only",
                )
            )
            + "\n",
            encoding="utf-8",
        )
        latest_image_zip = root / "aosp_cf_x86_64_only_phone-img-15885347.zip"
        shutil.copyfile(image_zip, latest_image_zip)
        latest_output = root / "latest-output"
        import_release(latest_image_zip, None, ota_metadata, None, latest_output)
        latest_manifest = json.loads(
            (latest_output / "import-manifest-v1.json").read_text(encoding="utf-8")
        )
        if latest_manifest["source"]["android_release"] != "17":
            fail("CFI_SELF_CHECK_FAILED", "OTA metadata import did not preserve Android release")
        if (
            "hd_runtime_android17_split_sensors_profile_not_integrated"
            not in latest_manifest["production_blockers"]
        ):
            fail("CFI_SELF_CHECK_FAILED", "latest release staging is missing runtime blocker")
        if latest_manifest.get("guest_profile") != {
            "hvc_layout": "cuttlefish-android17-split-sensors-v1",
            "virtio_console_count": 20,
            "sensors_control_hvc": 18,
            "sensors_data_hvc": 19,
        }:
            fail("CFI_SELF_CHECK_FAILED", "latest release HVC profile is incorrect")
        if "sensor_injector_missing" not in latest_manifest["production_blockers"]:
            fail("CFI_SELF_CHECK_FAILED", "sensor-less staging is missing its production blocker")
        latest_initrd = (latest_output / "guest/initrd_android.img").read_bytes()
        bootconfig_magic = b"#BOOTCONFIG\n"
        if not latest_initrd.endswith(bootconfig_magic):
            fail("CFI_SELF_CHECK_FAILED", "latest release initrd has no bootconfig trailer")
        trailer_offset = len(latest_initrd) - len(bootconfig_magic) - 8
        bootconfig_size, bootconfig_checksum = struct.unpack_from("<II", latest_initrd, trailer_offset)
        bootconfig_offset = trailer_offset - bootconfig_size
        if bootconfig_offset < 0:
            fail("CFI_SELF_CHECK_FAILED", "latest release bootconfig size is invalid")
        latest_bootconfig = latest_initrd[bootconfig_offset:trailer_offset]
        if (sum(latest_bootconfig) & 0xFFFFFFFF) != bootconfig_checksum:
            fail("CFI_SELF_CHECK_FAILED", "latest release bootconfig checksum is invalid")
        latest_bootconfig_lines = set(latest_bootconfig.decode("utf-8").splitlines())
        required_android17_bootconfig = {
            "androidboot.hypervisor.version=cf-crosvm",
            "androidboot.hardware.egl=angle",
            "androidboot.hardware.hwcomposer=ranchu",
            "androidboot.hardware.vulkan=pastel",
            "androidboot.cpuvulkan.version=4206592",
            "androidboot.openthread_node_id=1",
            "androidboot.vendor.apex.com.android.hardware.graphics.composer=com.android.hardware.graphics.composer.ranchu",
            "androidboot.vendor.apex.com.android.hardware.secure_element=com.android.hardware.secure_element",
            "androidboot.vendor.apex.com.android.hardware.strongbox=none",
            "androidboot.vendor.apex.com.google.emulated.camera.provider.hal=com.google.emulated.camera.provider.hal",
        }
        if not required_android17_bootconfig.issubset(latest_bootconfig_lines):
            fail("CFI_SELF_CHECK_FAILED", "latest release bootconfig profile is incomplete")
        unsafe_zip = root / "unsafe.zip"
        with zipfile.ZipFile(unsafe_zip, "w") as archive:
            archive.writestr("../boot.img", b"unsafe", zipfile.ZIP_STORED)
        expect_failure(
            "CFI_ARCHIVE_UNSAFE",
            lambda: extract_release_images(unsafe_zip, root / "unsafe-output"),
        )
        duplicate_zip = root / "duplicate.zip"
        with zipfile.ZipFile(image_zip) as source, zipfile.ZipFile(duplicate_zip, "w") as target:
            for info in source.infolist():
                target.writestr(info.filename, source.read(info.filename), zipfile.ZIP_STORED)
            target.writestr("nested/boot.img", b"duplicate", zipfile.ZIP_STORED)
        expect_failure(
            "CFI_ARCHIVE_AMBIGUOUS",
            lambda: extract_release_images(duplicate_zip, root / "duplicate-output"),
        )
        wrong_sensor = root / "wrong-sensor"
        wrong_sensor.write_bytes(b"not-an-elf")
        expect_failure("CFI_BUILD_MISMATCH", lambda: validate_sensor_injector(wrong_sensor))
    print("cuttlefish import self-check passed")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--image-zip", type=Path)
    parser.add_argument("--target-files-zip", type=Path)
    parser.add_argument("--ota-metadata", type=Path)
    parser.add_argument("--sensor-injector", type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--self-check", action="store_true")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        if args.self_check:
            if any(
                (args.image_zip, args.target_files_zip, args.ota_metadata, args.sensor_injector, args.output)
            ):
                fail("CFI_ARGUMENT_INVALID", "--self-check cannot be combined with import inputs")
            self_check()
            return 0
        missing = [
            name
            for name, value in (
                ("--image-zip", args.image_zip),
                ("--output", args.output),
            )
            if value is None
        ]
        if missing:
            fail("CFI_ARGUMENT_INVALID", f"missing required arguments: {', '.join(missing)}")
        if args.target_files_zip is None and args.ota_metadata is None:
            fail("CFI_ARGUMENT_INVALID", "either --target-files-zip or --ota-metadata is required")
        output = import_release(
            args.image_zip.resolve(),
            args.target_files_zip.resolve() if args.target_files_zip is not None else None,
            args.ota_metadata.resolve() if args.ota_metadata is not None else None,
            args.sensor_injector.resolve() if args.sensor_injector is not None else None,
            args.output.resolve(),
        )
        print(json.dumps({"status": "staged", "output": str(output)}, sort_keys=True))
        return 0
    except ImportFailure as error:
        print(
            json.dumps({"status": "failed", "error_code": error.code, "message": str(error)}, sort_keys=True),
            file=sys.stderr,
        )
        return 2
    except Exception as error:
        print(
            json.dumps(
                {"status": "failed", "error_code": "CFI_INTERNAL", "message": str(error)},
                sort_keys=True,
            ),
            file=sys.stderr,
        )
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
