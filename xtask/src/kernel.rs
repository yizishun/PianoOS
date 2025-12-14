use clap::Args;
use crate::utils::{CmdOptional, cargo};
use crate::KERNEL_PACKAGE_NAME;
use std::process::ExitStatus;
use log::{error, info};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::env;
use std::time::SystemTime;

#[derive(Debug, Args, Clone)]
pub struct KernelArg {
    	#[arg(short, long, default_value = "riscv64gc-unknown-none-elf")]
	pub target: String,

	#[arg(long, default_value_t = false)]
    	pub release: bool,

	#[arg(long, short = 'f', default_value = "float")]
    	pub features: Vec<String>,

	#[arg(long, default_value_t = false)]
	pub test: bool,
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
	let test = arg.test;

	// Build the kernel
	let mut cmd = if test {
		cargo::Cargo::new("test")
	} else {
		cargo::Cargo::new("build")
	};
	cmd.package(KERNEL_PACKAGE_NAME)
		.target(arch)
		.env("RUSTFLAGS", rustflags)
		.features(&arg.features)
		.release(release);
	if test {
		cmd.no_run();
	}
	
	info!("Build kernel: {:?}", cmd.cmd);
	let status = cmd.status()
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
	//let elf_path = target_dir.join(KERNEL_PACKAGE_NAME);
	let elf_path = if test {
		match find_test_binary(&target_dir, KERNEL_PACKAGE_NAME) {
			Some(path) => {
				info!("Found test binary: {}", path.display());
				path
			},
			None => {
				error!("Could not find generated test binary in {}", target_dir.join("deps").display());
				return None;
			}
		}
	} else {
		target_dir.join(KERNEL_PACKAGE_NAME)
	};

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

fn find_test_binary(target_dir: &Path, package_name: &str) -> Option<PathBuf> {
    let deps_dir = target_dir.join("deps");
    
    let mut files: Vec<(PathBuf, SystemTime)> = std::fs::read_dir(&deps_dir)
        .ok()?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            
            // 过滤条件：
            // 1. 必须是文件
            // 2. 文件名必须以 package_name 开头 (比如 "PianoOS-")
            // 3. 不能有扩展名 (排除 .d 依赖描述文件)
            // 4. (可选) 在 Linux 上通常测试文件是可执行的，但这里简单判断扩展名即可
            if path.is_file() 
               && path.file_name()?.to_string_lossy().starts_with(package_name)
               && path.extension().is_none() 
            {
                // 获取修改时间
                let metadata = path.metadata().ok()?;
                let modified = metadata.modified().ok()?;
                Some((path, modified))
            } else {
                None
            }
        })
        .collect();

    if files.is_empty() {
        return None;
    }

    files.sort_by(|a, b| a.1.cmp(&b.1));
    
    files.pop().map(|(path, _)| path)
}