pub mod entry;
pub mod mem;
pub mod power;
pub mod time;
pub mod trap;
pub mod harts;

use core::marker::PhantomData;

// common会使用设备树解析的结果
pub struct RiscvCommon;
pub struct RiscvVirt; //可以对某些platform做特殊的优化，比如硬编码某些东西

pub struct Riscv64<C>(PhantomData<C>);

impl<C> Riscv64<C> {
    	pub const fn new() -> Self {
		Riscv64::<C>(PhantomData)
    	}
}
