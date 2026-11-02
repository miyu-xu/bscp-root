// Copyright 2021 The ChromiumOS Authors
// Copyright (C) 2026 The Android Open Source Project
//
// SPDX-License-Identifier: BSD-3-Clause
//
// Functions for writing GUID Partition Tables for use in a composite disk image.
// Ported from external/crosvm/disk/src/gpt.rs (minimal subset).

use std::convert::TryInto;
use std::io;
use std::io::Write;
use std::num::TryFromIntError;

use crc32fast::Hasher;
use thiserror::Error as ThisError;
use uuid::Uuid;

/// The size in bytes of a disk sector (also called a block).
pub const SECTOR_SIZE: u64 = 1 << 9;
const MBR_PARTITION_ENTRY_SIZE: usize = 16;
/// The size in bytes of a GPT header.
pub const GPT_HEADER_SIZE: u32 = 92;
/// The number of partition entries in the GPT.
pub const GPT_NUM_PARTITIONS: u32 = 128;
/// The size in bytes of a single GPT partition entry.
pub const GPT_PARTITION_ENTRY_SIZE: u32 = 128;
/// Everything before the first partition: MBR, GPT header and GPT partition entries.
pub const GPT_BEGINNING_SIZE: u64 = SECTOR_SIZE * 40;
/// Everything after the last partition: GPT partition entries and GPT footer.
pub const GPT_END_SIZE: u64 = SECTOR_SIZE * 33;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("invalid disk size: {0}")]
    InvalidDiskSize(TryFromIntError),
    #[error("failed to write data: {0}")]
    WritingData(io::Error),
}

pub fn write_protective_mbr(file: &mut impl Write, disk_size: u64) -> Result<(), Error> {
    file.write_all(&[0; 446]).map_err(Error::WritingData)?;
    file.write_all(&[0x00]).map_err(Error::WritingData)?;
    file.write_all(&[0; 3]).map_err(Error::WritingData)?;
    file.write_all(&[0xEE]).map_err(Error::WritingData)?;
    file.write_all(&[0; 3]).map_err(Error::WritingData)?;
    let first_lba: u32 = 1;
    file.write_all(&first_lba.to_le_bytes())
        .map_err(Error::WritingData)?;
    let number_of_sectors: u32 = (disk_size / SECTOR_SIZE)
        .try_into()
        .map_err(Error::InvalidDiskSize)?;
    file.write_all(&number_of_sectors.to_le_bytes())
        .map_err(Error::WritingData)?;
    file.write_all(&[0; MBR_PARTITION_ENTRY_SIZE * 3])
        .map_err(Error::WritingData)?;
    file.write_all(&[0x55, 0xAA]).map_err(Error::WritingData)?;
    Ok(())
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct GptHeader {
    signature: [u8; 8],
    revision: [u8; 4],
    header_size: u32,
    header_crc32: u32,
    current_lba: u64,
    backup_lba: u64,
    first_usable_lba: u64,
    last_usable_lba: u64,
    disk_guid: Uuid,
    partition_entries_lba: u64,
    num_partition_entries: u32,
    partition_entry_size: u32,
    partition_entries_crc32: u32,
}

impl GptHeader {
    fn write_bytes(&self, out: &mut impl Write) -> Result<(), Error> {
        out.write_all(&self.signature).map_err(Error::WritingData)?;
        out.write_all(&self.revision).map_err(Error::WritingData)?;
        out.write_all(&self.header_size.to_le_bytes())
            .map_err(Error::WritingData)?;
        out.write_all(&self.header_crc32.to_le_bytes())
            .map_err(Error::WritingData)?;
        out.write_all(&[0; 4]).map_err(Error::WritingData)?;
        out.write_all(&self.current_lba.to_le_bytes())
            .map_err(Error::WritingData)?;
        out.write_all(&self.backup_lba.to_le_bytes())
            .map_err(Error::WritingData)?;
        out.write_all(&self.first_usable_lba.to_le_bytes())
            .map_err(Error::WritingData)?;
        out.write_all(&self.last_usable_lba.to_le_bytes())
            .map_err(Error::WritingData)?;
        write_guid(out, self.disk_guid).map_err(Error::WritingData)?;
        out.write_all(&self.partition_entries_lba.to_le_bytes())
            .map_err(Error::WritingData)?;
        out.write_all(&self.num_partition_entries.to_le_bytes())
            .map_err(Error::WritingData)?;
        out.write_all(&self.partition_entry_size.to_le_bytes())
            .map_err(Error::WritingData)?;
        out.write_all(&self.partition_entries_crc32.to_le_bytes())
            .map_err(Error::WritingData)?;
        Ok(())
    }
}

pub fn write_gpt_header(
    out: &mut impl Write,
    disk_guid: Uuid,
    partition_entries_crc32: u32,
    secondary_table_offset: u64,
    secondary: bool,
) -> Result<(), Error> {
    let primary_header_lba = 1;
    let secondary_header_lba = (secondary_table_offset + GPT_END_SIZE) / SECTOR_SIZE - 1;
    let mut gpt_header = GptHeader {
        signature: *b"EFI PART",
        revision: [0, 0, 1, 0],
        header_size: GPT_HEADER_SIZE,
        current_lba: if secondary {
            secondary_header_lba
        } else {
            primary_header_lba
        },
        backup_lba: if secondary {
            primary_header_lba
        } else {
            secondary_header_lba
        },
        first_usable_lba: GPT_BEGINNING_SIZE / SECTOR_SIZE,
        last_usable_lba: secondary_table_offset / SECTOR_SIZE - 1,
        disk_guid,
        partition_entries_lba: 2,
        num_partition_entries: GPT_NUM_PARTITIONS,
        partition_entry_size: GPT_PARTITION_ENTRY_SIZE,
        partition_entries_crc32,
        header_crc32: 0,
    };

    let mut header_without_crc = [0u8; GPT_HEADER_SIZE as usize];
    gpt_header.write_bytes(&mut &mut header_without_crc[..])?;
    let mut hasher = Hasher::new();
    hasher.update(&header_without_crc);
    gpt_header.header_crc32 = hasher.finalize();

    gpt_header.write_bytes(out)?;

    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GptPartitionEntry {
    pub partition_type_guid: Uuid,
    pub unique_partition_guid: Uuid,
    pub first_lba: u64,
    pub last_lba: u64,
    pub attributes: u64,
    pub partition_name: [u16; 36],
}

impl Default for GptPartitionEntry {
    fn default() -> Self {
        Self {
            partition_type_guid: Default::default(),
            unique_partition_guid: Default::default(),
            first_lba: 0,
            last_lba: 0,
            attributes: 0,
            partition_name: [0; 36],
        }
    }
}

impl GptPartitionEntry {
    pub fn write_bytes(&self, out: &mut impl Write) -> Result<(), Error> {
        write_guid(out, self.partition_type_guid).map_err(Error::WritingData)?;
        write_guid(out, self.unique_partition_guid).map_err(Error::WritingData)?;
        out.write_all(&self.first_lba.to_le_bytes())
            .map_err(Error::WritingData)?;
        out.write_all(&self.last_lba.to_le_bytes())
            .map_err(Error::WritingData)?;
        out.write_all(&self.attributes.to_le_bytes())
            .map_err(Error::WritingData)?;
        for code_unit in &self.partition_name {
            out.write_all(&code_unit.to_le_bytes())
                .map_err(Error::WritingData)?;
        }
        Ok(())
    }
}

fn write_guid(out: &mut impl Write, guid: Uuid) -> Result<(), io::Error> {
    let guid_fields = guid.as_fields();
    out.write_all(&guid_fields.0.to_le_bytes())?;
    out.write_all(&guid_fields.1.to_le_bytes())?;
    out.write_all(&guid_fields.2.to_le_bytes())?;
    out.write_all(guid_fields.3)?;
    Ok(())
}
