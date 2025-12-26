use std::fs;
use std::{env, path::PathBuf};

fn main() {
        let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
        let ld = &out.join("linker.ld");
        let arch = std::env::var("TARGET");

        let _ = match arch.as_ref().unwrap().as_str() {
                "riscv64gc-unknown-none-elf" => fs::write(ld, RISCV_LINKER_SCRIPT),
                "loongarch64-unknown-none" => unimplemented!(),
                _ => { println!(
                                    "
                    Unsupported ARCH triple={}.
                    Use 'riscv64gc-unknown-none-elf' or 'loongarch64-unknown-none'",
                                    arch.unwrap()
                        );
                    fs::write(ld, RISCV_LINKER_SCRIPT)
                }
        };
        std::fs::write(ld, RISCV_LINKER_SCRIPT).unwrap();

        println!("cargo:rustc-link-arg=-T{}", ld.display());
        println!("cargo:rustc-link-arg={}", "-Map=/tmp/UserMap.map");
        println!("cargo:rustc-link-search={}", out.display());
}

const RISCV_LINKER_SCRIPT: &[u8] = b"
OUTPUT_ARCH(riscv)
ENTRY(_start)

SECTIONS
{
    . = 0x1000;
    . = ALIGN(4K);
    .text : {
        *(.text.entry)
        *(.text .text.*)
    }
    . = ALIGN(4K);
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }
    . = ALIGN(4K);
    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }
    . = ALIGN(4K);
    .bss : {
        start_bss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        end_bss = .;
    }
    /DISCARD/ : {
        *(.eh_frame)
        *(.debug*)
    }
}
";
