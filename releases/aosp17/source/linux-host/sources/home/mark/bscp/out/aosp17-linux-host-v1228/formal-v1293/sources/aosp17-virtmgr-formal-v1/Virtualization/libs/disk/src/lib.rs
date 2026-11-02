//! Composite disk helpers (host port; crosvm-compatible format).
//!
//! `create_composite_disk` writes the same `CDISK_MAGIC` + `CompositeDisk` protobuf layout and
//! GPT headers/footers as `external/crosvm/disk` (see `composite.rs` there).

mod composite;
mod gpt;

#[allow(clippy::all)]
mod cdisk_spec {
    include!(concat!(env!("OUT_DIR"), "/cdisk_spec.rs"));
}

use std::fs::File;
use std::io;
use std::path::Path;

use uuid::Uuid;

pub use composite::CDISK_MAGIC;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PartitionInfo {
    pub label: String,
    pub path: std::path::PathBuf,
    pub partition_type: ImagePartitionType,
    pub writable: bool,
    pub size: u64,
    pub part_guid: Option<Uuid>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ImagePartitionType {
    LinuxFilesystem,
    EfiSystemPartition,
}

/// Build a composite disk image: GPT header/footer files plus a spec file listing components.
pub fn create_composite_disk(
    partitions: &[PartitionInfo],
    zero_filler_path: &Path,
    header_path: &Path,
    header_file: &mut File,
    footer_path: &Path,
    footer_file: &mut File,
    output_composite: &mut File,
) -> io::Result<()> {
    composite::create_composite_disk(
        partitions,
        zero_filler_path,
        header_path,
        header_file,
        footer_path,
        footer_file,
        output_composite,
    )
}
