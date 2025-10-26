use core::arch::asm;

use log::error;
use crate::{arch::common::hart::{HartInfo, HartState}};

pub fn get_cur_hart() -> &'static HartInfo {
    todo!("get_cur_hart");
}

pub fn get_hart_state(id: usize) -> HartState {
    todo!("get_hart_state");
}