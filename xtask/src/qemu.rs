use clap::Args;
use log::{error, info, warn};
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use std::time::Instant;

use crate::KERNEL_PACKAGE_NAME;

#[derive(Debug, Args, Clone)]
pub struct QemuArg {
	#[arg(short, long, default_value = "riscv64gc-unknown-none-elf")]
	pub target: String,

	#[arg(long, default_value_t = false)]
    	pub release: bool,

	#[arg(long, default_value_t = 1)]
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

	#[arg(long, short = 's')]
	pub gdbserver: bool,

	#[arg(long, short = 'c')]
	pub gdbclient: bool,

	#[arg(long)]
	pub gdb_bin: Option<String>,
}

#[must_use]
pub fn run(arg: &QemuArg) -> Option<ExitStatus> {
	let arch = &arg.target;
	let release = arg.release;
	let qemu_bin = arg
		.qemu
		.clone()
		.unwrap_or_else(|| guess_qemu_system(arch).to_string());

	let current_dir = env::current_dir().ok()?;
	let target_dir = get_target_dir(&current_dir, arch, release);

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
		error!("cannot parse base-addr:{}", arg.base_addr);
		return None;
		}
	};

	if arg.gdbclient {
		return run_gdb_client(arg)
	}

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
	if arg.gdbserver {
 	       cmd.arg("-s").arg("-S");
    	}

	info!("Boot QEMU: {:?}", cmd);

	let now = Instant::now();
	let res = cmd.status();
	let elapsed_time = now.elapsed();
	info!("Running kernel took {} micro-seconds.", elapsed_time.as_micros());

	match res {
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

pub fn run_gdb_client(arg: &QemuArg) -> Option<ExitStatus> {
	let arch = &arg.target;
	let release = arg.release;

	let current_dir = env::current_dir().ok()?;
	let target_dir = get_target_dir(&current_dir, arch, release);

	match run_gdb_client_inner(arg, &target_dir) {
		Ok(status) => Some(status),
		Err(e) => {
			error!("Failed to run gdb client: {}", e);
			None
		}
	}
}

fn run_gdb_client_inner(arg: &QemuArg, target_dir: &Path) -> Result<ExitStatus, String> {
	let arch = &arg.target;

	let elf_path = target_dir.join(KERNEL_PACKAGE_NAME);
	if !elf_path.exists() {
			return Err(format!(
				"Kernel ELF not found: {}. Did you build it first?",
				elf_path.display()
			));
	}

	let gdb_bin = arg
		.gdb_bin
		.clone()
		.unwrap_or_else(|| guess_gdb_bin(arch).to_string());

	let mut gdb_cmd = Command::new(&gdb_bin);

	// riscv64-none-elf-gdb \
	//   -ex "file <elf>" \
	//   -ex "set arch riscv:rv64" \
	//   -ex "target remote localhost:<port>" \
	//   -ex "layout asm" \
	//   -tui
	gdb_cmd
		.arg("-ex")
		.arg(format!("file {}", elf_path.display()));

	if let Some(gdb_arch) = guess_gdb_arch(arch) {
		gdb_cmd.arg("-ex").arg(format!("set arch {}", gdb_arch));
	}

	gdb_cmd
		.arg("-ex")
		.arg(format!("target remote localhost:{}", 1234))
		.arg("-ex")
		.arg("layout asm")
		.arg("-tui");

	info!("Start GDB client: {:?}", gdb_cmd);

	gdb_cmd
		.status()
		.map_err(|e| format!("failed to spawn gdb `{}`: {}", gdb_bin, e))
}



fn get_target_dir(current_dir: &Path, arch: &str, release: bool) -> PathBuf {
	let build_type = if release { "release" } else { "debug" };
	current_dir.join("target").join(arch).join(build_type)
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

fn guess_gdb_bin(arch: &str) -> &'static str {
	if arch.starts_with("riscv64") {
		"riscv64-none-elf-gdb"
	} else if arch.starts_with("riscv32") {
		"riscv32-none-elf-gdb"
	} else if arch.starts_with("x86_64") {
		"gdb"
	} else if arch.starts_with("aarch64") {
		"aarch64-none-elf-gdb"
	} else {
		"gdb"
	}
}

fn guess_gdb_arch(arch: &str) -> Option<&'static str> {
	if arch.starts_with("riscv64") {
		Some("riscv:rv64")
	} else if arch.starts_with("riscv32") {
		Some("riscv:rv32")
	} else if arch.starts_with("x86_64") {
		Some("i386:x86-64")
	} else if arch.starts_with("aarch64") {
		Some("aarch64")
	} else {
		None
	}
}

