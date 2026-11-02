use anyhow::{bail, Result};
use std::fs::File;

pub struct VbMetaImage;

impl VbMetaImage {
    pub fn verify_reader_region(_f: &File, _off: u64, _len: u64) -> Result<Self> {
        Ok(VbMetaImage)
    }

    pub fn descriptors(&self) -> Result<Vec<Descriptor>> {
        Ok(vec![])
    }
}

pub enum Descriptor {
    Hashtree(Hashtree),
}

pub struct Hashtree;

impl Descriptor {
    pub fn to_hashtree(&self) -> Result<&Hashtree> {
        match self {
            Descriptor::Hashtree(h) => Ok(h),
        }
    }
}

impl Hashtree {
    pub fn root_digest(&self) -> &[u8] {
        &[]
    }
}
