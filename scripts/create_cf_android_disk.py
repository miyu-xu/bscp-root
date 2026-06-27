#!/usr/bin/env python3
"""Create a Cuttlefish-style Android GPT disk from built AOSP images."""

import argparse
import os
from pathlib import Path
import struct
import uuid
import zlib


SECTOR_SIZE = 512
GPT_ENTRY_SIZE = 128
GPT_ENTRY_COUNT = 128
ALIGN_LBA = 2048
ANDROID_BASIC_DATA_GUID = uuid.UUID("EBD0A0A2-B9E5-4433-87C0-68B6B72699C7")


def align_up(value, alignment):
    return ((value + alignment - 1) // alignment) * alignment


def sectors_for_size(size):
    return max(1, align_up(size, SECTOR_SIZE) // SECTOR_SIZE)


def image_size(path):
    return Path(path).stat().st_size


def write_zeroes(out_file, count):
    out_file.seek(count, os.SEEK_CUR)


def copy_image(src, dst):
    zero_chunk = b"\0" * (8 * 1024 * 1024)
    with open(src, "rb") as in_file:
        while True:
            chunk = in_file.read(8 * 1024 * 1024)
            if not chunk:
                return
            if chunk == zero_chunk[: len(chunk)]:
                dst.seek(len(chunk), os.SEEK_CUR)
            else:
                dst.write(chunk)


def guid_bytes(guid):
    return guid.bytes_le


def build_partition_entry(partition):
    entry = bytearray(GPT_ENTRY_SIZE)
    entry[0:16] = guid_bytes(ANDROID_BASIC_DATA_GUID)
    entry[16:32] = guid_bytes(uuid.uuid4())
    struct.pack_into("<Q", entry, 32, partition["first_lba"])
    struct.pack_into("<Q", entry, 40, partition["last_lba"])
    name = partition["label"].encode("utf-16-le")
    if len(name) > 72:
        raise ValueError(f"partition label is too long: {partition['label']}")
    entry[56 : 56 + len(name)] = name
    return entry


def build_gpt_header(
    *,
    current_lba,
    backup_lba,
    first_usable_lba,
    last_usable_lba,
    disk_guid,
    entries_lba,
    entries_crc,
):
    header = bytearray(92)
    header[0:8] = b"EFI PART"
    struct.pack_into("<I", header, 8, 0x00010000)
    struct.pack_into("<I", header, 12, len(header))
    struct.pack_into("<Q", header, 24, current_lba)
    struct.pack_into("<Q", header, 32, backup_lba)
    struct.pack_into("<Q", header, 40, first_usable_lba)
    struct.pack_into("<Q", header, 48, last_usable_lba)
    header[56:72] = guid_bytes(disk_guid)
    struct.pack_into("<Q", header, 72, entries_lba)
    struct.pack_into("<I", header, 80, GPT_ENTRY_COUNT)
    struct.pack_into("<I", header, 84, GPT_ENTRY_SIZE)
    struct.pack_into("<I", header, 88, entries_crc)

    crc_header = bytearray(header)
    struct.pack_into("<I", crc_header, 16, 0)
    struct.pack_into("<I", header, 16, zlib.crc32(crc_header) & 0xFFFFFFFF)
    return header


def build_protective_mbr(disk_sectors):
    mbr = bytearray(SECTOR_SIZE)
    mbr[0x1BE + 4] = 0xEE
    struct.pack_into("<I", mbr, 0x1BE + 8, 1)
    struct.pack_into("<I", mbr, 0x1BE + 12, min(disk_sectors - 1, 0xFFFFFFFF))
    mbr[0x1FE] = 0x55
    mbr[0x1FF] = 0xAA
    return mbr


def add_if_exists(partitions, label, path, required=False):
    path = Path(path)
    if not path.exists():
        if required:
            raise FileNotFoundError(f"required image missing for {label}: {path}")
        return
    partitions.append({"label": label, "path": path, "size": image_size(path)})


def default_partitions(product_dir, misc_image, metadata_image, frp_image):
    product_dir = Path(product_dir)
    partitions = []

    add_if_exists(partitions, "misc", misc_image, required=True)
    add_if_exists(partitions, "boot_a", product_dir / "boot.img", required=True)
    add_if_exists(partitions, "boot_b", product_dir / "boot.img", required=True)
    add_if_exists(partitions, "init_boot_a", product_dir / "init_boot.img")
    add_if_exists(partitions, "init_boot_b", product_dir / "init_boot.img")
    add_if_exists(partitions, "vendor_boot_a", product_dir / "vendor_boot.img", required=True)
    add_if_exists(partitions, "vendor_boot_b", product_dir / "vendor_boot.img", required=True)
    add_if_exists(partitions, "vbmeta_a", product_dir / "vbmeta.img", required=True)
    add_if_exists(partitions, "vbmeta_b", product_dir / "vbmeta.img", required=True)
    add_if_exists(partitions, "vbmeta_system_a", product_dir / "vbmeta_system.img", required=True)
    add_if_exists(partitions, "vbmeta_system_b", product_dir / "vbmeta_system.img", required=True)
    add_if_exists(partitions, "vbmeta_vendor_dlkm_a", product_dir / "vbmeta_vendor_dlkm.img")
    add_if_exists(partitions, "vbmeta_vendor_dlkm_b", product_dir / "vbmeta_vendor_dlkm.img")
    add_if_exists(partitions, "vbmeta_system_dlkm_a", product_dir / "vbmeta_system_dlkm.img")
    add_if_exists(partitions, "vbmeta_system_dlkm_b", product_dir / "vbmeta_system_dlkm.img")
    add_if_exists(partitions, "super", product_dir / "super.img", required=True)
    add_if_exists(partitions, "userdata", product_dir / "userdata.img", required=True)
    add_if_exists(partitions, "frp", frp_image, required=True)
    add_if_exists(partitions, "metadata", metadata_image, required=True)
    return partitions


def layout_partitions(partitions):
    next_lba = ALIGN_LBA
    laid_out = []
    for partition in partitions:
        first_lba = align_up(next_lba, ALIGN_LBA)
        size_lba = align_up(sectors_for_size(partition["size"]), ALIGN_LBA)
        last_lba = first_lba + size_lba - 1
        laid_out.append({**partition, "first_lba": first_lba, "last_lba": last_lba})
        next_lba = last_lba + 1
    return laid_out


def write_disk(partitions, output_path):
    entries_size = GPT_ENTRY_COUNT * GPT_ENTRY_SIZE
    entries_lbas = align_up(entries_size, SECTOR_SIZE) // SECTOR_SIZE
    partitions = layout_partitions(partitions)
    backup_entries_lba = align_up(partitions[-1]["last_lba"] + 1, ALIGN_LBA)
    disk_sectors = backup_entries_lba + entries_lbas + 1
    backup_header_lba = disk_sectors - 1
    last_usable_lba = backup_entries_lba - 1

    entries = bytearray(entries_size)
    for index, partition in enumerate(partitions):
        start = index * GPT_ENTRY_SIZE
        entries[start : start + GPT_ENTRY_SIZE] = build_partition_entry(partition)
    entries_crc = zlib.crc32(entries) & 0xFFFFFFFF

    disk_guid = uuid.uuid4()
    primary_header = build_gpt_header(
        current_lba=1,
        backup_lba=backup_header_lba,
        first_usable_lba=34,
        last_usable_lba=last_usable_lba,
        disk_guid=disk_guid,
        entries_lba=2,
        entries_crc=entries_crc,
    )
    backup_header = build_gpt_header(
        current_lba=backup_header_lba,
        backup_lba=1,
        first_usable_lba=34,
        last_usable_lba=last_usable_lba,
        disk_guid=disk_guid,
        entries_lba=backup_entries_lba,
        entries_crc=entries_crc,
    )

    output_path = Path(output_path)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with open(output_path, "wb") as out_file:
        out_file.write(build_protective_mbr(disk_sectors))
        out_file.write(primary_header)
        write_zeroes(out_file, SECTOR_SIZE - len(primary_header))
        out_file.write(entries)

        for partition in partitions:
            target_offset = partition["first_lba"] * SECTOR_SIZE
            if out_file.tell() < target_offset:
                write_zeroes(out_file, target_offset - out_file.tell())
            print(
                f"{partition['label']}: LBA {partition['first_lba']}-"
                f"{partition['last_lba']} <- {partition['path']}"
            )
            copy_image(partition["path"], out_file)
            end_offset = (partition["last_lba"] + 1) * SECTOR_SIZE
            if out_file.tell() < end_offset:
                write_zeroes(out_file, end_offset - out_file.tell())

        out_file.seek(backup_entries_lba * SECTOR_SIZE)
        out_file.write(entries)
        out_file.seek(backup_header_lba * SECTOR_SIZE)
        out_file.write(backup_header)
        out_file.truncate(disk_sectors * SECTOR_SIZE)

    print(
        f"created {output_path} "
        f"({disk_sectors * SECTOR_SIZE / 1024 / 1024:.1f} MiB)"
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--product-dir", required=True, help="AOSP product output directory")
    parser.add_argument("--misc-image", required=True, help="Writable misc image")
    parser.add_argument("--metadata-image", required=True, help="Writable metadata image")
    parser.add_argument("--frp-image", required=True, help="Writable FRP image")
    parser.add_argument("--output", required=True, help="Output aggregate raw disk")
    args = parser.parse_args()

    partitions = default_partitions(
        args.product_dir, args.misc_image, args.metadata_image, args.frp_image
    )
    write_disk(partitions, args.output)


if __name__ == "__main__":
    main()
