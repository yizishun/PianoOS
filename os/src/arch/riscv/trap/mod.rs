use core::intrinsics::unreachable;
use core::{arch::naked_asm, usize};
use core::arch::asm;
use riscv::register::sie;
use riscv::register::sstatus::FS;
use riscv::register::{sepc, sscratch, sstatus::{self, SPP}, stvec::{self, Stvec}};
use log::{info, warn};
use crate::config::{TRAMPOLINE_VADDR, USER_STACK_SIZE};
use crate::harts::task_context_in_trap_stage;
use crate::{arch::{common::ArchTrap, riscv::Riscv64}};
use crate::{USER_STACK, println};
use crate::mm::stack::UserStack;
use crate::trap::fast::FastResult;
use crate::trap::fast::FastContext;
use crate::trap::LoadedTrapStack;
use crate::harts::hart_context_in_trap_stage;
pub mod handler;

impl<C> ArchTrap for Riscv64<C> {
	#[inline]
	unsafe fn load_direct_trap_entry(&self) {
		unsafe {
		    stvec::write(Stvec::new(trap_entry as *const() as usize, stvec::TrapMode::Direct));
		}
	}

	extern "C" fn fast_handler_user(
		ctx: FastContext,
		a1: usize,
		a2: usize,
		a3: usize,
		a4: usize,
		a5: usize,
		a6: usize,
		a7: usize,
	) -> FastResult {
		let result = handler::fast_handler_user(ctx, a1, a2, a3, a4, a5, a6, a7);
		result
	}

	extern "C" fn fast_handler_kernel(
		ctx: FastContext,
		a1: usize,
		a2: usize,
		a3: usize,
		a4: usize,
		a5: usize,
		a6: usize,
		a7: usize,
	) -> FastResult {
		let result = handler::fast_handler_kernel(ctx, a1, a2, a3, a4, a5, a6, a7);
		result
	}

	unsafe extern "C" fn boot_entry(a0: usize, a1: usize) -> ! {
		//a0: user sp, a1: user addr space
		unsafe {
			unsafe extern "C" {
				fn _boot_entry();
				fn _trap_enrty();
			}
			let real_boot_enrty_trampoline = _boot_entry as usize - _trap_enrty as usize + TRAMPOLINE_VADDR;
			println!("b: 0x{:x}, t: 0x{:x}, tram: 0x{:x}, r: 0x{:x}", _boot_entry as usize, _trap_enrty as usize, TRAMPOLINE_VADDR, real_boot_enrty_trampoline);
			asm!(
				"fence.i",
				"mv a0, {user_sp}",
				"mv a1, {user_addr_space}",
				"jr {real_entry}",
				user_sp = in(reg) a0,
				user_addr_space = in(reg) a1,
				real_entry = in(reg) real_boot_enrty_trampoline
			);
			unreachable();
		}

	}

	extern "C" fn boot_handler(entry: usize, trampoline: usize, utraph: usize) {
		unsafe {
			sstatus::set_spp(SPP::User);
			sstatus::set_fs(FS::Initial);
			sie::set_stimer();
			sepc::write(entry);
			stvec::write(Stvec::new(trampoline, stvec::TrapMode::Direct));
			sscratch::write(utraph);
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

#[cfg(feature = "float")]
macro_rules! fsave {
	($reg:ident => $ptr:ident[$pos:expr]) => {
		concat!("fsd ",
			stringify!($reg),
			", 8*",
			$pos,
			'(',
			stringify!($ptr),
			')')
	};
}
#[cfg(not(feature = "float"))]
macro_rules! fsave {
	($reg:ident => $ptr:ident[$pos:expr]) => {
		""
	};
}

#[cfg(feature = "float")]
macro_rules! fload {
	($ptr:ident[$pos:expr] => $reg:ident) => {
		concat!("fld ",
			stringify!($reg),
			", 8*",
			$pos,
			'(',
			stringify!($ptr),
			')')
	};
}
#[cfg(not(feature = "float"))]
macro_rules! fload {
	($ptr:ident[$pos:expr] => $reg:ident) => {
		""
	};

}

//TODO:其实不能将他和nested trap feature绑定
#[cfg(feature = "nested_trap")]
macro_rules! csr_save_n {
	($csr:ident => $tmp:ident => $ptr:ident[$pos:expr]) => {
		concat!(
			"csrr ", stringify!($tmp), ", ", stringify!($csr), "\n\t",
			"sd ", stringify!($tmp), ", 8*", stringify!($pos), "(", stringify!($ptr), ")"
		)
	};
}
#[cfg(not(feature = "nested_trap"))]
macro_rules! csr_save_n {
	($csr:ident => $tmp:ident => $ptr:ident[$pos:expr]) => {
		""
	};
}
#[cfg(feature = "nested_trap")]
macro_rules! csr_load_n {
    ($ptr:ident[$pos:expr] => $tmp:ident => $csr:ident) => {
        concat!(
            "ld ", stringify!($tmp), ", 8*", stringify!($pos),
            "(", stringify!($ptr), ")\n\t",
            "csrw ", stringify!($csr), ", ", stringify!($tmp)
        )
    };
}
#[cfg(not(feature = "nested_trap"))]
macro_rules! csr_load_n {
    	($ptr:ident[$pos:expr] => $tmp:ident => $csr:ident) => {
		""
	};

}

macro_rules! csr_save {
	($csr:ident => $tmp:ident => $ptr:ident[$pos:expr]) => {
		concat!(
			"csrr ", stringify!($tmp), ", ", stringify!($csr), "\n\t",
			"sd ", stringify!($tmp), ", 8*", stringify!($pos), "(", stringify!($ptr), ")"
		)
	};
}
macro_rules! csr_load {
    ($ptr:ident[$pos:expr] => $tmp:ident => $csr:ident) => {
        concat!(
            "ld ", stringify!($tmp), ", 8*", stringify!($pos),
            "(", stringify!($ptr), ")\n\t",
            "csrw ", stringify!($csr), ", ", stringify!($tmp)
        )
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
	pub uaddr_space: usize, // 32
	pub utrap_handler: usize, // 33
	#[cfg(feature = "float")]
	pub f:  [usize; 32], // 34..
}

impl FlowContext {
	pub const ZERO: Self = Self {
		ra: 0,
		t: [0; 7],
		a: [0; 8],
		s: [0; 12],
		gp: 0,
		tp: 0,
		sp: 0,
		pc: 0,
		uaddr_space: 0,
		utrap_handler: 0,
		#[cfg(feature = "float")]
		f: [0; 32],
	};

	pub fn new(sp: usize, entry: usize, uaddr_space: usize) -> Self {
		Self{
			ra: 0,
			t: [0; 7],
			a: [0; 8],
			s: [0; 12],
			gp: 0,
			tp: 0,
			sp,
			pc: entry,
			uaddr_space,
			utrap_handler: 0, //TODO:
			#[cfg(feature = "float")]
			f: [0; 32],
		}
	}

	/// 从上下文向硬件加载非调用规范约定的寄存器。
	#[inline]
	pub(crate) unsafe fn load_others(&self) {
		unsafe {
			asm!(
				//"mv         gp, {gp}",
				//"mv         tp, {tp}",
				"csrw sscratch, {sp}",
				"csrw     sepc, {pc}",
				//gp = in(reg) self.gp,
				//tp = in(reg) self.tp,
				sp = in(reg) self.sp,
				pc = in(reg) self.pc,
			);
		}
	}

	pub fn set_sp(&mut self, sp: usize) {
		self.sp = sp;
	}

	pub fn set_pc(&mut self, pc: usize) {
		self.pc = pc;
	}
}

#[unsafe(naked)]
#[unsafe(export_name = "_boot_entry")]
#[unsafe(link_section = ".text.trampoline.boot")]
pub unsafe extern "C" fn boot_entry(a0: usize, a1: usize) -> ! {
	naked_asm!(
		".align 2",
		"mv sp, a0",
		"csrw satp, a1",
		"sfence.vma",
		"sret",
	)
}

#[unsafe(naked)]
#[unsafe(export_name = "_trap_enrty")]
#[unsafe(link_section = ".text.trampoline.trap")]
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
		//如果要支持嵌套trap，需要保存可能被破坏的sscratch
		csr_save_n!(sscratch => t0 => a0[30]),
		csr_save_n!(sepc => t0 => a0[31]),
		// 换内核地址空间内核栈地址
		load!(sp[3] => t1),
		load!(t1[2] => sp),
		// 换地址空间
		csr_load!(t1[1] => t2 => satp),
		"sfence.vma",
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
		"0:", // 加载上下文指针 NOTE: 必须要在fast_handler中设置翻译的地址
		load!(sp[4] => a1),
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
		save!(gp  => a1[28]),

		"csrr t0, sstatus",
		"srli t0, t0, 13",
		"andi t0, t0, 0b11", //FS
		"li t1, 3",
		"bne t0, t1, fs_not_dirty1", //if FS != Dirty, skip f reg save
		fsave!(f0 =>  a1[34]),
		fsave!(f1 =>  a1[35]),
		fsave!(f2 =>  a1[36]),
		fsave!(f3 =>  a1[37]),
		fsave!(f4 =>  a1[38]),
		fsave!(f5 =>  a1[39]),
		fsave!(f6 =>  a1[40]),
		fsave!(f7 =>  a1[41]),
		fsave!(f8 =>  a1[42]),
		fsave!(f9 =>  a1[43]),
		fsave!(f10 => a1[44]),
		fsave!(f11 => a1[45]),
		fsave!(f12 => a1[46]),
		fsave!(f13 => a1[47]),
		fsave!(f14 => a1[48]),
		fsave!(f15 => a1[49]),
		fsave!(f16 => a1[50]),
		fsave!(f17 => a1[51]),
		fsave!(f18 => a1[52]),
		fsave!(f19 => a1[53]),
		fsave!(f20 => a1[54]),
		fsave!(f21 => a1[55]),
		fsave!(f22 => a1[56]),
		fsave!(f23 => a1[57]),
		fsave!(f24 => a1[58]),
		fsave!(f25 => a1[59]),
		fsave!(f26 => a1[60]),
		fsave!(f27 => a1[61]),
		fsave!(f28 => a1[62]),
		fsave!(f29 => a1[63]),
		fsave!(f30 => a1[64]),
		fsave!(f31 => a1[65]),
		"fs_not_dirty1:",
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
		load!(a1[28] => gp),

		fload!(a1[34] => f0),
		fload!(a1[35] => f1),
		fload!(a1[36] => f2),
		fload!(a1[37] => f3),
		fload!(a1[38] => f4),
		fload!(a1[39] => f5),
		fload!(a1[40] => f6),
		fload!(a1[41] => f7),
		fload!(a1[42] => f8),
		fload!(a1[43] => f9),
		fload!(a1[44] => f10),
		fload!(a1[45] => f11),
		fload!(a1[46] => f12),
		fload!(a1[47] => f13),
		fload!(a1[48] => f14),
		fload!(a1[49] => f15),
		fload!(a1[50] => f16),
		fload!(a1[51] => f17),
		fload!(a1[52] => f18),
		fload!(a1[53] => f19),
		fload!(a1[54] => f20),
		fload!(a1[55] => f21),
		fload!(a1[56] => f22),
		fload!(a1[57] => f23),
		fload!(a1[58] => f24),
		fload!(a1[59] => f25),
		fload!(a1[60] => f26),
		fload!(a1[61] => f27),
		fload!(a1[62] => f28),
		fload!(a1[63] => f29),
		fload!(a1[64] => f30),
		fload!(a1[65] => f31),
		"li t0, (1 << 13)",
		"csrc sstatus, t0", //csr clear FS dirty(11) to clean(10)
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
		csr_load_n!(a1[30] => a0 => sscratch),
		csr_load_n!(a1[31] => a0 => sepc),
		// a0=free, a1=k_flow, sscratch=u_stack
		// a0=u_traph, a1=k_flow
		load!(a1[33] => a0),
		// switch to u addr space
		// a0=u_traph, a1=free
		csr_load!(a1[32] => a1 => satp),
		"sfence.vma",
		// a0=u_traph, a1=u_flow
		load!(a0[0] => a1),
		// sp=u_traph
		"mv sp, a0",
		load!(a1[ 8] => a0),
		load!(a1[ 9] => a1),
		// sp=u_stack sscratch=u_traph
		exchange!(),
		r#return!(),
	)
}

// some commmon bavaior in the end of trap
pub extern "C" fn trap_end(switch: bool) {
	#[cfg(feature = "nested_trap")]
	unsafe {
		sstatus::clear_sie();
		let sp = task_context_in_trap_stage().flow_context.get().as_mut().unwrap().sp;
		drop(LoadedTrapStack::get(sp));
	}
	// switch will end previous kernel time in switch function
	if !switch {
		task_context_in_trap_stage().app_info().kernel_time.end();
	}

	// switch should restore sp and pc
	if switch {
		unsafe {
			task_context_in_trap_stage().flow_context().load_others();
		}
	}
	task_context_in_trap_stage().app_info().user_time.start();
}
