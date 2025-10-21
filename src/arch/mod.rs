use core::array::from_fn;

use crate::config::NUM_HART_MAX;

pub mod riscv;

pub static mut HART_INFO: [HartInfo; NUM_HART_MAX] = [HartInfo::ZERO_HART; NUM_HART_MAX];
const HART_INFO_SIZE: usize = size_of::<HartInfo>();

#[derive(Clone, Copy)]
pub struct HartInfo {
    pub hartid: usize
}

impl HartInfo {
    pub const ZERO_HART: HartInfo = HartInfo { hartid: 0 };
}