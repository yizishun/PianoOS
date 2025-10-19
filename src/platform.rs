use crate::console::ConsoleDevice;
use crate::devicetree::get_compatible_and_range;
use crate::devicetree::ParseDeviceTreeError;
use crate::driver::chardev::riscvsbi::RiscvSbi;
use crate::driver::chardev::uart16550::Uart16550Wrapper;
use crate::console::KernelConsole;
use crate::devicetree::parse_device_tree;
use crate::devicetree::Tree;
use crate::console::ConsoleType;
use crate::error::KernelError;

use alloc::boxed::Box;
use serde_device_tree::buildin::Node;
use spin::mutex::Mutex;

pub type BaseAddr = usize;

pub struct BoardInfo {
    pub cpu_num: Option<usize>,
    pub console: Option<(BaseAddr, ConsoleType)>
}

impl BoardInfo {
    pub const fn new() -> BoardInfo{
        BoardInfo {
            cpu_num: None,
            console: None
        }
    }
}

pub struct BoardDevice {
    pub console: Option<KernelConsole>
}

impl BoardDevice {
    pub const fn new() -> Self {
        BoardDevice { 
            console: None
        }
    }
}

pub struct Platform {
    pub board_info: BoardInfo,
    pub board_device: BoardDevice
}

impl Platform {
    const fn new() -> Self {
        Platform {
            board_info: BoardInfo::new(),
            board_device: BoardDevice::new()
        }
    }

    pub fn init(&mut self, dtb_addr: usize) -> Result<(), KernelError>{
        let dtb = parse_device_tree(dtb_addr)
            .unwrap_or_else(|_| panic!("parse dtb error"))
            .share();
        let root: serde_device_tree::buildin::Node = serde_device_tree::from_raw_mut(&dtb)
            .unwrap_or_else(|_| panic!("deserialze dtb fail"));
        let tree: Tree = root.deserialize();

        self.init_board_info(&tree, &root)?;

        self.init_board_device();

        Ok(())
    }

    fn init_board_device(&mut self) {
        self.board_device.console = self.init_console();
    }

    fn init_console(&mut self) -> Option<KernelConsole> {
        let Some((base, console_type)) = self.board_info.console else {
            return None;
        };
        let console: Box<dyn ConsoleDevice> = match console_type {
            ConsoleType::Uart16550U8 => Box::new(Uart16550Wrapper::<u8>::new(base)),
            ConsoleType::Uart16550U32 => Box::new(Uart16550Wrapper::<u32>::new(base)),
            ConsoleType::RiscvSbi => Box::new(RiscvSbi)
        };
        Some(KernelConsole::new(Mutex::new(console)))
    }

    fn init_board_info(&mut self, tree: &Tree, root: &Node) -> Result<(), ParseDeviceTreeError>{
        self.board_info.cpu_num = Some(tree.cpus.cpu.len());
        self.board_info.console = self.init_console_info(root)?;
        Ok(())
    }

    fn init_console_info(
        &mut self, 
        root: &Node
    ) -> Result<Option<(BaseAddr, ConsoleType)>, ParseDeviceTreeError> {
        let Some(stdout_path) = root.chosen_stdout_path() else {
            return Err(ParseDeviceTreeError::NoStdout)
        };
        let Some(stdout_node) = root.find(stdout_path) else {
            return Err(ParseDeviceTreeError::NoConsole)
        };
        let Some((compat, reg)) = get_compatible_and_range(&stdout_node) else {
            return Err(ParseDeviceTreeError::NoCompatOrRange)
        };
        Ok(compat
                .iter()
                .find_map(|dev| ConsoleType::compatible(dev))
                .map(|ctype| (reg.start, ctype))
        )
    }
    
}

pub static mut PLATFORM: Platform = Platform::new();