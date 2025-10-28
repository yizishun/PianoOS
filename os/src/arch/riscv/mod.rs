use core::marker::PhantomData;

use crate::arch::common::ArchTime;

pub mod hart;
pub mod entry;
pub mod trap;
pub mod power;
pub mod isa;
pub mod time;

pub struct RiscvVirt;

pub struct Riscv64<C>(PhantomData<C>);
