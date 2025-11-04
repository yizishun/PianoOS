use core::arch::naked_asm;
use riscv::register::{sepc, sstatus::{self, SPP, Sstatus}, stvec::{self, Stvec}};
use log::warn;
use riscv::interrupt::supervisor::Exception;
use riscv::register::{scause, stval};
use riscv::register::mcause::Trap;

use crate::{arch::{common::ArchTrap, riscv::Riscv64}, harts::hart_context_in_trap_stage};
use crate::APP_MANAGER;
use crate::syscall::syscall;
use crate::trap::fast::FastResult;
use crate::trap::fast::FastContext;

impl<C> ArchTrap for Riscv64<C> {
	#[inline]
	unsafe fn load_direct_trap_entry(&self) {
		unsafe {
		    stvec::write(Stvec::new(trap_entry as usize, stvec::TrapMode::Direct));
		}
	}
}

macro_rules! exchange {
    () => {
	exchange!(sp)
    };

    ($reg:ident) => {
	concat!("csrrw ", stringify!($reg), ", sscratch, ", stringify!($reg))
    };
}

macro_rules! r#return {
    () => {
	"sret"
    };
}

macro_rules! save {
	($reg:ident => $ptr:ident[$pos:expr]) => {
		concat!("sd ",
			stringify!($reg),
			", 8*",
			$pos,
			'(',
			stringify!($ptr),
			')')
	};
}

macro_rules! load {
	($ptr:ident[$pos:expr] => $reg:ident) => {
		concat!("ld ",
			stringify!($reg),
			", 8*",
			$pos,
			'(',
			stringify!($ptr),
			')')
	};
}

#[repr(C)]
pub struct FlowContext {
	pub ra: usize,      // 0..
	pub t: [usize; 7],  // 1..
	pub a: [usize; 8],  // 8..
	pub s: [usize; 12], // 16..
	pub gp: usize,      // 28..
	pub tp: usize,      // 29..
	pub sp: usize,      // 30..
	pub pc: usize,      // 31..
	pub sstatus: Sstatus,
	pub sepc: usize,
}

#[unsafe(naked)]
pub unsafe extern "C" fn trap_entry() {
	core::arch::naked_asm!(
		".align 2",
		// 换栈
		exchange!(),
		// 加载上下文指针
		save!(a0 => sp[2]),
		load!(sp[0] => a0),
		// 保存尽量少的寄存器
		save!(ra => a0[0]),
		save!(t0 => a0[1]),
		save!(t1 => a0[2]),
		save!(t2 => a0[3]),
		save!(t3 => a0[4]),
		save!(t4 => a0[5]),
		save!(t5 => a0[6]),
		save!(t6 => a0[7]),
		save!(tp => a0[29]), //tp存放原sscratch值，在整个trap过程有效
		// 调用快速路径函数
		//
		// | reg    | position
		// | ------ | -
		// | ra     | `TrapHandler.context`
		// | t0-t6  | `TrapHandler.context`
		// | a0     | `TrapHandler.scratch`
		// | a1-a7  | 参数寄存器
		// | sp     | sscratch
		// | gp, tp | gp, tp
		// | s0-s11 | 不支持
		//
		// > 若要保留陷入上下文，
		// > 必须在快速路径保存 a0-a7 到 `TrapHandler.context`，
		// > 并进入完整路径执行后续操作。
		// >
		// > 若要切换上下文，在快速路径设置 gp/tp/sscratch/sepc 和 sstatus。
		"mv   a0, sp",
		"mv   tp, sp",
		load!(sp[1] => ra),
		"jalr ra",
		"0:", // 加载上下文指针
		load!(sp[0] => a1),
		// 0：设置少量参数寄存器
		"   beqz  a0, 0f",
		// 1：设置所有参数寄存器
		"   addi  a0, a0, -1
		beqz  a0, 1f
		",
		// 2：设置所有调用者寄存器
		"   addi  a0, a0, -1
		beqz  a0, 2f
		",
		// 3：设置所有寄存器
		"   addi  a0, a0, -1
		beqz  a0, 3f
		",
		// 4：完整路径
		save!(s0  => a1[16]),
		save!(s1  => a1[17]),
		save!(s2  => a1[18]),
		save!(s3  => a1[19]),
		save!(s4  => a1[20]),
		save!(s5  => a1[21]),
		save!(s6  => a1[22]),
		save!(s7  => a1[23]),
		save!(s8  => a1[24]),
		save!(s9  => a1[25]),
		save!(s10 => a1[26]),
		save!(s11 => a1[27]),
		// 调用完整路径函数
		//
		// | reg    | position
		// | ------ | -
		// | sp     | sscratch
		// | gp, tp | gp, tp
		// | else   | `TrapHandler.context`
		//
		// > 若要保留陷入上下文，
		// > 在完整路径中保存 gp/tp/sp/pc 到 `TrapHandler.context`。
		// >
		// > 若要切换上下文，在完整路径设置 gp/tp/sscratch/sepc 和 sstatus。
		"mv   a0, sp",
		load!(sp[2] => ra),
		"jalr ra",
		"j    0b",
		"3:", // 设置所有寄存器
		load!(a1[16] => s0),
		load!(a1[17] => s1),
		load!(a1[18] => s2),
		load!(a1[19] => s3),
		load!(a1[20] => s4),
		load!(a1[21] => s5),
		load!(a1[22] => s6),
		load!(a1[23] => s7),
		load!(a1[24] => s8),
		load!(a1[25] => s9),
		load!(a1[26] => s10),
		load!(a1[27] => s11),
		"2:", // 设置所有调用者寄存器
		load!(a1[ 0] => ra),
		load!(a1[ 1] => t0),
		load!(a1[ 2] => t1),
		load!(a1[ 3] => t2),
		load!(a1[ 4] => t3),
		load!(a1[ 5] => t4),
		load!(a1[ 6] => t5),
		load!(a1[ 7] => t6),
		load!(a1[29] => tp),
		"1:", // 设置所有参数寄存器
		load!(a1[10] => a2),
		load!(a1[11] => a3),
		load!(a1[12] => a4),
		load!(a1[13] => a5),
		load!(a1[14] => a6),
		load!(a1[15] => a7),
		"0:", // 设置少量参数寄存器
		load!(a1[ 8] => a0),
		load!(a1[ 9] => a1),
		exchange!(),
		r#return!(),
	)
}

pub extern "C" fn fast_handler(
    mut ctx: FastContext,
    a1: usize,
    a2: usize,
    a3: usize,
    a4: usize,
    a5: usize,
    a6: usize,
    a7: usize,
) -> FastResult {
    let save_regs = |ctx: &mut FastContext| {
	ctx.regs().a = [ctx.a0(), a1, a2, a3, a4, a5, a6, a7];
    };
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause()
	.try_into::<riscv::interrupt::Interrupt, riscv::interrupt::supervisor::Exception>()
	.unwrap() {

	Trap::Exception(Exception::UserEnvCall) => {
		save_regs(&mut ctx);
		unsafe {
			sepc::write(sepc::read() + 4);
			ctx.regs().a[0] = 
			syscall(a7.try_into().unwrap(), [ctx.a0(), a1, a2]) as usize
		}
		ctx.restore()
	}
	Trap::Exception(Exception::StoreFault) |
	Trap::Exception(Exception::StorePageFault) |
	Trap::Exception(Exception::LoadFault) | 
	Trap::Exception(Exception::LoadMisaligned) => {
		save_regs(&mut ctx);
		ctx.hart().app_info.end();
		warn!("PageFault in application, kernel killed it.");
		warn!("Illegal addr: 0x{:x}", stval);
		warn!("excption pc: 0x{:x}", sepc::read());
		APP_MANAGER.get().unwrap().run_next_app_in_trap();
		unsafe {
			sepc::write(crate::config::APP_BASE_ADDR);
		}
		ctx.restore()
	}
	Trap::Exception(Exception::IllegalInstruction) => {
		save_regs(&mut ctx);
		ctx.hart().app_info.end();
		warn!("IllegalInstruction in application, kernel killed it.");
		warn!("excption pc: 0x{:x}", sepc::read());
		APP_MANAGER.get().unwrap().run_next_app_in_trap();
		unsafe {
			sepc::write(crate::config::APP_BASE_ADDR);
		}
		ctx.restore()
	}
	Trap::Exception(Exception::InstructionFault) |
	Trap::Exception(Exception::InstructionMisaligned) |
	Trap::Exception(Exception::InstructionPageFault) => {
		save_regs(&mut ctx);
		ctx.hart().app_info.end();
		warn!("Instruction PageFault in application, kernel killed it.");
		warn!("Illegal addr: 0x{:x}", stval);
		warn!("excption pc: 0x{:x}", sepc::read());
		APP_MANAGER.get().unwrap().run_next_app_in_trap();
		unsafe {
			sepc::write(crate::config::APP_BASE_ADDR);
		}
		ctx.restore()
	}

	_ => {
	    	panic!("Unsupported trap {:?}, stval = {:#x}!", scause.cause(), stval);
	}
    }
}


#[unsafe(naked)]
pub unsafe extern "C" fn boot_entry() -> ! {
	naked_asm!(
		".align 2",
		// sscratch is set in load_as_stack in main
		"call {boot_handler}",
		"call {locate}",
		"sret",
		boot_handler = sym boot_handler,
		locate = sym locate_user_stack

	)
}

pub extern "C" fn boot_handler() {
	unsafe {
		sstatus::set_spp(SPP::User);
		sepc::write(crate::config::APP_BASE_ADDR);
	}
}

/// Locates and initializes user stack for each hart.
///
/// This is a naked function that sets up the stack pointer based on hart ID.
#[unsafe(naked)]
pub(crate) unsafe extern "C" fn locate_user_stack() {
    core::arch::naked_asm!(
	"   la   sp, {stack}            // Load stack base address
	    li   t0, {per_hart_stack_size} // Load stack size per hart
	    mv t1, a0                   // Get current hart ID
	    addi t1, t1,  1             // Add 1 to hart ID
	 1: add  sp, sp, t0             // Calculate stack pointer
	    addi t1, t1, -1             // Decrement counter
	    bnez t1, 1b                 // Loop if not zero
	    ret                         // Return
	",
	per_hart_stack_size = const crate::config::STACK_SIZE,
	stack               =   sym crate::USER_STACK,
    )
}