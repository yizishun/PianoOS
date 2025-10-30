use core::arch::asm;

use crate::arch::common::hart::{HartContext, HartState};
use log::error;

pub fn get_cur_hart() -> &'static HartContext {
        todo!("get_cur_hart");
}

pub fn get_hart_state(id: usize) -> HartState {
        todo!("get_hart_state");
}
