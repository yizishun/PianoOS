use crate::devicetree::parse_device_tree;
use crate::devicetree::Tree;

pub struct Platform {
    pub board_info: BoardInfo
}

pub struct BoardInfo {
    pub cpu_num: usize
}

impl Platform {
    const fn new() -> Self {
        Platform {
            board_info: BoardInfo { cpu_num: (0) }
        }
    }

    pub fn init(&mut self, dtb_addr: usize) {
        let dtb = parse_device_tree(dtb_addr)
            .unwrap_or_else(|_| panic!("parse dtb error"))
            .share();
        let root: serde_device_tree::buildin::Node = serde_device_tree::from_raw_mut(&dtb)
            .unwrap_or_else(|_| panic!("deserialze dtb fail"));
        let tree: Tree = root.deserialize();
        self.board_info.cpu_num = tree.cpus.cpu.len()
    }
    
}

pub static mut PLATFORM: Platform = Platform::new();