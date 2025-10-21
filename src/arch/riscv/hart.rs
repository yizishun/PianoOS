use core::arch::asm;

use log::error;
use sbi_rt::SbiRet;
use crate::{arch::HartInfo};

pub enum HartState {
    Started,
    Stoped,
    StartPeding,
    StopPeding,
    Invalid
}

impl core::fmt::Display for HartState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            HartState::Started => write!(f, "Hart is Started"),
            HartState::Stoped => write!(f, "Hart is Stoped"),
            HartState::StartPeding => write!(f, "Hart is Start pending"),
            HartState::StopPeding => write!(f, "Hart is Stop pending"),
            HartState::Invalid => write!(f, "Hart is Invalid"),
        }
    }
}

pub fn get_cur_hartid() -> usize {
    let hart_info_addr: usize;
    unsafe { 
        asm!("csrr {}, sscratch", out(reg) hart_info_addr);
    }
    let hart_info = hart_info_addr as *const HartInfo;
    unsafe {
        (*hart_info).hartid
    }
}

pub fn get_hartnum() -> usize {
     (0..).take_while(|h| {
        sbi_rt::hart_get_status(*h).error == SbiRet::success(0).error
    }).count()
    //PLATFORM.get().unwrap().board_info.cpu_num.unwrap()
}

pub fn get_cur_hart_state() -> HartState {
    use sbi_rt::hart_get_status;
    match hart_get_status(get_cur_hartid()).value {
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