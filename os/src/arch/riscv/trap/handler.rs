use crate::TASK_MANAGER;
use crate::config::TICK_MS;
use crate::global::ARCH;
use crate::arch::common::ArchTime;
use crate::syscall::syscall;
use crate::syscall::syscallid::SyscallID;
use crate::trap::entire::EntireContext;
use crate::trap::entire::EntireResult;
use crate::trap::fast::FastResult;
use crate::trap::fast::FastContext;
use riscv::register::sscratch;
use riscv::register::sepc;

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
	TASK_MANAGER.get().unwrap().suspend_cur_and_run_next();
	split_ctx.restore()
}

pub extern "C" fn timer_handler(ctx: EntireContext) -> EntireResult {
	let mut split_ctx = ctx.split().0;
	split_ctx.regs().set_sp(sscratch::read());
	split_ctx.regs().set_pc(sepc::read());
	ARCH.set_next_timer_intr(TICK_MS);
	TASK_MANAGER.get().unwrap().suspend_cur_and_run_next();
	split_ctx.restore()
}
