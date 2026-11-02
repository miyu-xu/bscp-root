// Copyright 2025 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#[cfg(target_arch = "aarch64")]
use std::arch::asm;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use applevisor_sys::hv_error_t;
use applevisor_sys::hv_exit_reason_t;
use applevisor_sys::hv_reg_t;
use applevisor_sys::hv_return_t;
use applevisor_sys::hv_simd_fp_reg_t;
use applevisor_sys::hv_sys_reg_t;
use applevisor_sys::hv_vcpu_create;
use applevisor_sys::hv_vcpu_destroy;
use applevisor_sys::hv_vcpu_exit_t;
use applevisor_sys::hv_vcpu_get_reg;
use applevisor_sys::hv_vcpu_get_simd_fp_reg;
use applevisor_sys::hv_vcpu_get_sys_reg;
use applevisor_sys::hv_vcpu_run;
use applevisor_sys::hv_vcpu_set_reg;
use applevisor_sys::hv_vcpu_set_simd_fp_reg;
use applevisor_sys::hv_vcpu_set_sys_reg;
use applevisor_sys::hv_vcpu_t;
use applevisor_sys::hv_vcpus_exit;
use base::warn;
use base::Error;
use base::Event;
use base::Result;
use libc::EINVAL;
use libc::EIO;
use libc::ENOSYS;

use super::vm::check_hv;
use super::debug_registers::DebugRegisters;
use aarch64_sys_reg::AArch64SysRegId;
use crate::IoEventAddress;
use crate::IoOperation;
use crate::IoParams;
use crate::PsciVersion;
use crate::Vcpu;
use crate::VcpuAArch64;
use crate::VcpuExit;
use crate::VcpuFeature;
use crate::VcpuRegAArch64;
use crate::VcpuSignalHandle;
use crate::VcpuSignalHandleInner;
use crate::PSCI_0_2;

unsafe extern "C" {
    fn mach_absolute_time() -> u64;
    fn hv_vcpu_set_trap_debug_reg_accesses(vcpu: hv_vcpu_t, value: bool) -> hv_return_t;
    fn hv_vcpu_get_trap_debug_reg_accesses(vcpu: hv_vcpu_t, value: *mut bool) -> hv_return_t;
    // Apple's SDK defines hv_sys_reg_t as OS_ENUM(..., uint16_t). The Rust
    // dependency enum is not exhaustive: arbitrary architectural IDs must not
    // be transmuted into it. Keep the integer ABI for dynamic register access.
    #[link_name = "hv_vcpu_get_sys_reg"]
    fn hv_get_sys_reg_id(vcpu: hv_vcpu_t, reg: u16, value: *mut u64) -> hv_return_t;
    #[link_name = "hv_vcpu_set_sys_reg"]
    fn hv_set_sys_reg_id(vcpu: hv_vcpu_t, reg: u16, value: u64) -> hv_return_t;
}

/// Pending guest data abort (MMIO) exit state for `handle_mmio`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct MmioPending {
    pub gpa: u64,
    pub size: usize,
    pub write: bool,
    pub rt: u8,
    pub reg_64: bool,
    pub sign_extend: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum VcpuPowerState {
    On,
    Off,
    OnPending { entry: u64, context: u64 },
}

#[derive(Default)]
pub(crate) struct VcpuPowerControl {
    states: Mutex<Vec<VcpuPowerState>>,
    changed: Condvar,
}

impl VcpuPowerControl {
    pub(crate) fn initialize(&self, count: usize) -> Result<()> {
        let mut states = self.states.lock().map_err(|_| Error::new(EIO))?;
        *states = (0..count)
            .map(|id| {
                if id == 0 {
                    VcpuPowerState::On
                } else {
                    VcpuPowerState::Off
                }
            })
            .collect();
        Ok(())
    }

    fn complete_on(&self, id: usize, entry: u64, context: u64, initialize: impl FnOnce() -> Result<()>) -> Result<()> {
        let expected=VcpuPowerState::OnPending { entry, context };
        if self.states.lock().map_err(|_| Error::new(EIO))?.get(id) != Some(&expected) {
            return Err(Error::new(EINVAL));
        }
        initialize()?;
        let mut states=self.states.lock().map_err(|_| Error::new(EIO))?;
        if states.get(id) != Some(&expected) { return Err(Error::new(EIO)); }
        states[id]=VcpuPowerState::On;
        Ok(())
    }

    fn register(&self, id: usize) -> Result<()> {
        let mut states = self.states.lock().map_err(|_| Error::new(EIO))?;
        if states.len() <= id {
            let old_len = states.len();
            states.resize(id + 1, VcpuPowerState::Off);
            if old_len == 0 {
                states[0] = VcpuPowerState::On;
            }
        }
        Ok(())
    }

    fn set_off(&self, id: usize) -> Result<()> {
        let mut states = self.states.lock().map_err(|_| Error::new(EIO))?;
        let state = states.get_mut(id).ok_or_else(|| Error::new(EINVAL))?;
        *state = VcpuPowerState::Off;
        Ok(())
    }
}

impl VcpuPowerControl {
    fn request_on(&self, id: usize, entry: u64, context: u64) -> Result<i64> {
        let mut states = self.states.lock().map_err(|_| Error::new(EIO))?;
        let state = match states.get_mut(id) {
            Some(state) => state,
            None => return Ok(PSCI_RET_INVALID_PARAMS),
        };
        let response = match *state {
            VcpuPowerState::On => PSCI_RET_ALREADY_ON,
            VcpuPowerState::OnPending { .. } => PSCI_RET_ON_PENDING,
            VcpuPowerState::Off => {
                *state = VcpuPowerState::OnPending { entry, context };
                self.changed.notify_all();
                PSCI_RET_SUCCESS
            }
        };
        Ok(response)
    }
}

impl VcpuPowerControl {
    fn affinity_info(&self, id: usize) -> Result<i64> {
        let states = self.states.lock().map_err(|_| Error::new(EIO))?;
        Ok(match states.get(id) {
            Some(VcpuPowerState::On) => PSCI_AFFINITY_LEVEL_ON,
            Some(VcpuPowerState::Off) => PSCI_AFFINITY_LEVEL_OFF,
            Some(VcpuPowerState::OnPending { .. }) => PSCI_AFFINITY_LEVEL_ON_PENDING,
            None => PSCI_RET_INVALID_PARAMS,
        })
    }
}

struct HvfVcpuSignalHandle {
    vcpu: hv_vcpu_t,
    immediate_exit: Arc<AtomicBool>,
}

impl VcpuSignalHandleInner for HvfVcpuSignalHandle {
    fn signal_immediate_exit(&self) {
        self.immediate_exit.store(true, Ordering::Release);
        let vcpu = self.vcpu;
        // SAFETY: Hypervisor.framework supports canceling a running vCPU
        // from another thread with hv_vcpus_exit.
        unsafe {
            let _ = hv_vcpus_exit(&vcpu, 1);
        }
    }
}

pub struct HvfVcpu {
    id: usize,
    vcpu: hv_vcpu_t,
    exit: *const hv_vcpu_exit_t,
    cntfrq: u64,
    summary: Option<ExitSummary>,
    pending_mmio: Mutex<Option<MmioPending>>,
    debug_registers: Mutex<DebugRegisters>,
    ioevents: Arc<Mutex<HashMap<IoEventAddress, Event>>>,
    power_control: Arc<VcpuPowerControl>,
    immediate_exit: Arc<AtomicBool>,
}

unsafe impl Send for HvfVcpu {}
unsafe impl Sync for HvfVcpu {}

static HVF_EXIT_LOG_COUNT: AtomicUsize = AtomicUsize::new(0);
static HVF_MMIO_LOG_COUNT: AtomicUsize = AtomicUsize::new(0);

const EC_WFX_TRAP: u64 = 0x01;
const EC_AA64_HVC: u64 = 0x16;
const EC_AA64_SMC: u64 = 0x17;
const EC_SYSTEMREGISTERTRAP: u64 = 0x18;

const TMR_CTL_ENABLE: u64 = 1 << 0;
const TMR_CTL_IMASK: u64 = 1 << 1;

const AA64PFR0_EL1_GIC3EN: u64 = 1 << 24;
const AA64PFR1_EL1_SMEMASK: u64 = 3 << 24;
const CNTV_CTL_EL0: AArch64SysRegId =
    AArch64SysRegId::new_unchecked(0b11, 0b011, 0b1110, 0b0011, 0b001);

const PSCI_VERSION: u32 = 0x8400_0000;
const PSCI_CPU_SUSPEND_64: u32 = 0xc400_0001;
const PSCI_CPU_OFF: u32 = 0x8400_0002;
const PSCI_CPU_ON: u32 = 0x8400_0003;
const PSCI_CPU_ON_64: u32 = 0xc400_0003;
const PSCI_AFFINITY_INFO: u32 = 0x8400_0004;
const PSCI_AFFINITY_INFO_64: u32 = 0xc400_0004;
const PSCI_MIGRATE_INFO_TYPE: u32 = 0x8400_0006;
const PSCI_SYSTEM_OFF: u32 = 0x8400_0008;
const PSCI_SYSTEM_RESET: u32 = 0x8400_0009;
const PSCI_FEATURES: u32 = 0x8400_000a;

const PSCI_RET_SUCCESS: i64 = 0;
const PSCI_RET_NOT_SUPPORTED: i64 = -1;
const PSCI_RET_INVALID_PARAMS: i64 = -2;
const PSCI_RET_ALREADY_ON: i64 = -4;
const PSCI_RET_ON_PENDING: i64 = -5;

const PSCI_AFFINITY_LEVEL_ON: i64 = 0;
const PSCI_AFFINITY_LEVEL_OFF: i64 = 1;
const PSCI_AFFINITY_LEVEL_ON_PENDING: i64 = 2;

struct ExitSummary {
    last_log: Instant,
    total: u64,
    mmio: u64,
    psci: u64,
    sysreg: u64,
    wfx: u64,
    vtimer: u64,
    other_exception: u64,
    canceled: u64,
    unknown: u64,
    last_pc: u64,
    last_syndrome: u64,
    last_gpa: u64,
}

impl HvfVcpu {
    pub fn new(
        id: usize,
        ioevents: Arc<Mutex<HashMap<IoEventAddress, Event>>>,
        power_control: Arc<VcpuPowerControl>,
    ) -> Result<Self> {
        let mut vcpu: hv_vcpu_t = 0;
        let mut exit: *const hv_vcpu_exit_t = std::ptr::null();
        let cntfrq = {
            let cntfrq: u64;
            unsafe { asm!("mrs {}, cntfrq_el0", out(reg) cntfrq) };
            cntfrq
        };
        check_hv(unsafe {
            hv_vcpu_create(
                &mut vcpu,
                &mut exit as *mut *const hv_vcpu_exit_t,
                std::ptr::null_mut(),
            )
        })?;
        power_control.register(id)?;
        check_hv(unsafe { hv_vcpu_set_sys_reg(vcpu, hv_sys_reg_t::MPIDR_EL1, id as u64) })?;
        Ok(HvfVcpu {
            id,
            vcpu,
            exit,
            cntfrq,
            summary: std::env::var_os("CROSVM_HVF_EXIT_SUMMARY").map(|_| ExitSummary {
                last_log: Instant::now(),
                total: 0,
                mmio: 0,
                psci: 0,
                sysreg: 0,
                wfx: 0,
                vtimer: 0,
                other_exception: 0,
                canceled: 0,
                unknown: 0,
                last_pc: 0,
                last_syndrome: 0,
                last_gpa: 0,
            }),
            pending_mmio: Mutex::new(None),
            debug_registers: Mutex::new(DebugRegisters::default()),
            ioevents,
            power_control,
            immediate_exit: Arc::new(AtomicBool::new(false)),
        })
    }

    fn wait_until_powered_on(&self) -> Result<bool> {
        let mut states = self
            .power_control
            .states
            .lock()
            .map_err(|_| Error::new(EIO))?;
        loop {
            if self.immediate_exit.load(Ordering::Acquire) {
                return Ok(false);
            }
            match states.get(self.id).copied() {
                Some(VcpuPowerState::On) => return Ok(true),
                Some(VcpuPowerState::OnPending { entry, context }) => {
                    drop(states);
                    self.power_control.complete_on(self.id, entry, context, || {
                        self.reset_debug_registers()?;
                        self.write_reg(hv_reg_t::PC, entry)?;
                        self.write_reg(hv_reg_t::X0, context)
                    })?;
                    return Ok(true);
                }
                Some(VcpuPowerState::Off) => {
                    let (next_states, _) = self
                        .power_control
                        .changed
                        .wait_timeout(states, Duration::from_millis(50))
                        .map_err(|_| Error::new(EIO))?;
                    states = next_states;
                }
                None => return Err(Error::new(EINVAL)),
            }
        }
    }

    fn q_reg(n: u8) -> Result<hv_simd_fp_reg_t> {
        if n > 31 {
            return Err(Error::new(EINVAL));
        }
        // SAFETY: `hv_simd_fp_reg_t` is a C enum with discriminants Q0..=Q31 == 0..=31.
        Ok(unsafe { std::mem::transmute::<u32, hv_simd_fp_reg_t>(n as u32) })
    }

    fn x_reg(n: u8) -> Result<hv_reg_t> {
        Ok(match n {
            0 => hv_reg_t::X0,
            1 => hv_reg_t::X1,
            2 => hv_reg_t::X2,
            3 => hv_reg_t::X3,
            4 => hv_reg_t::X4,
            5 => hv_reg_t::X5,
            6 => hv_reg_t::X6,
            7 => hv_reg_t::X7,
            8 => hv_reg_t::X8,
            9 => hv_reg_t::X9,
            10 => hv_reg_t::X10,
            11 => hv_reg_t::X11,
            12 => hv_reg_t::X12,
            13 => hv_reg_t::X13,
            14 => hv_reg_t::X14,
            15 => hv_reg_t::X15,
            16 => hv_reg_t::X16,
            17 => hv_reg_t::X17,
            18 => hv_reg_t::X18,
            19 => hv_reg_t::X19,
            20 => hv_reg_t::X20,
            21 => hv_reg_t::X21,
            22 => hv_reg_t::X22,
            23 => hv_reg_t::X23,
            24 => hv_reg_t::X24,
            25 => hv_reg_t::X25,
            26 => hv_reg_t::X26,
            27 => hv_reg_t::X27,
            28 => hv_reg_t::X28,
            29 => hv_reg_t::X29,
            30 => hv_reg_t::X30,
            _ => return Err(Error::new(EINVAL)),
        })
    }

    fn parse_mmio(ex: &applevisor_sys::hv_vcpu_exit_exception_t) -> Option<MmioPending> {
        let ec = ex.syndrome >> 26;
        // Data Abort from lower EL / same EL (taken as MMIO candidates).
        if ec != 0x24 && ec != 0x25 {
            return None;
        }
        let iss = ex.syndrome & 0x01ff_ffff;
        let wnr = (iss >> 6) & 1 != 0;
        let sas = (iss >> 22) & 3;
        let size = 1usize << sas;
        let rt = ((iss >> 16) & 0x1f) as u8;
        let reg_64 = ((iss >> 15) & 1) != 0;
        let sign_extend = ((iss >> 21) & 1) != 0;
        Some(MmioPending {
            gpa: ex.physical_address,
            size,
            write: wnr,
            rt,
            reg_64,
            sign_extend,
        })
    }

    fn read_reg(&self, reg: hv_reg_t) -> Result<u64> {
        let mut v = 0u64;
        check_hv(unsafe { hv_vcpu_get_reg(self.vcpu, reg, &mut v) })?;
        Ok(v)
    }

    fn write_reg(&self, reg: hv_reg_t, value: u64) -> Result<()> {
        check_hv(unsafe { hv_vcpu_set_reg(self.vcpu, reg, value) })
    }

    fn read_sys_reg(&self, reg: AArch64SysRegId) -> Result<u64> {
        let mut value = 0u64;
        check_hv(unsafe { hv_get_sys_reg_id(self.vcpu, Self::sys_reg(reg), &mut value) })?;
        Ok(value)
    }

    fn advance_pc(&self) -> Result<()> {
        let pc = self.read_reg(hv_reg_t::PC)?;
        self.write_reg(hv_reg_t::PC, pc.wrapping_add(4))
    }

    fn sys_reg(reg: AArch64SysRegId) -> u16 {
        reg.encoded()
    }

    fn trapped_sys_reg(syndrome: u64) -> AArch64SysRegId {
        let op0 = 2 + ((syndrome >> 20) & 0x1) as u8;
        let op1 = ((syndrome >> 14) & 0x7) as u8;
        let crn = ((syndrome >> 10) & 0xf) as u8;
        let crm = ((syndrome >> 1) & 0xf) as u8;
        let op2 = ((syndrome >> 17) & 0x7) as u8;
        AArch64SysRegId::new_unchecked(op0, op1, crn, crm, op2)
    }

    fn psci_ret(code: i64) -> u64 {
        code as u64
    }

    fn handle_psci_call(&self, exception_class: u64) -> Result<Option<VcpuExit>> {
        let function = self.read_reg(hv_reg_t::X0)? as u32;
        // HVF returns HVC at the following instruction, but SMC at the trap.
        if exception_class == EC_AA64_SMC { self.advance_pc()?; }
        else if exception_class != EC_AA64_HVC { return Err(Error::new(EINVAL)); }
        match function {
            PSCI_VERSION => {
                self.write_reg(hv_reg_t::X0, u64::from(PSCI_0_2.minor))?;
                Ok(None)
            }
            PSCI_FEATURES => {
                let requested = self.read_reg(hv_reg_t::X1)? as u32;
                let response = match requested {
                    PSCI_VERSION
                    | PSCI_CPU_SUSPEND_64
                    | PSCI_CPU_OFF
                    | PSCI_CPU_ON
                    | PSCI_CPU_ON_64
                    | PSCI_AFFINITY_INFO
                    | PSCI_AFFINITY_INFO_64
                    | PSCI_MIGRATE_INFO_TYPE
                    | PSCI_SYSTEM_OFF
                    | PSCI_SYSTEM_RESET
                    | PSCI_FEATURES => PSCI_RET_SUCCESS,
                    _ => PSCI_RET_NOT_SUPPORTED,
                };
                self.write_reg(hv_reg_t::X0, Self::psci_ret(response))?;
                Ok(None)
            }
            PSCI_MIGRATE_INFO_TYPE => {
                self.write_reg(hv_reg_t::X0, 2)?;
                Ok(None)
            }
            PSCI_CPU_OFF => {
                self.power_control.set_off(self.id)?;
                Ok(None)
            }
            PSCI_SYSTEM_OFF => {
                let pc = self.read_reg(hv_reg_t::PC)?;
                let lr = self.read_reg(hv_reg_t::X30)?;
                let x1 = self.read_reg(hv_reg_t::X1)?;
                let x2 = self.read_reg(hv_reg_t::X2)?;
                warn!(
                    "HVF PSCI shutdown function=0x{function:08x} pc=0x{pc:016x} lr=0x{lr:016x} x1=0x{x1:016x} x2=0x{x2:016x}"
                );
                Ok(Some(VcpuExit::SystemEventShutdown))
            }
            PSCI_SYSTEM_RESET => {
                let pc = self.read_reg(hv_reg_t::PC)?;
                let lr = self.read_reg(hv_reg_t::X30)?;
                let x1 = self.read_reg(hv_reg_t::X1)?;
                let x2 = self.read_reg(hv_reg_t::X2)?;
                warn!(
                    "HVF PSCI reset function=0x{function:08x} pc=0x{pc:016x} lr=0x{lr:016x} x1=0x{x1:016x} x2=0x{x2:016x}"
                );
                Ok(Some(VcpuExit::SystemEventReset))
            }
            PSCI_CPU_ON | PSCI_CPU_ON_64 => {
                let target = self.read_reg(hv_reg_t::X1)? as usize;
                let entry = self.read_reg(hv_reg_t::X2)?;
                let context = self.read_reg(hv_reg_t::X3)?;
                let response = self.power_control.request_on(target, entry, context)?;
                self.write_reg(hv_reg_t::X0, Self::psci_ret(response))?;
                Ok(None)
            }
            PSCI_AFFINITY_INFO | PSCI_AFFINITY_INFO_64 => {
                let target = self.read_reg(hv_reg_t::X1)? as usize;
                let response = self.power_control.affinity_info(target)?;
                self.write_reg(hv_reg_t::X0, Self::psci_ret(response))?;
                Ok(None)
            }
            PSCI_CPU_SUSPEND_64 => {
                self.write_reg(hv_reg_t::X0, Self::psci_ret(PSCI_RET_NOT_SUPPORTED))?;
                Ok(None)
            }
            _ => {
                warn!("HVF unhandled PSCI function 0x{function:08x}");
                self.write_reg(hv_reg_t::X0, Self::psci_ret(PSCI_RET_NOT_SUPPORTED))?;
                Ok(None)
            }
        }
    }

    fn reset_debug_registers(&self) -> Result<()> {
        self.debug_registers.lock().map_err(|_| Error::new(EIO))?
            .reset(|effective| self.sync_debug_mdscr(effective))
    }

    fn sync_debug_mdscr(&self, effective: u64) -> Result<()> {
        check_hv(unsafe { hv_vcpu_set_sys_reg(self.vcpu, hv_sys_reg_t::MDSCR_EL1, effective) })?;
        let mut observed = 0;
        check_hv(unsafe { hv_vcpu_get_sys_reg(self.vcpu, hv_sys_reg_t::MDSCR_EL1, &mut observed) })?;
        if observed != effective { return Err(Error::new(EIO)); }
        Ok(())
    }

    fn write_debug_reg(&self, reg: AArch64SysRegId, value: u64) -> Result<()> {
        self.debug_registers.lock().map_err(|_| Error::new(EIO))?
            .write(reg, value, |effective| self.sync_debug_mdscr(effective))
    }

    fn read_debug_reg(&self, reg: AArch64SysRegId) -> Result<u64> {
        self.debug_registers.lock().map_err(|_| Error::new(EIO))?.read(reg)
    }

    fn handle_system_register_trap(&self, syndrome: u64) -> Result<()> {
        let is_read = (syndrome & 1) != 0;
        let rt = ((syndrome >> 5) & 0x1f) as u8;
        let reg = Self::trapped_sys_reg(syndrome);
        let sys = Self::sys_reg(reg);
        if DebugRegisters::owns(reg) {
            // A trap from EL0 cannot acquire EL1 access through the emulator.
            if !matches!(self.read_reg(hv_reg_t::CPSR)? & 0xf, 4 | 5) {
                return Err(Error::new(EINVAL));
            }
            if is_read {
                let value = self.read_debug_reg(reg)?;
                if rt < 31 { self.write_reg(Self::x_reg(rt)?, value)?; }
            } else {
                let value = if rt < 31 { self.read_reg(Self::x_reg(rt)?)? } else { 0 };
                self.write_debug_reg(reg, value)?;
            }
            return self.advance_pc();
        }

        // An unknown or unsupported access is not a successful read-zero/write-ignore.
        // Leave PC unchanged on failure so the owning run loop reports failure.
        if is_read {
            let mut value = 0u64;
            check_hv(unsafe { hv_get_sys_reg_id(self.vcpu, sys, &mut value) })?;
            if rt < 31 { self.write_reg(Self::x_reg(rt)?, value)?; }
        } else {
            let value = if rt < 31 { self.read_reg(Self::x_reg(rt)?)? } else { 0 };
            check_hv(unsafe { hv_set_sys_reg_id(self.vcpu, sys, value) })?;
        }
        self.advance_pc()?;

        Ok(())
    }

    fn handle_wait_for_event_trap(&self) -> Result<()> {
        self.advance_pc()?;

        let ctl = self.read_sys_reg(CNTV_CTL_EL0)?;
        if (ctl & TMR_CTL_ENABLE) == 0 || (ctl & TMR_CTL_IMASK) != 0 {
            thread::sleep(Duration::from_millis(1));
            return Ok(());
        }

        let cval = self.read_sys_reg(aarch64_sys_reg::CNTV_CVAL_EL0)?;
        let now = unsafe { mach_absolute_time() };
        if now >= cval {
            thread::yield_now();
            return Ok(());
        }

        let ticks = u128::from(cval - now);
        let nanos = ticks.saturating_mul(1_000_000_000u128) / u128::from(self.cntfrq.max(1));
        let timeout = Duration::from_nanos(nanos.min(u64::MAX as u128) as u64);
        thread::sleep(timeout.min(Duration::from_millis(1)));
        Ok(())
    }

    fn record_exit(&mut self, exit: &hv_vcpu_exit_t, pc: u64) {
        let Some(summary) = self.summary.as_mut() else {
            return;
        };
        summary.total += 1;
        summary.last_pc = pc;
        match exit.reason {
            hv_exit_reason_t::EXCEPTION => {
                let ec = exit.exception.syndrome >> 26;
                summary.last_syndrome = exit.exception.syndrome;
                summary.last_gpa = exit.exception.physical_address;
                if ec == EC_AA64_HVC || ec == EC_AA64_SMC {
                    summary.psci += 1;
                } else if ec == EC_SYSTEMREGISTERTRAP {
                    summary.sysreg += 1;
                } else if ec == EC_WFX_TRAP {
                    summary.wfx += 1;
                } else if Self::parse_mmio(&exit.exception).is_some() {
                    summary.mmio += 1;
                } else {
                    summary.other_exception += 1;
                }
            }
            hv_exit_reason_t::VTIMER_ACTIVATED => {
                summary.vtimer += 1;
            }
            hv_exit_reason_t::CANCELED => {
                summary.canceled += 1;
            }
            hv_exit_reason_t::UNKNOWN => {
                summary.unknown += 1;
            }
        }

        if summary.last_log.elapsed() >= Duration::from_secs(1) {
            warn!(
                "HVF summary vcpu={} total={} mmio={} psci={} sysreg={} wfx={} vtimer={} other_exc={} canceled={} unknown={} last_pc=0x{:x} last_syndrome=0x{:x} last_gpa=0x{:x}",
                self.id,
                summary.total,
                summary.mmio,
                summary.psci,
                summary.sysreg,
                summary.wfx,
                summary.vtimer,
                summary.other_exception,
                summary.canceled,
                summary.unknown,
                summary.last_pc,
                summary.last_syndrome,
                summary.last_gpa,
            );
            summary.last_log = Instant::now();
        }
    }
}

impl Drop for HvfVcpu {
    fn drop(&mut self) {
        unsafe {
            let _ = hv_vcpu_destroy(self.vcpu);
        }
    }
}

impl Vcpu for HvfVcpu {
    fn try_clone(&self) -> Result<Self> {
        Err(Error::new(ENOSYS))
    }

    fn as_vcpu(&self) -> &dyn Vcpu {
        self
    }

    fn run(&mut self) -> Result<VcpuExit> {
        loop {
            if !self.wait_until_powered_on()? {
                return Ok(VcpuExit::Canceled);
            }
            let r = unsafe { hv_vcpu_run(self.vcpu) };
            if r != hv_error_t::HV_SUCCESS as hv_return_t {
                let mut pc = 0u64;
                let _ = check_hv(unsafe { hv_vcpu_get_reg(self.vcpu, hv_reg_t::PC, &mut pc) });
                let mut cpsr = 0u64;
                let _ = check_hv(unsafe { hv_vcpu_get_reg(self.vcpu, hv_reg_t::CPSR, &mut cpsr) });
                let mut esr = 0u64;
                let _ = check_hv(unsafe {
                    hv_vcpu_get_sys_reg(self.vcpu, hv_sys_reg_t::ESR_EL1, &mut esr)
                });
                let mut far = 0u64;
                let _ = check_hv(unsafe {
                    hv_vcpu_get_sys_reg(self.vcpu, hv_sys_reg_t::FAR_EL1, &mut far)
                });
                let mut x0 = 0u64;
                let _ = check_hv(unsafe { hv_vcpu_get_reg(self.vcpu, hv_reg_t::X0, &mut x0) });
                warn!(
                    "HVF hv_vcpu_run failed for vcpu {} with code {} pc=0x{:x} cpsr=0x{:x} x0=0x{:x} esr=0x{:x} far=0x{:x}",
                    self.id, r, pc, cpsr, x0, esr, far
                );
                return Err(Error::new(EIO));
            }
            // SAFETY: `exit` is owned by the hypervisor and valid until destroy.
            let exit = unsafe { &*self.exit };
            let mut pc = 0u64;
            let _ = check_hv(unsafe { hv_vcpu_get_reg(self.vcpu, hv_reg_t::PC, &mut pc) });
            self.record_exit(exit, pc);
            if self.summary.is_some() && HVF_EXIT_LOG_COUNT.fetch_add(1, Ordering::Relaxed) < 2048 {
                match exit.reason {
                    hv_exit_reason_t::EXCEPTION => warn!(
                        "HVF exit vcpu={} reason=EXCEPTION pc=0x{:x} syndrome=0x{:x} gpa=0x{:x} gva=0x{:x}",
                        self.id,
                        pc,
                        exit.exception.syndrome,
                        exit.exception.physical_address,
                        exit.exception.virtual_address
                    ),
                    hv_exit_reason_t::VTIMER_ACTIVATED => {
                        warn!("HVF exit vcpu={} reason=VTIMER_ACTIVATED pc=0x{:x}", self.id, pc)
                    }
                    hv_exit_reason_t::CANCELED => {
                        warn!("HVF exit vcpu={} reason=CANCELED pc=0x{:x}", self.id, pc)
                    }
                    hv_exit_reason_t::UNKNOWN => {
                        warn!("HVF exit vcpu={} reason=UNKNOWN pc=0x{:x}", self.id, pc)
                    }
                }
            }
            match exit.reason {
                hv_exit_reason_t::EXCEPTION => {
                    let ec = exit.exception.syndrome >> 26;
                    if ec == EC_WFX_TRAP {
                        self.handle_wait_for_event_trap()?;
                        continue;
                    }
                    if ec == EC_AA64_HVC || ec == EC_AA64_SMC {
                        if let Some(vcpu_exit) = self.handle_psci_call(ec)? {
                            return Ok(vcpu_exit);
                        }
                        continue;
                    }
                    if ec == EC_SYSTEMREGISTERTRAP {
                        self.handle_system_register_trap(exit.exception.syndrome)?;
                        continue;
                    }
                    if let Some(mmio) = Self::parse_mmio(&exit.exception) {
                        *self.pending_mmio.lock().unwrap() = Some(mmio);
                        return Ok(VcpuExit::Mmio);
                    }
                    return Ok(VcpuExit::Exception);
                }
                hv_exit_reason_t::VTIMER_ACTIVATED => return Ok(VcpuExit::Intr),
                hv_exit_reason_t::CANCELED => return Ok(VcpuExit::Canceled),
                hv_exit_reason_t::UNKNOWN => return Ok(VcpuExit::InternalError),
            }
        }
    }

    fn id(&self) -> usize {
        self.id
    }

    fn set_immediate_exit(&self, exit: bool) {
        self.immediate_exit.store(exit, Ordering::Release);
        if exit {
            self.power_control.changed.notify_all();
            let v = self.vcpu;
            unsafe {
                let _ = hv_vcpus_exit(&v, 1);
            }
        }
    }

    fn signal_handle(&self) -> VcpuSignalHandle {
        VcpuSignalHandle {
            inner: Box::new(HvfVcpuSignalHandle {
                vcpu: self.vcpu,
                immediate_exit: self.immediate_exit.clone(),
            }),
        }
    }

    fn handle_mmio(
        &self,
        handle_fn: &mut dyn FnMut(IoParams) -> Result<()>,
    ) -> Result<()> {
        let mmio = self
            .pending_mmio
            .lock()
            .unwrap()
            .take()
            .ok_or_else(|| Error::new(EINVAL))?;

        let read_rt_u64 = |rt: u8| -> Result<u64> {
            let mut v = 0u64;
            check_hv(unsafe { hv_vcpu_get_reg(self.vcpu, Self::x_reg(rt)?, &mut v) })?;
            Ok(v)
        };

        let write_rt_u64 = |rt: u8, val: u64| -> Result<()> {
            check_hv(unsafe { hv_vcpu_set_reg(self.vcpu, Self::x_reg(rt)?, val) })
        };

        if !matches!(mmio.size, 1 | 2 | 4 | 8) {
            return Err(Error::new(EINVAL));
        }
        let mut data = [0u8; 8];
        if mmio.write {
            let value = if mmio.rt < 31 { read_rt_u64(mmio.rt)? } else { 0 };
            data.copy_from_slice(&value.to_le_bytes());
            handle_fn(IoParams { address: mmio.gpa,
                operation: IoOperation::Write(&data[..mmio.size]) })?;
        } else {
            handle_fn(IoParams { address: mmio.gpa,
                operation: IoOperation::Read(&mut data[..mmio.size]) })?;
            let mut val = u64::from_le_bytes(data);
            if mmio.size < 8 && mmio.sign_extend {
                let bits = (mmio.size * 8) as u32;
                val = ((val << (64 - bits)) as i64 >> (64 - bits)) as u64;
            }
            if !mmio.reg_64 { val &= u32::MAX as u64; }
            if mmio.rt != 31 { write_rt_u64(mmio.rt, val)?; }
        }
        if self.summary.is_some() && HVF_MMIO_LOG_COUNT.fetch_add(1, Ordering::Relaxed) < 2048 {
            warn!("HVF mmio vcpu={} addr=0x{:x} size={} write={} rt={} data={:02x?}",
                self.id, mmio.gpa, mmio.size, mmio.write, mmio.rt, &data[..mmio.size]);
        }

        // Advance PC past faulting instruction (AArch64: 4 bytes).
        let mut pc = 0u64;
        check_hv(unsafe { hv_vcpu_get_reg(self.vcpu, hv_reg_t::PC, &mut pc) })?;
        check_hv(unsafe { hv_vcpu_set_reg(self.vcpu, hv_reg_t::PC, pc + 4) })?;

        let addr = IoEventAddress::Mmio(mmio.gpa);
        let map = self.ioevents.lock().unwrap();
        if let Some(evt) = map.get(&addr) {
            evt.signal()?;
        }

        Ok(())
    }

    fn handle_io(&self, _handle_fn: &mut dyn FnMut(IoParams)) -> Result<()> {
        Ok(())
    }

    fn on_suspend(&self) -> Result<()> {
        Ok(())
    }

    unsafe fn enable_raw_capability(&self, _cap: u32, _args: &[u64; 4]) -> Result<()> {
        Err(Error::new(ENOSYS))
    }
}

impl VcpuAArch64 for HvfVcpu {
    fn init(&self, features: &[VcpuFeature]) -> Result<()> {
        for f in features {
            match f {
                VcpuFeature::PsciV0_2 => {}
                VcpuFeature::PmuV3 | VcpuFeature::Sve => return Err(Error::new(ENOSYS)),
                VcpuFeature::PowerOff => self.power_control.set_off(self.id)?,
            }
        }

        // Every Guest MDSCR/debug-register access must be intercepted before
        // software lock state can safely control the native effective value.
        check_hv(unsafe { hv_vcpu_set_trap_debug_reg_accesses(self.vcpu, true) })?;
        let mut trapped = false;
        check_hv(unsafe { hv_vcpu_get_trap_debug_reg_accesses(self.vcpu, &mut trapped) })?;
        if !trapped { return Err(Error::new(EIO)); }
        let mut dfr0 = 0;
        check_hv(unsafe { hv_vcpu_get_sys_reg(self.vcpu, hv_sys_reg_t::ID_AA64DFR0_EL1, &mut dfr0) })?;
        // The currently supported platform advertises Debugv8 and DoubleLock.
        // Never silently change that advertisement to avoid emulation.
        if dfr0 & 0xf != 6 || (dfr0 >> 36) & 0xf != 0 { return Err(Error::new(ENOSYS)); }
        self.reset_debug_registers()?;

        // The VM is created without nested EL2. Hypervisor.framework owns the
        // host EL2 controls; accessing HCR_EL2/CNTHCTL_EL2 here is unsupported.
        // Preserve the framework's advertised exception levels instead of
        // fabricating Guest EL2 support. All required feature accesses fail closed.
        configure_el1_features(
            |reg| {
                let mut value = 0;
                check_hv(unsafe { hv_vcpu_get_sys_reg(self.vcpu, reg, &mut value) })?;
                Ok(value)
            },
            |reg, value| check_hv(unsafe { hv_vcpu_set_sys_reg(self.vcpu, reg, value) }),
        )?;

        Ok(())
    }

    fn init_pmu(&self, _irq: u64) -> Result<()> {
        Err(Error::new(ENOSYS))
    }

    fn has_pvtime_support(&self) -> bool {
        false
    }

    fn init_pvtime(&self, _pvtime_ipa: u64) -> Result<()> {
        Err(Error::new(ENOSYS))
    }

    fn set_one_reg(&self, reg_id: VcpuRegAArch64, data: u64) -> Result<()> {
        match reg_id {
            VcpuRegAArch64::X(31) => Ok(()),
            VcpuRegAArch64::X(n) => {
                let r = Self::x_reg(n)?;
                check_hv(unsafe { hv_vcpu_set_reg(self.vcpu, r, data) })
            }
            VcpuRegAArch64::Sp => {
                check_hv(unsafe { hv_vcpu_set_sys_reg(self.vcpu, hv_sys_reg_t::SP_EL1, data) })
            }
            VcpuRegAArch64::Pc => {
                check_hv(unsafe { hv_vcpu_set_reg(self.vcpu, hv_reg_t::PC, data) })
            }
            VcpuRegAArch64::Pstate => {
                check_hv(unsafe { hv_vcpu_set_reg(self.vcpu, hv_reg_t::CPSR, data) })
            }
            VcpuRegAArch64::System(id) => {
                if DebugRegisters::owns(id) { return self.write_debug_reg(id, data); }
                let sys = Self::sys_reg(id);
                check_hv(unsafe { hv_set_sys_reg_id(self.vcpu, sys, data) })
            }
        }
    }

    fn get_one_reg(&self, reg_id: VcpuRegAArch64) -> Result<u64> {
        let mut v = 0u64;
        match reg_id {
            VcpuRegAArch64::X(31) => return Ok(0),
            VcpuRegAArch64::X(n) => {
                let r = Self::x_reg(n)?;
                check_hv(unsafe { hv_vcpu_get_reg(self.vcpu, r, &mut v) })?;
            }
            VcpuRegAArch64::Sp => {
                check_hv(unsafe { hv_vcpu_get_sys_reg(self.vcpu, hv_sys_reg_t::SP_EL1, &mut v) })?;
            }
            VcpuRegAArch64::Pc => {
                check_hv(unsafe { hv_vcpu_get_reg(self.vcpu, hv_reg_t::PC, &mut v) })?;
            }
            VcpuRegAArch64::Pstate => {
                check_hv(unsafe { hv_vcpu_get_reg(self.vcpu, hv_reg_t::CPSR, &mut v) })?;
            }
            VcpuRegAArch64::System(id) => {
                if DebugRegisters::owns(id) { return self.read_debug_reg(id); }
                let sys = Self::sys_reg(id);
                check_hv(unsafe { hv_get_sys_reg_id(self.vcpu, sys, &mut v) })?;
            }
        }
        Ok(v)
    }

    fn set_vector_reg(&self, reg_num: u8, data: u128) -> Result<()> {
        let reg = Self::q_reg(reg_num)?;
        check_hv(unsafe { hv_vcpu_set_simd_fp_reg(self.vcpu, reg, data) })
    }

    fn get_vector_reg(&self, reg_num: u8) -> Result<u128> {
        let reg = Self::q_reg(reg_num)?;
        let mut v: applevisor_sys::hv_simd_fp_uchar16_t = 0;
        check_hv(unsafe { hv_vcpu_get_simd_fp_reg(self.vcpu, reg, &mut v) })?;
        Ok(v)
    }

    fn get_system_regs(&self) -> Result<BTreeMap<AArch64SysRegId, u64>> {
        Ok(BTreeMap::new())
    }

    fn hypervisor_specific_snapshot(&self) -> anyhow::Result<snapshot::AnySnapshot> {
        anyhow::bail!("HVF snapshot is not supported")
    }

    fn hypervisor_specific_restore(&self, _data: snapshot::AnySnapshot) -> anyhow::Result<()> {
        anyhow::bail!("HVF restore is not supported")
    }

    fn get_psci_version(&self) -> Result<PsciVersion> {
        Ok(PSCI_0_2)
    }

    fn set_guest_debug(
        &self,
        _addrs: &[vm_memory::GuestAddress],
        _enable_singlestep: bool,
    ) -> Result<()> {
        Err(Error::new(ENOSYS))
    }

    fn get_max_hw_bps(&self) -> Result<usize> {
        Ok(0)
    }

    fn get_cache_info(&self) -> Result<BTreeMap<u8, u64>> {
        Ok(BTreeMap::new())
    }

    fn set_cache_info(&self, _cache_info: BTreeMap<u8, u64>) -> Result<()> {
        Ok(())
    }
}

fn configure_el1_features(
    mut get: impl FnMut(hv_sys_reg_t) -> Result<u64>,
    mut set: impl FnMut(hv_sys_reg_t, u64) -> Result<()>,
) -> Result<()> {
    let pfr0 = get(hv_sys_reg_t::ID_AA64PFR0_EL1)?;
    set(hv_sys_reg_t::ID_AA64PFR0_EL1, pfr0 | AA64PFR0_EL1_GIC3EN)?;
    let pfr1 = get(hv_sys_reg_t::ID_AA64PFR1_EL1)?;
    set(hv_sys_reg_t::ID_AA64PFR1_EL1, pfr1 & !AA64PFR1_EL1_SMEMASK)
}

#[cfg(test)]
mod el1_contract_tests {
    use super::*;
    #[test]
    fn preserves_exception_levels_and_only_updates_supported_features() {
        let mut writes = Vec::new();
        configure_el1_features(|reg| {
            match reg {
                hv_sys_reg_t::ID_AA64PFR0_EL1 => Ok(0x1101000010110011),
                hv_sys_reg_t::ID_AA64PFR1_EL1 => Ok(0x03000020),
                _ => panic!("unexpected register"),
            }
        }, |reg,value| { writes.push((reg,value)); Ok(()) }).unwrap();
        assert_eq!(writes.len(),2);
        assert_eq!(writes[0].1,0x1101000010110011 | (1 << 24));
        assert_eq!((writes[0].1 >> 8) & 15,0);
        assert_eq!(writes[1].1,0x20);
    }
    #[test]
    fn every_failed_access_stops_without_followup() {
        for fail in 0..4 {
            let calls=std::cell::Cell::new(0);
            let next=|| {let n=calls.get(); calls.set(n+1); if n==fail {Err(Error::new(EIO))} else {Ok(())}};
            assert!(configure_el1_features(|_| {next()?; Ok(0)}, |_,_| next()).is_err());
            assert_eq!(calls.get(),fail+1);
        }
    }
}

#[cfg(test)]
mod sysreg_abi_tests {
    use super::*;
    #[test]
    fn all_architectural_ids_are_integer_encoded_without_enum_construction() {
        for id in 0..=u16::MAX {
            assert_eq!(HvfVcpu::sys_reg(AArch64SysRegId::from_encoded(id)), id);
        }
        assert_eq!(HvfVcpu::sys_reg(AArch64SysRegId::new_unchecked(2,0,1,3,4)),0x809c);
        assert_eq!(HvfVcpu::sys_reg(CNTV_CTL_EL0),0xdf19);
    }
}

#[cfg(test)]
mod debug_native_contract_tests {
    use super::*;
    use applevisor_sys::{hv_vm_create,hv_vm_destroy,hv_vm_map,hv_vm_unmap};
    struct EmptyVm;
    impl Drop for EmptyVm { fn drop(&mut self) { assert_eq!(unsafe {hv_vm_destroy()},0); } }
    struct Code(*mut std::ffi::c_void);
    impl Drop for Code { fn drop(&mut self) { assert_eq!(unsafe {hv_vm_unmap(0x10000,16384)},0);unsafe{libc::free(self.0)}; } }
    fn msr(reg: AArch64SysRegId,rt:u32)->u32 {0xd5000000|((reg.encoded() as u32)<<5)|rt}
    fn mrs(reg: AArch64SysRegId,rt:u32)->u32 {0xd5200000|((reg.encoded() as u32)<<5)|rt}
    #[test]
    #[ignore = "requires explicit signed native HVF mechanism run"]
    fn real_traps_use_production_lock_state_and_hvc_smc_pc_contract() {
        use aarch64_sys_reg::{OSLAR_EL1,OSLSR_EL1,OSDLR_EL1,DBGPRCR_EL1,MDSCR_EL1};
        assert_eq!(unsafe {hv_vm_create(std::ptr::null_mut())},0);let vm=EmptyVm;
        let mut memory=std::ptr::null_mut();assert_eq!(unsafe{libc::posix_memalign(&mut memory,16384,16384)},0);
        let instructions=[msr(OSLAR_EL1,0),msr(MDSCR_EL1,2),mrs(MDSCR_EL1,1),msr(OSDLR_EL1,0),msr(OSLAR_EL1,3),msr(DBGPRCR_EL1,0),0xd4000002,0xd4000003,msr(OSLSR_EL1,0),0xd503201f,0xd4000002];
        unsafe{std::ptr::write_bytes(memory,0,16384);std::ptr::copy_nonoverlapping(instructions.as_ptr(),memory.cast::<u32>(),instructions.len())};
        assert_eq!(unsafe{hv_vm_map(memory,0x10000,16384,5)},0);let code=Code(memory);
        let cpu=HvfVcpu::new(0,Arc::new(Mutex::new(HashMap::new())),Arc::new(VcpuPowerControl::default())).unwrap();
        cpu.init(&[VcpuFeature::PsciV0_2]).unwrap();
        cpu.write_reg(hv_reg_t::PC,0x10000).unwrap();cpu.write_reg(hv_reg_t::CPSR,0x3c5).unwrap();
        cpu.write_reg(hv_reg_t::X0,1).unwrap();cpu.write_reg(hv_reg_t::X2,0xa000).unwrap();cpu.write_reg(hv_reg_t::X3,0).unwrap();
        for i in 0..6 {
            check_hv(unsafe{hv_vcpu_run(cpu.vcpu)}).unwrap();
            let syndrome=unsafe{(*cpu.exit).exception.syndrome};assert_eq!(syndrome>>26,EC_SYSTEMREGISTERTRAP);
            assert_eq!(cpu.read_reg(hv_reg_t::PC).unwrap(),0x10000+4*i);
            cpu.handle_system_register_trap(syndrome).unwrap();
            assert_eq!(cpu.read_reg(hv_reg_t::PC).unwrap(),0x10004+4*i);
            let mut effective=0;check_hv(unsafe{hv_vcpu_get_sys_reg(cpu.vcpu,hv_sys_reg_t::MDSCR_EL1,&mut effective)}).unwrap();
            assert_eq!(effective,if i==5 {0xa000}else{0});
            if i==2 {assert_eq!(cpu.read_reg(hv_reg_t::X1).unwrap(),0xa000);}
        }
        // Real HVC returns at PC+4; real SMC returns at its trapping instruction.
        for (pc,ec,returned_pc) in [(0x10018,EC_AA64_HVC,0x1001c),(0x1001c,EC_AA64_SMC,0x1001c)] {
            cpu.write_reg(hv_reg_t::X0,PSCI_VERSION as u64).unwrap();cpu.write_reg(hv_reg_t::PC,pc).unwrap();
            check_hv(unsafe{hv_vcpu_run(cpu.vcpu)}).unwrap();assert_eq!(unsafe{(*cpu.exit).exception.syndrome}>>26,ec);
            assert_eq!(cpu.read_reg(hv_reg_t::PC).unwrap(),returned_pc);
            assert!(cpu.handle_psci_call(ec).unwrap().is_none());assert_eq!(cpu.read_reg(hv_reg_t::PC).unwrap(),pc+4);
        }
        check_hv(unsafe{hv_vcpu_run(cpu.vcpu)}).unwrap();let syndrome=unsafe{(*cpu.exit).exception.syndrome};
        // A write to architecturally read-only OSLSR is UNDEFINED, not an EL2 sysreg trap.
        // With an intentionally unmapped EL1 vector, the next exit is its instruction abort.
        assert_eq!(syndrome,0x82000007);assert_eq!(cpu.read_reg(hv_reg_t::PC).unwrap(),0x200);
        let mut esr=0;let mut elr=0;
        check_hv(unsafe{hv_vcpu_get_sys_reg(cpu.vcpu,hv_sys_reg_t::ESR_EL1,&mut esr)}).unwrap();
        check_hv(unsafe{hv_vcpu_get_sys_reg(cpu.vcpu,hv_sys_reg_t::ELR_EL1,&mut elr)}).unwrap();
        assert_eq!(esr,0x02000000);assert_eq!(elr,0x10020);
        let state=*cpu.debug_registers.lock().unwrap();assert!(cpu.write_debug_reg(OSLSR_EL1,0).is_err());
        assert_eq!(*cpu.debug_registers.lock().unwrap(),state);
        cpu.power_control.set_off(0).unwrap();
        assert_eq!(cpu.power_control.request_on(0,0x10000,0x1234).unwrap(),PSCI_RET_SUCCESS);
        assert!(cpu.wait_until_powered_on().unwrap());
        assert_eq!(cpu.read_debug_reg(OSDLR_EL1).unwrap(),0);
        assert_eq!(cpu.read_debug_reg(MDSCR_EL1).unwrap(),0);
        assert_eq!(cpu.read_reg(hv_reg_t::X0).unwrap(),0x1234);
        assert_eq!(cpu.read_reg(hv_reg_t::PC).unwrap(),0x10000);
        let mut effective=1;check_hv(unsafe{hv_vcpu_get_sys_reg(cpu.vcpu,hv_sys_reg_t::MDSCR_EL1,&mut effective)}).unwrap();assert_eq!(effective,0);
        check_hv(unsafe{applevisor_sys::hv_vcpu_set_trap_debug_exceptions(cpu.vcpu,true)}).unwrap();
        cpu.write_debug_reg(OSLAR_EL1,1).unwrap();cpu.write_debug_reg(MDSCR_EL1,0x2001).unwrap();
        cpu.write_reg(hv_reg_t::PC,0x10024).unwrap();cpu.write_reg(hv_reg_t::CPSR,0x2001c5).unwrap();
        check_hv(unsafe{hv_vcpu_run(cpu.vcpu)}).unwrap();
        println!("LOCKED_STEP syndrome={:#x} pc={:#x}",unsafe{(*cpu.exit).exception.syndrome},cpu.read_reg(hv_reg_t::PC).unwrap());
        assert_eq!(unsafe{(*cpu.exit).exception.syndrome}>>26,EC_AA64_HVC);
        cpu.write_debug_reg(OSLAR_EL1,0).unwrap();
        cpu.write_reg(hv_reg_t::PC,0x10024).unwrap();cpu.write_reg(hv_reg_t::CPSR,0x2001c5).unwrap();
        check_hv(unsafe{hv_vcpu_run(cpu.vcpu)}).unwrap();
        println!("UNLOCKED_STEP syndrome={:#x} pc={:#x}",unsafe{(*cpu.exit).exception.syndrome},cpu.read_reg(hv_reg_t::PC).unwrap());
        assert_eq!(unsafe{(*cpu.exit).exception.syndrome}>>26,0x32);
        drop(cpu);drop(code);drop(vm);println!("PRODUCTION_NATIVE_DEBUG_TRAPS_AND_CLEANUP_VERIFIED");
    }
}

#[cfg(test)]
mod debug_reset_contract_tests {
    use super::*;
    #[test]
    fn cpu_on_failed_native_initialization_does_not_mark_on() {
        let p=VcpuPowerControl::default();p.initialize(1).unwrap();p.set_off(0).unwrap();
        assert_eq!(p.request_on(0,16,32).unwrap(),PSCI_RET_SUCCESS);
        assert!(p.complete_on(0,16,32,||Err(Error::new(EIO))).is_err());
        assert_eq!(p.affinity_info(0).unwrap(),PSCI_AFFINITY_LEVEL_ON_PENDING);
        assert!(p.complete_on(0,17,32,||panic!("identity mismatch invoked init")).is_err());
        p.complete_on(0,16,32,||Ok(())).unwrap();assert_eq!(p.affinity_info(0).unwrap(),PSCI_AFFINITY_LEVEL_ON);
    }
    #[test]
    fn failed_debug_reset_preserves_shadow() {
        let mut d=DebugRegisters::default();d.write(aarch64_sys_reg::OSDLR_EL1,1,|_|Ok(())).unwrap();
        let old=d;assert!(d.reset(|_|Err(Error::new(EIO))).is_err());assert_eq!(old,d);
        d.reset(|v|{assert_eq!(v,0);Ok(())}).unwrap();assert_eq!(d,DebugRegisters::default());
    }
}
