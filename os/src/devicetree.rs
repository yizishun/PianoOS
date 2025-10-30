use core::ops::Range;
use serde::Deserialize;
use serde_device_tree::{
        Dtb, DtbPtr,
        buildin::{Node, NodeSeq, Reg, StrSeq},
};

/// Root device tree structure containing system information.
#[derive(Deserialize)]
pub struct Tree<'a> {
        /// Optional model name string.
        pub model: Option<StrSeq<'a>>,
        /// Memory information.
        pub memory: NodeSeq<'a>,
        /// CPU information.
        pub cpus: Cpus<'a>,
}

/// CPU information container.
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Cpus<'a> {
        /// Sequence of CPU nodes.
        pub cpu: NodeSeq<'a>,
}

/// Individual CPU node information.
#[derive(Deserialize, Debug)]
#[allow(unused)]
pub struct Cpu {}

/// Memory range.
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Memory<'a> {
        pub reg: Reg<'a>,
}

/// Errors that can occur during device tree parsing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ParseDeviceTreeError {
        /// Invalid device tree format.
        Format,
        NoStdout,
        NoConsole,
        NoCompatOrRange,
}

pub fn get_compatible_and_range<'de>(node: &Node) -> Option<(StrSeq<'de>, Range<usize>)> {
        let compatible = node.get_prop("compatible")
                             .map(|prop_item| prop_item.deserialize::<StrSeq<'de>>());
        let regs = node.get_prop("reg")
                       .map(|prop_item| {
                               let reg = prop_item.deserialize::<serde_device_tree::buildin::Reg>();
                               if let Some(range) = reg.iter().next() {
                                       return Some(range);
                               }
                               None
                       })
                       .map_or_else(|| None, |v| v);
        if let Some(compatible) = compatible {
                if let Some(regs) = regs {
                        Some((compatible, regs.0))
                } else {
                        None
                }
        } else {
                None
        }
}

pub fn parse_device_tree(opaque: usize) -> Result<Dtb, ParseDeviceTreeError> {
        // this will also check the validity of the dtb header
        let Ok(ptr) = DtbPtr::from_raw(opaque as *mut _) else {
                return Err(ParseDeviceTreeError::Format);
        };
        let dtb = Dtb::from(ptr);
        Ok(dtb)
}
