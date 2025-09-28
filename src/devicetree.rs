use serde_device_tree::{
    Dtb, DtbPtr,
    buildin::{NodeSeq, Reg, StrSeq},
};
use serde::Deserialize;

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
pub struct Cpu<'a> {
    /// RISC-V ISA extensions supported by this CPU.
    #[serde(rename = "riscv,isa-extensions")]
    pub isa_extensions: Option<StrSeq<'a>>,
    #[serde(rename = "riscv,isa")]
    pub isa: Option<StrSeq<'a>>,
    /// CPU register information.
    pub reg: Reg<'a>,
}

/// Generic device node information.
#[allow(unused)]
#[derive(Deserialize, Debug)]
pub struct Device<'a> {
    /// Device register information.
    pub reg: Reg<'a>,
}

/// Memory range.
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Memory<'a> {
    pub reg: Reg<'a>,
}

/// Errors that can occur during device tree parsing.
pub enum ParseDeviceTreeError {
    /// Invalid device tree format.
    Format,
}

pub fn parse_device_tree(opaque: usize) -> Result<Dtb, ParseDeviceTreeError> {
    let Ok(ptr) = DtbPtr::from_raw(opaque as *mut _) else {
        return Err(ParseDeviceTreeError::Format);
    };
    let dtb = Dtb::from(ptr);
    Ok(dtb)
}