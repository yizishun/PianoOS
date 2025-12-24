use crate::TASK_MANAGER;
use crate::arch::riscv::trap::trap_end;
use crate::config::TICK_MS;
use crate::global::ARCH;
use crate::arch::common::ArchTime;
use crate::println;
use crate::syscall::syscall;
use crate::syscall::syscallid::SyscallID;
use crate::trap::entire::EntireContext;
use crate::trap::entire::EntireResult;
use crate::trap::fast::FastResult;
use crate::trap::fast::FastContext;
use crate::arch::common::ArchTrap;
use crate::harts::task_context_in_trap_stage;
use crate::trap::LoadedTrapStack;
use crate::mm::stack::KernelStack;
use crate::config::{KERNEL_STACK_ALIGN, KERNEL_STACK_SIZE};
use crate::{harts::hart_context_in_trap_stage, mm::stack::stack_drop};
use crate::arch::common::Arch;
use crate::arch::common::FlowContext;
use log::{warn, info};
use riscv::interrupt::Interrupt;
use riscv::interrupt::supervisor::Exception;
use riscv::interrupt::Trap;
use riscv::register::scause;
use riscv::register::sepc;
use riscv::register::sstatus;
use riscv::register::sscratch;
use riscv::register::stval;
use core::alloc::Layout;
use core::intrinsics::forget;
use core::ptr::NonNull;
use alloc::alloc::alloc;

pub extern "C" fn fast_handler_user(
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
	//TODO: translate ctx to get the real ctx
	ctx.tasks().app_info().user_time.end();
	ctx.tasks().app_info().kernel_time.start();

	let scause = scause::read();
	let stval = stval::read();
	let sepc = sepc::read();

	#[cfg(feature = "nested_trap")]
	unsafe {
		// SAFETY: allocated stack and flow context will be free in the end of the trap
		let stack_layout = Layout::from_size_align(KERNEL_STACK_SIZE, KERNEL_STACK_ALIGN).unwrap();
		let flow_context_layout = Layout::new::<FlowContext>();
		let stack = alloc(stack_layout) as *mut KernelStack;
		let flow_context = alloc(flow_context_layout) as *mut FlowContext;
		forget((*stack).load_as_stack(
			hart_context_in_trap_stage().hartid(),
			NonNull::new_unchecked(flow_context),
			<Arch as ArchTrap>::fast_handler_kernel,
			stack_drop));
		sstatus::set_sie();
	}
	match scause.cause()
		.try_into::<riscv::interrupt::Interrupt, riscv::interrupt::supervisor::Exception>()
		.unwrap() {

		Trap::Interrupt(Interrupt::SupervisorTimer) => {
			save_regs(&mut ctx);
			ctx.continue_with(timer_handler, ())
		}

		Trap::Exception(Exception::UserEnvCall) => {
			save_regs(&mut ctx);
			syscall_handler(ctx, a1, a2, a3, a4, a5, a6, a7)
		}
		Trap::Exception(Exception::StoreFault) |
		Trap::Exception(Exception::StorePageFault) |
		Trap::Exception(Exception::LoadFault) |
		Trap::Exception(Exception::LoadMisaligned) => {
			warn!("PageFault in application, kernel killed it.");
			warn!("Illegal addr: 0x{:x}", stval);
			warn!("excption pc: 0x{:x}", sepc);
			TASK_MANAGER.get().unwrap().exit_cur_and_run_next();
			ctx.switch_to()
		}
		Trap::Exception(Exception::IllegalInstruction) => {
			warn!("IllegalInstruction in application, kernel killed it.");
			warn!("excption pc: 0x{:x}", sepc);
			TASK_MANAGER.get().unwrap().exit_cur_and_run_next();
			ctx.switch_to()
		}
		Trap::Exception(Exception::InstructionFault) |
		Trap::Exception(Exception::InstructionMisaligned) |
		Trap::Exception(Exception::InstructionPageFault) => {
			warn!("Instruction PageFault in application, kernel killed it.");
			warn!("Illegal addr: 0x{:x}", stval);
			warn!("excption pc: 0x{:x}", sepc);
			ctx.tasks().app_info().end();
			TASK_MANAGER.get().unwrap().exit_cur_and_run_next();
			ctx.switch_to()
		}

		_ => {
			panic!("Unsupported trap {:?}, stval = {:#x}!", scause.cause(), stval);
		}
	}
}

pub extern "C" fn fast_handler_kernel(
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
	let sepc = sepc::read();
	match scause.cause()
		.try_into::<riscv::interrupt::Interrupt, riscv::interrupt::supervisor::Exception>()
		.unwrap() {

		Trap::Interrupt(Interrupt::SupervisorTimer) => {
			//TODO: do something useful
			save_regs(&mut ctx);
			//println!("Kernel recieve timer intrrupt");
			ARCH.set_next_timer_intr(TICK_MS);
			ctx.nested_restore()
		}

		Trap::Exception(Exception::StoreFault) |
		Trap::Exception(Exception::StorePageFault) |
		Trap::Exception(Exception::LoadFault) |
		Trap::Exception(Exception::LoadMisaligned) => {
			panic!("PageFault in kernel, kernel panic.\n Illegal addr: 0x{:x}\n, excption pc: 0x{:x}\n",
				stval,
				sepc
			);
		}
		Trap::Exception(Exception::IllegalInstruction) => {
			panic!("IllegalInstruction in application, kernel panic, excption pc: 0x{:x}",
			sepc);
		}
		Trap::Exception(Exception::InstructionFault) |
		Trap::Exception(Exception::InstructionMisaligned) |
		Trap::Exception(Exception::InstructionPageFault) => {
			panic!("Instruction PageFault in kernel, kernel panic.\n Illegal addr: 0x{:x}\n, excption pc: 0x{:x}\n",
				stval,
				sepc
			);
		}

		_ => {
			panic!("Unsupported trap {:?}, stval = {:#x}!", scause.cause(), stval);
		}
	}
}


pub extern "C" fn syscall_handler(
	mut ctx: FastContext,
	a1: usize,
	a2: usize,
	a3: usize,
	a4: usize,
	a5: usize,
	a6: usize,
	a7: usize,
) -> FastResult {
	let syscall_id: SyscallID = a7.try_into().unwrap();
	let tasks = ctx.tasks();
	let app_info = tasks.app_info();

	*app_info.syscall_record.get_mut(&syscall_id).unwrap() += 1;

	ctx.regs().a[0] = syscall(syscall_id, [ctx.a0(), a1, a2]) as usize;

	match syscall_id {
		SyscallID::Yield => {
			ctx.continue_with(yield_handler, ())
		},
		SyscallID::Exit => {
			TASK_MANAGER.get().unwrap().exit_cur_and_run_next();
			ctx.switch_to()
		}
		_ => {
			unsafe {
				if cfg!(feature = "nested_trap") {
					ctx.regs().pc = ctx.regs().pc + 4;
				} else {
					sepc::write(sepc::read() + 4);
				}
			}
			ctx.restore()
		}
	}
}

pub extern "C" fn yield_handler(ctx: EntireContext) -> EntireResult {
	let mut split_ctx = ctx.split().0;
	if cfg!(feature = "nested_trap") {
		let sepc = split_ctx.regs().pc;
		split_ctx.regs().set_pc(sepc + 4);
	} else {
		split_ctx.regs().set_sp(sscratch::read());
		split_ctx.regs().set_pc(sepc::read() + 4);
	}
	TASK_MANAGER.get().unwrap().suspend_cur_and_run_next();
	split_ctx.switch()
}

pub extern "C" fn timer_handler(ctx: EntireContext) -> EntireResult {
	let mut split_ctx = ctx.split().0;
	#[cfg(not(feature = "nested_trap"))] {
		split_ctx.regs().set_sp(sscratch::read());
		split_ctx.regs().set_pc(sepc::read());
	}
	ARCH.set_next_timer_intr(TICK_MS);
	TASK_MANAGER.get().unwrap().suspend_cur_and_run_next();
	split_ctx.switch()
}
