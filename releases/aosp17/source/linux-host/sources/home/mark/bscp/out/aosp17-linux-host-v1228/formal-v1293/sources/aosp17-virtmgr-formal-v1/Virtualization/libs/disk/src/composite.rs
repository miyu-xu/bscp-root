// Copyright 2019 The ChromiumOS Authors
// Copyright (C) 2026 The Android Open Source Project
//
// SPDX-License-Identifier: BSD-3-Clause
//
// Composite disk image creation (crosvm-compatible). Ported from
// external/crosvm/disk/src/composite.rs (create path only).

use std::collections::HashSet;
use std::convert::TryInto;
use std::fs::File;
use std::io::{self, ErrorKind, Write};
use std::path::Path;

use crc32fast::Hasher;
use prost::Message;
use uuid::Uuid;

use crate::cdisk_spec::{ComponentDisk, CompositeDisk, ReadWriteCapability};
use crate::gpt::{
    write_gpt_header, write_protective_mbr, GptPartitionEntry, GPT_BEGINNING_SIZE, GPT_END_SIZE,
    GPT_HEADER_SIZE, GPT_NUM_PARTITIONS, GPT_PARTITION_ENTRY_SIZE, SECTOR_SIZE,
};
use crate::{ImagePartitionType, PartitionInfo};

/// Crosvm-compatible composite disk format version.
const COMPOSITE_DISK_VERSION: u64 = 2;

const PARTITION_ALIGNMENT_SIZE: usize = GPT_BEGINNING_SIZE as usize
    - 2 * SECTOR_SIZE as usize
    - GPT_NUM_PARTITIONS as usize * GPT_PARTITION_ENTRY_SIZE as usize;
const HEADER_PADDING_LENGTH: usize = SECTOR_SIZE as usize - GPT_HEADER_SIZE as usize;
const PARTITION_SIZE_SHIFT: u8 = 12;

const LINUX_FILESYSTEM_GUID: Uuid = Uuid::from_u128(0x0FC63DAF_8483_4772_8E79_3D69D8477DE4);
const EFI_SYSTEM_PARTITION_GUID: Uuid = Uuid::from_u128(0xC12A7328_F81F_11D2_BA4B_00A0C93EC93B);

/// Magic prefix for composite disk files (crosvm `CDISK_MAGIC`).
pub const CDISK_MAGIC: &str = "composite_disk\x1d";

fn io_err(msg: impl Into<String>) -> io::Error {
    io::Error::new(ErrorKind::Other, msg.into())
}

fn align_to_power_of_2(val: u64, align_log: u8) -> u64 {
    let align = 1 << align_log;
    ((val + (align - 1)) / align) * align
}

impl PartitionInfo {
    fn aligned_size(&self) -> u64 {
        if self.size == 0 {
            return 1 << PARTITION_SIZE_SHIFT;
        }
        align_to_power_of_2(self.size, PARTITION_SIZE_SHIFT)
    }
}

impl ImagePartitionType {
    fn guid(self) -> Uuid {
        match self {
            Self::LinuxFilesystem => LINUX_FILESYSTEM_GUID,
            Self::EfiSystemPartition => EFI_SYSTEM_PARTITION_GUID,
        }
    }
}

fn write_beginning(
    file: &mut impl Write,
    disk_guid: Uuid,
    partitions: &[u8],
    partition_entries_crc32: u32,
    secondary_table_offset: u64,
    disk_size: u64,
) -> io::Result<()> {
    write_protective_mbr(file, disk_size).map_err(|e| io_err(e.to_string()))?;
    write_gpt_header(
        file,
        disk_guid,
        partition_entries_crc32,
        secondary_table_offset,
        false,
    )
    .map_err(|e| io_err(e.to_string()))?;
    file.write_all(&[0; HEADER_PADDING_LENGTH])?;
    file.write_all(partitions)?;
    file.write_all(&[0; PARTITION_ALIGNMENT_SIZE])?;
    Ok(())
}

fn write_end(
    file: &mut impl Write,
    disk_guid: Uuid,
    partitions: &[u8],
    partition_entries_crc32: u32,
    secondary_table_offset: u64,
    disk_size: u64,
) -> io::Result<()> {
    file.write_all(partitions)?;
    write_gpt_header(
        file,
        disk_guid,
        partition_entries_crc32,
        secondary_table_offset,
        true,
    )
    .map_err(|e| io_err(e.to_string()))?;
    file.write_all(&[0; HEADER_PADDING_LENGTH])?;
    let used_disk_size = secondary_table_offset + GPT_END_SIZE;
    let padding = disk_size - used_disk_size;
    file.write_all(&vec![0; padding as usize])?;
    Ok(())
}

fn create_gpt_entry(partition: &PartitionInfo, offset: u64) -> GptPartitionEntry {
    let mut partition_name: Vec<u16> = partition.label.encode_utf16().collect();
    partition_name.resize(36, 0);

    GptPartitionEntry {
        partition_type_guid: partition.partition_type.guid(),
        unique_partition_guid: partition.part_guid.unwrap_or_else(Uuid::new_v4),
        first_lba: offset / SECTOR_SIZE,
        last_lba: (offset + partition.aligned_size()) / SECTOR_SIZE - 1,
        attributes: 0,
        partition_name: partition_name.try_into().unwrap_or([0; 36]),
    }
}

fn create_component_disks(
    partition: &PartitionInfo,
    offset: u64,
    zero_filler_path: &str,
) -> io::Result<Vec<ComponentDisk>> {
    let aligned_size = partition.aligned_size();

    if partition.size == 0 {
        if partition.writable {
            return Err(io_err("read-write partition size is not a multiple of 4096"));
        }
        return Ok(vec![ComponentDisk {
            file_path: zero_filler_path.to_owned(),
            offset,
            read_write_capability: ReadWriteCapability::ReadOnly as i32,
        }]);
    }

    let mut component_disks = vec![ComponentDisk {
        file_path: partition
            .path
            .to_str()
            .ok_or_else(|| io_err(format!("invalid partition path {:?}", partition.path)))?
            .to_string(),
        offset,
        read_write_capability: if partition.writable {
            ReadWriteCapability::ReadWrite as i32
        } else {
            ReadWriteCapability::ReadOnly as i32
        },
    }];

    if partition.size != aligned_size {
        if partition.writable {
            return Err(io_err(
                "read-write partition size is not a multiple of 4096",
            ));
        }
        component_disks.push(ComponentDisk {
            offset: offset + partition.size,
            file_path: zero_filler_path.to_owned(),
            read_write_capability: ReadWriteCapability::ReadOnly as i32,
        });
    }

    Ok(component_disks)
}

/// Create a composite disk image compatible with crosvm's composite disk reader.
pub fn create_composite_disk(
    partitions: &[PartitionInfo],
    zero_filler_path: &Path,
    header_path: &Path,
    header_file: &mut File,
    footer_path: &Path,
    footer_file: &mut File,
    output_composite: &mut File,
) -> io::Result<()> {
    let zero_filler_path = zero_filler_path
        .to_str()
        .ok_or_else(|| io_err(format!("invalid zero filler path {:?}", zero_filler_path)))?
        .to_string();
    let header_path = header_path
        .to_str()
        .ok_or_else(|| io_err(format!("invalid header path {:?}", header_path)))?
        .to_string();
    let footer_path = footer_path
        .to_str()
        .ok_or_else(|| io_err(format!("invalid footer path {:?}", footer_path)))?
        .to_string();

    let mut composite_proto = CompositeDisk {
        version: COMPOSITE_DISK_VERSION,
        component_disks: vec![ComponentDisk {
            file_path: header_path,
            offset: 0,
            read_write_capability: ReadWriteCapability::ReadOnly as i32,
        }],
        length: 0,
    };

    let mut partitions_buffer =
        [0u8; GPT_NUM_PARTITIONS as usize * GPT_PARTITION_ENTRY_SIZE as usize];
    let mut writer: &mut [u8] = &mut partitions_buffer;
    let mut next_disk_offset = GPT_BEGINNING_SIZE;
    let mut labels = HashSet::with_capacity(partitions.len());

    for partition in partitions {
        let gpt_entry = create_gpt_entry(partition, next_disk_offset);
        if !labels.insert(gpt_entry.partition_name) {
            return Err(io_err(format!("duplicate GPT partition label {:?}", partition.label)));
        }
        gpt_entry
            .write_bytes(&mut writer)
            .map_err(|e| io_err(e.to_string()))?;

        for component_disk in create_component_disks(partition, next_disk_offset, &zero_filler_path)? {
            composite_proto.component_disks.push(component_disk);
        }

        next_disk_offset += partition.aligned_size();
    }

    let secondary_table_offset = next_disk_offset;
    let disk_size = secondary_table_offset + GPT_END_SIZE;

    composite_proto.component_disks.push(ComponentDisk {
        file_path: footer_path,
        offset: secondary_table_offset,
        read_write_capability: ReadWriteCapability::ReadOnly as i32,
    });

    let mut hasher = Hasher::new();
    hasher.update(&partitions_buffer);
    let partition_entries_crc32 = hasher.finalize();

    let disk_guid = Uuid::new_v4();
    write_beginning(
        header_file,
        disk_guid,
        &partitions_buffer,
        partition_entries_crc32,
        secondary_table_offset,
        disk_size,
    )?;
    write_end(
        footer_file,
        disk_guid,
        &partitions_buffer,
        partition_entries_crc32,
        secondary_table_offset,
        disk_size,
    )?;

    composite_proto.length = disk_size;
    output_composite.write_all(CDISK_MAGIC.as_bytes())?;
    let mut encoded = Vec::new();
    composite_proto
        .encode(&mut encoded)
        .map_err(|e| io_err(format!("encode CompositeDisk: {e}")))?;
    output_composite.write_all(&encoded)?;
    Ok(())
}
