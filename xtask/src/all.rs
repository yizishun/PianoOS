use clap::Args;
use log::{error, info};
use std::path::PathBuf;
use std::process::ExitStatus;

use crate::user::{self, UserArg};
use crate::{kernel, qemu};
use crate::kernel::KernelArg;
use crate::qemu::QemuArg;
use crate::USER_PACKAGE_NAME;

#[derive(Debug, Args, Clone)]
pub struct AllArg {
	//qemu + kernel + user args
	#[arg(short, long, default_value = "riscv64gc-unknown-none-elf")]
    	pub target: String,
	#[arg(long, default_value_t = false)]
    	pub release: bool,

	//qemu args
	#[arg(long, default_value_t = 8)]
	pub smp: usize,

	#[arg(long, default_value = "virt")]
	pub machine: String,

	#[arg(long, default_value = "./bootloader/rustsbi-qemu.bin")]
	pub bios: PathBuf,

	#[arg(long)]
	pub gui: bool,

	#[arg(long, default_value = "0x80200000")]
	pub base_addr: String,

	#[arg(long)]
	pub qemu: Option<String>,

	#[arg(long)]
	pub gdbserver: bool,

	#[arg(long)]
	pub gdbclient: bool,

	#[arg(long)]
	pub gdb_bin: Option<String>,
}

#[must_use]
pub fn run(arg: &AllArg) -> Option<ExitStatus> {
	let arch = &arg.target;
	let release = arg.release;

	let uarg = UserArg {
		target: arg.target.clone() 
	};
	info!("Building User package: {USER_PACKAGE_NAME}");
	let u_status = user::run(&uarg)?;
	if !u_status.success() {
		error!("User Build fail.");
		return Some(u_status);
	}

	let karg = KernelArg { 
		target: arg.target.clone(),
		release
	};
	info!("Building Kernel");
	let k_status = kernel::run(&karg)?;
	if !k_status.success() {
		error!("Kernel Build fail.");
		return Some(k_status);
	}

	let qarg = QemuArg {
		target: arg.target.clone(),
		release,
		smp: arg.smp,
		machine: arg.machine.clone(),
		bios: arg.bios.clone(),
		gui: arg.gui,
		base_addr: arg.base_addr.clone(),
		qemu: arg.qemu.clone(),
		gdbserver: arg.gdbserver,
		gdbclient: arg.gdbclient,
		gdb_bin: arg.gdb_bin.clone()
	};
	qemu::run(&qarg)
}
