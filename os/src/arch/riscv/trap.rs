use core::arch::naked_asm;

use riscv::register::sstatus::Sstatus;

macro_rules! save {
    ($reg:ident => $ptr:ident[$pos:expr]) => {
        concat!(
            "sd ",
            stringify!($reg),
            ", 8*",
            $pos,
            '(',
            stringify!($ptr),
            ')'
        )
    };
}

macro_rules! load {
    ($ptr:ident[$pos:expr] => $reg:ident) => {
        concat!(
            "ld ",
            stringify!($reg),
            ", 8*",
            $pos,
            '(',
            stringify!($ptr),
            ')'
        )
    };
}

#[repr(C)]
pub struct TrapContext {
    pub ra: usize,      // 0..
    pub t: [usize; 7],  // 1..
    pub a: [usize; 8],  // 8..
    pub s: [usize; 12], // 16..
    pub gp: usize,      // 28..
    pub tp: usize,      // 29..
    pub sp: usize,      // 30..
    pub pc: usize,      // 31..
    pub sstatus: Sstatus,
    pub sepc: usize
}

#[unsafe(naked)]
unsafe extern "C" fn all_trap() {
    naked_asm!("nop");
}