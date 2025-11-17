use clap::Args;
use crate::utils::{CmdOptional, cargo};
use crate::KERNEL_PACKAGE_NAME;
use std::process::ExitStatus;
use log::{error, info};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::env;

#[derive(Debug, Args, Clone)]
pub struct KernelArg {
    	#[arg(short, long, default_value = "riscv64gc-unknown-none-elf")]
	pub target: String,

	#[arg(long, default_value_t = false)]
    	pub release: bool,

	#[arg(long, short = 'f', default_value = "float")]
    	pub features: Vec<String>,
}

#[must_use]
pub fn run(arg: &KernelArg) -> Option<ExitStatus> {
	let exit_status = build_kernel(arg)?;
	if !exit_status.success() {
		error!(
			"Failed to execute rust-objcopy. Please ensure that cargo-binutils is installed and available in your system's PATH."
		);
		return Some(exit_status);
	}

	Some(exit_status)
}

fn get_target_dir(current_dir: &Path, arch: &str, release: bool) -> PathBuf {
    let build_type = if release { "release" } else { "debug" };
    current_dir.join("target").join(arch).join(build_type)
}

fn build_kernel(arg: &KernelArg) -> Option<ExitStatus> {
	info!("Building Kernel");

	let rustflags = 
		"-C relocation-model=pie -C force-frame-pointers=yes";

	let arch: &str = &arg.target;
	let release = arg.release;

	// Build the prototyper
	let status = cargo::Cargo::new("build")
		.package(KERNEL_PACKAGE_NAME)
		.target(arch)
		.env("RUSTFLAGS", rustflags)
		.features(&arg.features)
		.release(release)
		.status()
		.ok()?;

	if !status.success() {
		error!(
			"Failed to build prototyper. Please check the cargo output above for detailed error information."
		);
		return Some(status);
	}

    	// Get target directory once instead of recreating it
	let current_dir = env::current_dir().ok()?;
	let target_dir = get_target_dir(&current_dir, &arch, release);
	let elf_path = target_dir.join(KERNEL_PACKAGE_NAME);
	let bin_path = target_dir.join(format!("{}.bin", KERNEL_PACKAGE_NAME));

	// Create binary from ELF
	info!("Converting ELF to binary with rust-objcopy");
	let result = Command::new("rust-objcopy")
		.args([
			"-O",
			"binary",
			"--strip-all",
			&elf_path.to_string_lossy(),
			&bin_path.to_string_lossy(),
		])
		.status()
		.ok();

	if result.is_none() {
		error!(
			"Failed to execute rust-objcopy. Command not found or failed to start.\n\
			Source: {}\n\
			Destination: {}\n\
			Please install cargo-binutils with cmd: cargo install cargo-binutils",
			elf_path.display(),
			bin_path.display()
		);
	}

	result
}
