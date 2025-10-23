use std::{env, path::PathBuf};
use std::fs;

fn main() {
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let ld = &out.join("linker.ld");
    let arch = std::env::var("TARGET");

    let _ = match arch.as_ref().unwrap().as_str() {
        "riscv64gc-unknown-none-elf" => fs::write(ld, RISCV_LINKER_SCRIPT),
        "loongarch64-unknown-none" => unimplemented!(),
        _ => panic!("
            Unsupported ARCH triple={}. 
            Use 'riscv64gc-unknown-none-elf' or 'loongarch64-unknown-none'", arch.unwrap())
    };
    std::fs::write(ld, RISCV_LINKER_SCRIPT).unwrap();

    println!("cargo:rustc-link-arg=-T{}", ld.display());
    println!("cargo:rustc-link-arg={}", "-Map=/tmp/pianoOSMap.map");
    println!("cargo:rustc-link-search={}", out.display());
}

const RISCV_LINKER_SCRIPT: &[u8] = b"
OUTPUT_ARCH(riscv)
ENTRY(_start)
BASE_ADDRESS = 0x80200000;
SECTIONS
{
    . = BASE_ADDRESS;
    skernel = .;
    
    stext = .;
    .text : {
        *(.text.entry)
        *(.text .text.*)
    }
    . = ALIGN(4K);
    etext = .;

    srodata = .;
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }
    . = ALIGN(4K);
    erodata = .;

    sdata = .;
    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }
    . = ALIGN(4K);
    edata = .;

    .bss : {
        sstack = .;
        *(.bss.stack)
        estack = .;
        sheap = .;
        *(.bss.heap)
        eheap = .;
        sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    }
    . = ALIGN(4K);
    ebss = .;

    ekernel = .;
    /DISCARD/ : {
        *(.eh_frame)
    }
}
";
