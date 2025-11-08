use clap::Args;
use log::{error, info, warn};
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

use crate::KERNEL_PACKAGE_NAME;

#[derive(Debug, Args, Clone)]
pub struct QemuArg {
	#[arg(short, long, default_value = "riscv64gc-unknown-none-elf")]
	pub target: String,

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
}

#[must_use]
pub fn run(arg: &QemuArg) -> Option<ExitStatus> {
	let arch = &arg.target;
	let qemu_bin = arg
		.qemu
		.clone()
		.unwrap_or_else(|| guess_qemu_system(arch).to_string());

	let current_dir = env::current_dir().ok()?;
	let target_dir = get_target_dir(&current_dir, arch);

	let bin_path = target_dir.join(format!("{}.bin", KERNEL_PACKAGE_NAME));
	if !bin_path.exists() {
		error!(
		"Kernel binary not found: {}\nPlease perform a build first (e.g., `cargo kernel`).",
		bin_path.display()
		);
		return None;
	}

	if !arg.bios.exists() {
		warn!(
		"Note: The specified BIOS does not exist: {} (This can be ignored if it is not RISC-V or if the -kernel method is used)",
		arg.bios.display()
		);
	}

	let base_addr = match parse_int_or_hex(&arg.base_addr) {
		Some(v) => format!("0x{:x}", v),
		None => {
		error!("cannot parse base-addr：{}", arg.base_addr);
		return None;
		}
	};

	let mut cmd = Command::new(&qemu_bin);
	cmd.arg("-machine").arg(&arg.machine)
		.arg("-smp").arg(arg.smp.to_string())
		.arg("-bios").arg(&arg.bios)
		.args([
		"-device",
		&format!("loader,file={},addr={}", bin_path.display(), base_addr),
		]);

	if !arg.gui {
		cmd.arg("-nographic");
	}

	info!("Boot QEMU: {:?}", cmd);

	match cmd.status() {
		Ok(status) => {
		if !status.success() {
			error!("QEMU Exit Code {:?}", status.code());
		}
		Some(status)
		}
		Err(e) => {
		error!(
			"boot QEMU fail: {}\nplease confirm `{}` is installed",
			e, qemu_bin
		);
		None
		}
	}
}

fn get_target_dir(current_dir: &Path, arch: &str) -> PathBuf {
    	current_dir.join("target").join(arch).join("release")
}

fn parse_int_or_hex(s: &str) -> Option<u64> {
	let s = s.trim();
	if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
		u64::from_str_radix(hex, 16).ok()
	} else {
		s.parse::<u64>().ok()
	}
}

fn guess_qemu_system(arch: &str) -> &'static str {
	if arch.starts_with("riscv64") {
		"qemu-system-riscv64"
	} else if arch.starts_with("loongarch64") || arch.starts_with("loongarch") {
		"qemu-system-loongarch64"
	} else if arch.starts_with("riscv32") {
		"qemu-system-riscv32"
	} else if arch.starts_with("x86_64") {
		"qemu-system-x86_64"
	} else if arch.starts_with("aarch64") {
		"qemu-system-aarch64"
	} else {
		"qemu-system-riscv64"
	}
}

