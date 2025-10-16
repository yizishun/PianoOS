use crate::driver::chardev::riscvsbi::RiscvSbi;
use crate::driver::chardev::uart16550::Uart16550Wrapper;
use crate::console::KernelConsole;
use crate::devicetree::parse_device_tree;
use crate::devicetree::Tree;
use crate::console::ConsoleType;

use alloc::boxed::Box;
use spin::Mutex;

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

    pub fn init(&mut self, dtb_addr: usize) {
        let dtb = parse_device_tree(dtb_addr)
            .unwrap_or_else(|_| panic!("parse dtb error"))
            .share();
        let root: serde_device_tree::buildin::Node = serde_device_tree::from_raw_mut(&dtb)
            .unwrap_or_else(|_| panic!("deserialze dtb fail"));
        let tree: Tree = root.deserialize();
        self.board_info.cpu_num = Some(tree.cpus.cpu.len());

        //TODO: use Uart16550 temporily
        let base = 0x04140000;
        //let base = 0x10000000;
        let console_type = ConsoleType::RiscvSbi;
        //let console_type = ConsoleType::Uart16550U32;
        //let console = Uart16550Wrapper::<u32>::new(base);
        let console = RiscvSbi;
        //let console = RiscvSbi;
        self.board_info.console = Some((base, console_type));
        self.board_device.console = Some(
            KernelConsole::new(
                Mutex::new(
                    Box::new(
                        console
                    )
                )
            )
        )
    }
    
}

pub static mut PLATFORM: Platform = Platform::new();