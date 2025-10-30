pub mod entry;
pub mod hart;
pub mod mem;
pub mod power;
pub mod time;
pub mod trap;

use core::marker::PhantomData;

pub struct RiscvVirt;

pub struct Riscv64<C>(PhantomData<C>);
