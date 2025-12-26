use clap::Args;
use crate::utils::{CmdOptional, cargo};
use crate::USER_PACKAGE_NAME;
use std::ffi::{OsStr, OsString};
use std::process::ExitStatus;
use std::ptr::fn_addr_eq;
use log::{error, info};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

#[derive(Debug, Args, Clone)]
pub struct UserArg {
    	#[arg(short, long, default_value = "riscv64gc-unknown-none-elf")]
	pub target: String
}

#[must_use]
pub fn run(arg: &UserArg) -> Option<ExitStatus> {
	let exit_status = build_user(arg)?;
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

fn build_user(arg: &UserArg) -> Option<ExitStatus> {
	info!("Building User");

	let rustflags = "\
		-C link-arg=-zmax-page-size=4096 \
		-C link-arg=-zcommon-page-size=4096 \
		-C force-frame-pointers=yes";

	let arch: &str = &arg.target;

	// Build the prototyper
	let status = cargo::Cargo::new("build")
		.package(USER_PACKAGE_NAME)
		.target(arch)
		.unstable("build-std", ["core", "alloc"])
		.env("RUSTFLAGS", rustflags)
		.release(true)
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
	let target_user_dir = current_dir.join("user").join("elf");
	let mut bin_file: Vec<OsString> = fs::read_dir(&user_bin)
		.unwrap()
		.filter_map(|entry| entry.ok())
		.filter_map(|entry| entry.path().file_stem().map(|stem| stem.to_os_string()))
		.collect();
	let elfs_path: Vec<PathBuf> = bin_file.iter().map(|f| {
		target_dir.join(f)
	}).collect();
	let dst_elfs_path: Vec<PathBuf> = bin_file.iter_mut().map(|f| {
		target_user_dir.join(f)
	}).collect();

	// Create binary from ELF
	info!("Copy Elf");
	for (src_path, dst_path) in elfs_path.iter().zip(dst_elfs_path) {
		info!("{:?} -> {:?}", src_path, dst_path);
		let result = Command::new("cp")
			.args([
				&OsStr::new(&src_path.to_string_lossy().as_ref()),
        			&OsStr::new(&dst_path.to_string_lossy().as_ref()),
			])
			.status()
			.ok();

		if result.is_none() {
			error!(
				"Failed to execute cp. Command not found or failed to start.\n\
				Source: {}\n\
				Destination: {}",
				src_path.display(),
				dst_path.display()
			);
		}
	}

	return Some(status);
}
