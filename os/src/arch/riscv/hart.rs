use core::arch::asm;

use log::error;
use crate::{arch::common::hart::{HartContext, HartState}};

pub fn get_cur_hart() -> &'static HartContext {
    let hart_info_addr: usize;
    unsafe { 
        asm!("csrr {}, sscratch", out(reg) hart_info_addr);
    }
    let hart_info = hart_info_addr as *const HartContext;
    unsafe {
        &(*hart_info)
    }
}

pub fn get_hart_state(id: usize) -> HartState {
    use sbi_rt::hart_get_status;
    match hart_get_status(id).value {
        0 => HartState::Started,
        1 => HartState::Stoped,
        2 => HartState::StartPeding,
        3 => HartState::StopPeding,
        _ => {
            error!("unexpected hart id, may be rustsbi problem"); 
            HartState::Invalid
        }
    }
}