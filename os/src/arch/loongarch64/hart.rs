use core::arch::asm;

use log::error;
use crate::{arch::common::hart::{HartContext, HartState}};

pub fn get_cur_hart() -> &'static HartContext {
    todo!("get_cur_hart");
}

pub fn get_hart_state(id: usize) -> HartState {
    todo!("get_hart_state");
}