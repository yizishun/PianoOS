pub mod entry;
pub mod mem;
pub mod power;
pub mod time;
pub mod trap;
pub mod harts;

use core::marker::PhantomData;

pub struct RiscvVirt;

pub struct Riscv64<C>(PhantomData<C>);

impl<C> Riscv64<C> {
    pub const fn new() -> Self {
	Riscv64::<C>(PhantomData)
    }
}
