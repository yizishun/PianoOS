use std::fmt::Write;
use std::fs;
use std::fs::read_dir;
use std::{env, path::PathBuf};

fn main() {
        let arch = std::env::var("TARGET");
        let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
        let ld = &out.join("linker.ld");
        let app_data = &out.join("link_app.S");

        // choose linker script base on arch
        let _ = match arch.as_ref().unwrap().as_str() {
                "riscv64gc-unknown-none-elf" => fs::write(ld, RISCV_LINKER_SCRIPT),
                "loongarch64-unknown-none" => unimplemented!(),
                _ => panic!(
                            "
            Unsupported ARCH triple={}. 
            Use 'riscv64gc-unknown-none-elf' or 'loongarch64-unknown-none'",
                            arch.unwrap()
                ),
        };
        std::fs::write(ld, RISCV_LINKER_SCRIPT).unwrap();

        // create app data section asm files
        std::fs::write(app_data, insert_app_data()).unwrap();
        let mut build = cc::Build::new();
        build.compiler("riscv64-none-elf-gcc");
        build.file(app_data).flag("-mabi=lp64d").compile("app_data");

        println!("cargo:rustc-link-arg=-T{}", ld.display());
        println!("cargo:rustc-link-arg={}", "-Map=/tmp/pianoOSMap.map");
        println!("cargo:rustc-link-search={}", out.display());
}

static TARGET_PATH: &str = "../user/binary/";

fn insert_app_data() -> String {
        let mut f = String::new();
        let mut apps: Vec<_> =
                read_dir("../user/src/bin").unwrap()
                                           .into_iter()
                                           .map(|dir_entry| {
                                                   let mut name_with_ext = dir_entry.unwrap()
                                                                                    .file_name()
                                                                                    .into_string()
                                                                                    .unwrap();
                                                   name_with_ext.drain(name_with_ext.find('.')
                                                                                    .unwrap()
                                                                       ..name_with_ext.len());
                                                   name_with_ext
                                           })
                                           .collect();
        apps.sort();

        writeln!(
                 f,
                 r#"
    .align 3
    .section .data
    .global _num_app
_num_app:
    .quad {}"#,
                 apps.len()
        ).unwrap();

        for i in 0..apps.len() {
                writeln!(f, r#"    .quad app_{}_start"#, i).unwrap();
        }
        writeln!(f, r#"    .quad app_{}_end"#, apps.len() - 1).unwrap();

        for (idx, app) in apps.iter().enumerate() {
                println!("app_{}: {}", idx, app);
                writeln!(
                         f,
                         r#"
    .section .data
    .global app_{0}_start
    .global app_{0}_end
app_{0}_start:
    .incbin "{2}{1}.bin"
app_{0}_end:"#,
                         idx, app, TARGET_PATH
                ).unwrap();
        }
        f
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
