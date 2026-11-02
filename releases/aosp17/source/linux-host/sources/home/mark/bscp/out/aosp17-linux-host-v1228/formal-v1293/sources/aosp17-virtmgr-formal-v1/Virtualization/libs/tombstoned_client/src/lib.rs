use anyhow::{bail, Result};
use std::fs::File;

#[derive(Clone, Copy, Debug)]
pub enum DebuggerdDumpType {
    Tombstone,
}

pub struct TombstonedConnection {
    pub text_output: Option<File>,
}

impl TombstonedConnection {
    pub fn connect(_pid: i32, _t: DebuggerdDumpType) -> Result<Self> {
        bail!("tombstoned_client stub")
    }

    pub fn notify_completion(&self) -> Result<()> {
        Ok(())
    }
}
