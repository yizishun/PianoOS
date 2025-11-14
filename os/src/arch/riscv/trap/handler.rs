use crate::TASK_MANAGER;
use crate::syscall::syscall;
use crate::syscall::syscallid::SyscallID;
use crate::trap::entire::EntireContext;
use crate::trap::entire::EntireResult;
use crate::trap::fast::FastResult;
use crate::trap::fast::FastContext;
use log::debug;
use log::info;
use log::warn;
use riscv::interrupt::supervisor::Exception;
use riscv::register::sscratch;
use riscv::register::{scause, stval};
use riscv::register::mcause::Trap;
use riscv::register::sepc;

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
		syscall_handler(ctx, a1, a2, a3, a4, a5, a6, a7)
	}
	Trap::Exception(Exception::StoreFault) |
	Trap::Exception(Exception::StorePageFault) |
	Trap::Exception(Exception::LoadFault) | 
	Trap::Exception(Exception::LoadMisaligned) => {
		unsafe {
			(*ctx.tasks().app_info.get()).end();
		}
		warn!("PageFault in application, kernel killed it.");
		warn!("Illegal addr: 0x{:x}", stval);
		warn!("excption pc: 0x{:x}", sepc::read());
		TASK_MANAGER.get().unwrap().exit_cur_and_run_next();
		ctx.switch_to()
	}
	Trap::Exception(Exception::IllegalInstruction) => {
		unsafe {
			(*ctx.tasks().app_info.get()).end();
		}
		warn!("IllegalInstruction in application, kernel killed it.");
		warn!("excption pc: 0x{:x}", sepc::read());
		TASK_MANAGER.get().unwrap().exit_cur_and_run_next();
		ctx.switch_to()
	}
	Trap::Exception(Exception::InstructionFault) |
	Trap::Exception(Exception::InstructionMisaligned) |
	Trap::Exception(Exception::InstructionPageFault) => {
		unsafe {
			(*ctx.tasks().app_info.get()).end();
		}
		warn!("Instruction PageFault in application, kernel killed it.");
		warn!("Illegal addr: 0x{:x}", stval);
		warn!("excption pc: 0x{:x}", sepc::read());
		TASK_MANAGER.get().unwrap().exit_cur_and_run_next();
		ctx.switch_to()
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
	let app_info = unsafe { tasks.app_info.get().as_mut().unwrap() };

	*app_info.syscall_record.get_mut(&syscall_id).unwrap() += 1;

	ctx.regs().a[0] = syscall(syscall_id, [ctx.a0(), a1, a2]) as usize;

	match syscall_id {
		SyscallID::Yield => {
			ctx.continue_with(yield_handler, ())
		},
		SyscallID::Exit => {
			ctx.switch_to()
		}
		_ => {
			unsafe {
				sepc::write(sepc::read() + 4);
			}
			ctx.restore()
		}
	}
}

pub extern "C" fn yield_handler(ctx: EntireContext) -> EntireResult {
	let mut split_ctx = ctx.split().0;
	split_ctx.regs().set_sp(sscratch::read());
	split_ctx.regs().set_pc(sepc::read() + 4);
	info!("yield");
	TASK_MANAGER.get().unwrap().suspend_cur_and_run_next();
	EntireResult::Restore
}
