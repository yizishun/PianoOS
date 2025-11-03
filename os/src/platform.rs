use crate::console::ConsoleDevice;
use crate::console::ConsoleType;
use crate::console::KernelConsole;
use crate::devicetree::ParseDeviceTreeError;
use crate::devicetree::Tree;
use crate::devicetree::get_compatible_and_range;
use crate::devicetree::parse_device_tree;
use crate::driver::chardev::riscvsbi::RiscvSbi;
use crate::driver::chardev::uart16550::Uart16550Wrapper;
use crate::error::KernelError;

use alloc::boxed::Box;
use log::info;
use serde_device_tree::buildin::Node;
use spin::Once;
use spin::mutex::Mutex;

pub type BaseAddr = usize;

pub struct BoardInfo {
	pub cpu_num: Option<usize>,
	pub console: Option<(BaseAddr, ConsoleType)>,
}

impl BoardInfo {
	pub const fn new() -> BoardInfo {
		BoardInfo { cpu_num: None,
			    console: None }
	}
}

pub struct BoardDevice {
	pub console: Option<KernelConsole>,
}

impl BoardDevice {
	pub const fn new() -> Self {
		BoardDevice { console: None }
	}
}

pub struct Platform {
	pub board_info: BoardInfo,
	pub board_device: BoardDevice,
}

impl Platform {
	const fn new() -> Self {
		Platform { board_info: BoardInfo::new(),
			   board_device: BoardDevice::new() }
	}

	pub fn init_platform(dtb_addr: usize) -> Result<Self, KernelError> {
		let mut plat = Platform::new();
		let dtb = parse_device_tree(dtb_addr).unwrap_or_else(|_| panic!("parse dtb error"))
						     .share();
		let root: serde_device_tree::buildin::Node = serde_device_tree::from_raw_mut(&dtb)
	    .unwrap_or_else(|_| panic!("deserialze dtb fail"));
		let tree: Tree = root.deserialize();

		plat.board_info = Self::init_board_info(&tree, &root)?;

		plat.board_device = Self::init_board_device(&plat.board_info);

		Ok(plat)
	}

	fn init_board_info(tree: &Tree, root: &Node) -> Result<BoardInfo, ParseDeviceTreeError> {
		let mut board_info = BoardInfo::new();
		board_info.cpu_num = Some(tree.cpus.cpu.len());
		board_info.console = Self::init_console_info(root)?;
		Ok(board_info)
	}

	fn init_console_info(root: &Node)
			     -> Result<Option<(BaseAddr, ConsoleType)>, ParseDeviceTreeError>
	{
		let Some(stdout_path) = root.chosen_stdout_path() else {
			return Err(ParseDeviceTreeError::NoStdout);
		};
		let Some(stdout_node) = root.find(stdout_path) else {
			return Err(ParseDeviceTreeError::NoConsole);
		};
		let Some((compat, reg)) = get_compatible_and_range(&stdout_node) else {
			return Err(ParseDeviceTreeError::NoCompatOrRange);
		};
		Ok(compat.iter()
			 .find_map(|dev| ConsoleType::compatible(dev))
			 .map(|ctype| (reg.start, ctype)))
	}

	fn init_board_device(board_info: &BoardInfo) -> BoardDevice {
		let mut board_device = BoardDevice::new();
		board_device.console = Self::init_console(&board_info);
		board_device
	}

	fn init_console(board_info: &BoardInfo) -> Option<KernelConsole> {
		let Some((base, console_type)) = board_info.console else {
			return None;
		};
		let console: Box<dyn ConsoleDevice> = match console_type {
			ConsoleType::Uart16550U8 => Box::new(Uart16550Wrapper::<u8>::new(base)),
			ConsoleType::Uart16550U32 => Box::new(Uart16550Wrapper::<u32>::new(base)),
			ConsoleType::RiscvSbi => Box::new(RiscvSbi),
		};
		Some(KernelConsole::new(Mutex::new(console)))
	}

	pub fn print_platform_info(&self) {
		info!("cpu number: {}", self.board_info.cpu_num.unwrap());
		info!("uart type is {:#?}, base addr is 0x{:X}",
		      self.board_info.console.unwrap().1,
		      self.board_info.console.unwrap().0)
	}
}
