use clap::Args;
use crate::utils::{CmdOptional, cargo};
use crate::USER_PACKAGE_NAME;
use crate::ARCH;
use std::ffi::{OsStr, OsString};
use std::process::ExitStatus;
use log::{error, info};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

#[derive(Debug, Args, Clone)]
pub struct UserArg {
	#[arg(short, long)]
	pub target: Option<String>
}

#[must_use]
pub fn run(arg: &UserArg) -> Option<ExitStatus> {
	let exit_status = build_User(arg)?;
	if !exit_status.success() {
		error!(
			"Failed to execute rust-objcopy. Please ensure that cargo-binutils is installed and available in your system's PATH."
		);
		return Some(exit_status);
	}

	Some(exit_status)
}

fn get_target_dir(current_dir: &Path, arch: &str) -> PathBuf {
    	current_dir.join("target").join(arch).join("release")
}

fn build_User(arg: &UserArg) -> Option<ExitStatus> {
	info!("Building User");

	let rustflags = 
		"-C relocation-model=pie -C link-arg=-pie -C force-frame-pointers=yes";

	let arch: &str = &arg.target.as_deref().unwrap_or(ARCH);

	// Build the prototyper
	let status = cargo::Cargo::new("build")
		.package(USER_PACKAGE_NAME)
		.target(arch)
		.unstable("build-std", ["core", "alloc"])
		.env("RUSTFLAGS", rustflags)
		.release()
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
	let target_dir = get_target_dir(&current_dir, &arch);
	let user_bin = current_dir.join("user").join("src").join("bin");
	let target_user_bin = current_dir.join("user").join("binary");
	let mut bin_file: Vec<OsString> = fs::read_dir(&user_bin)
		.unwrap()
		.filter_map(|entry| entry.ok())
		.filter_map(|entry| entry.path().file_stem().map(|stem| stem.to_os_string()))
		.collect();
	let elfs_path: Vec<PathBuf> = bin_file.iter().map(|f| {
		target_dir.join(f)	
	}).collect();
	let bins_path: Vec<PathBuf> = bin_file.iter_mut().map(|f| {
		let ext = OsString::from(".bin");
		f.push(ext);
		target_user_bin.join(f)
	}).collect();
	
	// Create binary from ELF
	info!("Converting ELF to binary with rust-objcopy");
	for (elf_path, bin_path) in elfs_path.iter().zip(bins_path) {
		info!("{:?} -> {:?}", elf_path, bin_path);
		let result = Command::new("rust-objcopy")
			.args([
				"-O",
				"binary",
				"--binary-architecture=riscv64",
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
	}

	return Some(status);
}
