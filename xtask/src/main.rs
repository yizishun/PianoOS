use clap::{Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use std::process::{ExitCode, ExitStatus};

use crate::kernel::KernelArg;
use crate::user::UserArg;
use crate::qemu::QemuArg;
use crate::all::AllArg;

mod utils;
mod kernel;
mod user;
mod qemu;
mod all;
mod logger;

const KERNEL_PACKAGE_NAME: &str = "PianoOS";
const USER_PACKAGE_NAME: &str = "user_lib";

#[derive(Parser)]
#[command(
	name = "xtask",
	version,
	about = "A task runner for building, running PianoOS and user program",
	long_about = None,
)]
struct Cli {
	#[command(subcommand)]
	cmd: Cmd,
	#[command(flatten)]
	verbose: Verbosity<InfoLevel>,
}

#[derive(Subcommand)]
enum Cmd {
	/// Build and configure the RustSBI Prototyper bootloader.
	Kernel(KernelArg),
	/// Build test-kernel for the RustSBI Prototyper.
	User(UserArg),
	/// Run Kernel using Qemu
	Qemu(QemuArg),
	/// Run all above
	All(AllArg),
}

fn main() -> ExitCode {
	let cli_args = Cli::parse();
	if let Err(e) = logger::Logger::init(&cli_args) {
		eprintln!("Logger initialization failed: {}", e);
        	return ExitCode::FAILURE;
	}

	let result = match &cli_args.cmd {
		Cmd::Kernel(arg) => kernel::run(arg),
		Cmd::User(arg) => user::run(arg),
		Cmd::Qemu(arg) => qemu::run(arg),
		Cmd::All(arg) => all::run(arg),
	};

	ExitCode::SUCCESS
}
