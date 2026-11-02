// Copyright 2026 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//! Guest OS debug locks. No external/halting debugger is exposed by this backend.
//! Unknown accesses and unsupported debug context-restore fields fail closed.
use aarch64_sys_reg::{AArch64SysRegId, DBGPRCR_EL1, MDSCR_EL1, OSDLR_EL1, OSLAR_EL1, OSLSR_EL1};
use base::{Error, Result};
use libc::{EINVAL, ENOTSUP};

pub(super) const ENABLE_BITS: u64 = (1 << 15) | (1 << 13) | 1;
// Monitor enable, kernel debug, software step, and trap DCC. Other nonzero
// context-restoration fields require an external-debug model we do not expose.
const SUPPORTED_MDSCR: u64 = ENABLE_BITS | (1 << 12);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct DebugRegisters {
    os_lock: bool,
    double_lock: bool,
    core_no_powerdown: bool,
    mdscr: u64,
}

impl DebugRegisters {
    pub(super) fn reset(&mut self, mut synchronize: impl FnMut(u64) -> Result<()>) -> Result<()> {
        let next=Self::default();
        synchronize(next.effective_mdscr())?;
        *self=next;
        Ok(())
    }
    pub(super) fn owns(reg: AArch64SysRegId) -> bool {
        [OSLAR_EL1, OSLSR_EL1, OSDLR_EL1, DBGPRCR_EL1, MDSCR_EL1].contains(&reg)
    }
    pub(super) fn inhibited(&self) -> bool {
        // A VM exit is not architectural Debug state. This backend has no
        // external debugger that can enter Debug state.
        self.os_lock || (self.double_lock && !self.core_no_powerdown)
    }
    pub(super) fn effective_mdscr(&self) -> u64 {
        if self.inhibited() { self.mdscr & !ENABLE_BITS } else { self.mdscr }
    }
    pub(super) fn read(&self, reg: AArch64SysRegId) -> Result<u64> {
        if reg == OSLSR_EL1 { Ok(8 | ((self.os_lock as u64) << 1)) }
        else if reg == OSDLR_EL1 { Ok(self.double_lock as u64) }
        else if reg == DBGPRCR_EL1 { Ok(self.core_no_powerdown as u64) }
        else if reg == MDSCR_EL1 { Ok(self.mdscr) }
        else { Err(Error::new(EINVAL)) } // OSLAR is write-only.
    }
    pub(super) fn write(
        &mut self, reg: AArch64SysRegId, value: u64,
        mut synchronize: impl FnMut(u64) -> Result<()>,
    ) -> Result<()> {
        let mut next = *self;
        if reg == OSLAR_EL1 { next.os_lock = value & 1 != 0; }
        else if reg == OSDLR_EL1 { next.double_lock = value & 1 != 0; }
        else if reg == DBGPRCR_EL1 { next.core_no_powerdown = value & 1 != 0; }
        else if reg == MDSCR_EL1 {
            if value & !SUPPORTED_MDSCR != 0 { return Err(Error::new(ENOTSUP)); }
            next.mdscr = value;
        } else { return Err(Error::new(EINVAL)); } // OSLSR is read-only.
        // Required before every state commit, including unlock. A failed native
        // write/readback stops the VM; it is never retried as a successful write.
        synchronize(next.effective_mdscr())?;
        *self = next;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lock_combinations_and_restore_guest_value() {
        for os in [false,true] { for dl in [false,true] { for np in [false,true] {
            let mut s=DebugRegisters::default();
            s.write(MDSCR_EL1,0xb001, |_| Ok(())).unwrap();
            s.write(OSLAR_EL1,os as u64, |_| Ok(())).unwrap();
            s.write(OSDLR_EL1,dl as u64, |_| Ok(())).unwrap();
            s.write(DBGPRCR_EL1,np as u64, |v| {
                assert_eq!(v,if os || (dl&&!np) {0x1000}else{0xb001}); Ok(())
            }).unwrap();
            assert_eq!(s.read(MDSCR_EL1).unwrap(),0xb001);
            assert_eq!(s.read(OSLSR_EL1).unwrap(),8 | ((os as u64)<<1));
        }}}
    }
    #[test]
    fn writes_while_locked_restore_on_unlock_and_cpus_are_independent() {
        let mut a=DebugRegisters::default();let b=DebugRegisters::default();
        a.write(OSLAR_EL1,1, |_| Ok(())).unwrap();
        a.write(MDSCR_EL1,0xa001, |v| {assert_eq!(v,0);Ok(())}).unwrap();
        a.write(OSLAR_EL1,0, |v| {assert_eq!(v,0xa001);Ok(())}).unwrap();
        assert_eq!(a.read(MDSCR_EL1).unwrap(),0xa001);
        assert_eq!(b.read(MDSCR_EL1).unwrap(),0);
        assert_eq!(b.read(OSDLR_EL1).unwrap(),0);
    }
    #[test]
    fn invalid_access_or_native_failure_does_not_commit() {
        let mut s=DebugRegisters::default();let old=s;
        assert!(s.read(OSLAR_EL1).is_err());
        for (r,v) in [(OSLSR_EL1,0),(MDSCR_EL1,1<<30)] {
            assert!(s.write(r,v, |_| panic!("invalid access reached native")).is_err());assert_eq!(s,old);
        }
        assert!(s.write(OSDLR_EL1,1, |_| Err(Error::new(libc::EIO))).is_err());assert_eq!(s,old);
        s.write(OSDLR_EL1,u64::MAX, |_| Ok(())).unwrap();assert_eq!(s.read(OSDLR_EL1).unwrap(),1);
    }
}
