# CH1 应用程序与基本执行环境

## 

## 应用程序执行环境与平台支持

首先我第一次知道三元组的概念，比如

> `x86_64-unknown-linux-gnu`，其中 CPU 架构是 x86_64，CPU 厂商是 unknown，操作系统是 linux，运行时库是 GNU libc
> 
> `riscv64gc-unknown-none-elf` 目标平台。这其中的 CPU 架构是 riscv64gc ，CPU厂商是 unknown ，操作系统是 none ， elf 表示没有标准的运行时库（表明没有任何系统调用的封装支持），但可以生成 ELF 格式的执行程序

即是 `CPU架构-CPU厂商-操作系统-运行时库`这样的划分

然后rust的std库需要依赖操作系统，但是core库不需要依赖操作系统，所以第一步应该是移植到core库上

## 移除标准库依赖

### 安装rust toolchains

首先根据教程`rustup target add riscv64gc-unknown-none-elf` 首先通过rustup [^1] 安装这个平台的toolchains，这其实就类似于c里面安装riscv64-unknown-elf-gcc一样，然后修改cargo的配置，让其对于这个package使用这个toolchains，需要用.cargo/config.toml文件

### 使用Core而不是Std

但是现在依然会报错，因为rust编译器依然会默认从std中拿println的实现，所以需要使用Attributes [^2]来告诉编译器，即使用`#![no_std]` 

*(在vscode中使用no_std时rust-analyzer会出现问题，详见 [issue](https://github.com/rust-lang/rust-analyzer/issues/3297))*

 *(在vscode中使用rust-analyzer pre-release版本的时候，checkOnSave选项被改成check,上述issue中的名称需要改变)*

### 提供默认的Panic实现

由于在core中没有提供对panic的默认实现（估计是panic的实现需要打印内容，但是他不知道怎么使用当前os的syscall），所以我们需要给一个默认实现

使用`#[panic_handler]`来标记一个具有`fn(&PanicInfo) -> !`函数签名的函数，即可为编译器提供实现，此时使用loop来做一个简单实现

### 移除 main 函数

main函数也需要std,没有std的我们只能自己定义__start，所以先使用`#![no_main]`然后编译

### 分析

有一个轮椅项目，[cargo-binutils](https://crates.io/crates/cargo-binutils)他是相当于llvm的那些tool的proxy，但是他和cargo联系的比较紧密，他能有两种用法，一个是类似与`rust-$tool ${args[@]}`一个是直接用cargo,例子为`cargo objdump --release -- --disassemble --no-show-raw-insn` 

```shell
> cargo objdump -- -S 
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.01s

os:     file format elf64-littleriscv
```

发现是一个空程序

## 内核第一条指令（基础篇）

### qemu-system-riscv64

像是verilator模拟fpga之类的一样，qemu也在模拟真实的计算机板卡，（甚至已经预料到操作系统比赛前期疯狂用qemu,然后上板出问题了，有阴影了属于是），是前期开发必不可少的重要工具

然后先了解他的几个重要选项

```shell
-M/--machine： 选择模拟的board，包括CPU,SoC,板上资源。一般选择virt，而且不同板子在rv上差异较大
-bios：CPU的firmware/bootloader。一般选择rustsbi,默认是opensbi
-nographic：表示模拟器不需要提供图形界面，而只需要对外输出字符流。
-device loader：使用loader这个奇怪的device能直接让qemu帮忙将某个bin文件做加载到某个特定地址
```

然后qemu的启动流程也比较简单，首先0x80000000开始执行rustsbi,然后rustsbi约定好会跳到0x80200000处开始执行kernel，然后由于暂时rustsbi不执行加载工作，所以使用-device loader自动加载内核，但是其实我之前有一个不知道的一点，就是qemu会先到自己内部的一些代码，即0x1000，做一些工作之后再跳到0x80000000

## 内核第一条指令（实践篇）

首先编写第一条指令，即一个asm文件，然后使用`global_asm!(include_str!("entry.asm"));`导入，为此我还去学了一下rust的macro（macro主要处理source code,有两种macro,一种是匹配source code，做出相应处理，另一种是输出source code）

之后`cargo objdump -- -S`，发现有指令，但是链接到的地址不对，需要自己写链接脚本，为了让rustc能使用我们自己编写的链接脚本，需要给他传参数，[rustc Command-line Arguments](https://doc.rust-lang.org/beta/rustc/command-line-arguments.html#command-line-arguments) ，然后由于是cargo托管编译，所以需要还需要改cargo的config.toml

之后使用qemu，并使用gdb远程连接做调试

## 为内核支持函数调用（即创造boot stack）

感觉看还是学到了一些东西的

函数调用约定，即calling convention，是ISA和编程语言共同决定的，比如说RV64+C是一套calling convention，而RV64+Rust又是另一套，然后在Rust中写extern "C"，实际上是告诉编译器，这个函数是用C的ABI，但是看起来我搜了一下，Rust中的calling convention是unstable的，是internal的

函数约定主要规范了

- 各种寄存器在函数调用时的用途

- 寄存器被谁保存

rCore选择将内核的启动栈帧放到.bss段里面，但是不对其做初始化

之后其实直接设置好sp就可以直接跳到rust code里面了其实，毕竟写rust总比写asm要好

在rust code中需要做一个清空bss段的操作，最主要的是利用了rust的两个特性，[Accessing or Modifying a Mutable Static Variable](https://doc.rust-lang.org/book/ch20-01-unsafe-rust.html?highlight=static#accessing-or-modifying-a-mutable-static-variable) rust中的statice var就是常说的全局变量和[Dereferencing a Raw Pointer](https://doc.rust-lang.org/book/ch20-01-unsafe-rust.html?highlight=static#dereferencing-a-raw-pointer) 来直接操作内存地址，都是rust unsafe部分

## 基于 SBI 服务完成输出和关机

为此，去看了一眼SBI的[Spec](https://github.com/riscv-non-isa/riscv-sbi-doc)，sbi我认为最主要的意义就是让kernel变得更可移植了

定义了一套m mode和s mode下的规范，狭义来讲就是calling convention，他们通过ecall进行控制转移，a6+a7传递需要调用的函数，a0-a5传参数，a0-a1返回值，也有extention系统

看起来，文档上写的[sbi_rt](https://github.com/rustsbi/sbi-rt)已经停止维护了，我发现他被移到了rustsbi仓库中，我打算直接使用git来引用他，即`sbi-rt = { git = "https://github.com/rustsbi/rustsbi", package = "sbi-rt" }`，使用`cargo doc -p sbi-rt --open`可以本地打开他的doc，并且使用git的版本

然后文档使用`console_putchar`但是他是deprecated的，但是其实sbi ch12已经有新的extention了，即`console_write_byte`

```rust
pub fn console_putchar(c: usize) {
    c
    .to_le_bytes()
    .iter()
    .for_each( |c_bytes| {
            sbi_rt::console_write_byte(*c_bytes);
        }
    );
}
```

然后来拆解一下这个macro

```rust
#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
    }
}
```

首先要了解format_args!宏，他应该是parse了一个patern,然后变成了一个`fmt::Arguments`类型，然后传给print，`fmt: literal (, (arg: tt)+)?`感觉就像是`"{} {}", 1, 2`这种pattern，所以感觉理解是啥意思没啥问题，但是让我写还是有些细节不太清楚

## 基于 SBI 实现sleep

> ** 实现一个基于rcore/ucore tutorial的应用程序C，用sleep系统调用睡眠5秒（in rcore/ucore tutorial v3: Branch ch1）

 睡（

感觉这个应用程序的说法有歧义呀，应该他的意思就是在kernel上即S mode上实现，而不是广义的用户程序

那就是调用sbi啰，直接去快速定位手册，有点找不到

-----

### 测试

在看手册之前，我想给我的os加一个test（不然rust的模板ci过不去），主要分为单元测试和集成测试，我暂时也只需要单元测试，但是我发现一件事，我几乎现在所有的函数都是impure function，即会产生副作用的函数，比如说sleep这件事，你根本没有办法测他，一般来说，只能通过将impure func中pure的部分提取出来，从而测试pure部分的代码，但是我现在pure部分的代码过少，所以现在的测试没有什么意义，我暂时也只为他做一个占位符

然后报错`can't find crate for test`，是因为test需要std，但是riscv64gc-unknown-none-elf没有std，所以就很麻烦，只能在内核中自己写测试框架，那就先搁置了

-----

找手册，找到了几个疑似的，一个是ch6 time，一个是ch11 pmu，因为有性能计数器，我想是不是有时钟相关的，一个是ch13 susp，暂停整个系统，类似la的idle，等待时钟中断（或其他中断）的发生，（其实还有一种可能，就是riscv的s mode有一些寄存器能直接读出时间）

思路感觉是首先set timer interrupt，在5s之后，然后susp暂停整个系统，最后恢复的时候跳到函数的末尾

 在之前，我想看看各个寄存器的值，一种方法是用qemu直接拿值，一种方法是在rust中用csrr(pseudo CSRRS rd, csr, x0)，然后我打算还是用第二种来适配多种平台，但是如下会编译错误

```rust
pub fn read_csr(csrNum: i16) -> i64{
    let read_value: i64;
    unsafe {
        asm!(
            "csrr {0}, {csrNum}",
            out(reg) read_value
        )
    }
    read_value
}
```

这是因为csrr中的csrNum必须在编译时完全确定，而不支持传参实现，于是不使用函数，使用macro（其实这里用proc-macro会更好我感觉，但是我有点不太会用）

```rust
#[macro_export]
macro_rules! read_csr {
    ($csrNum: expr) => {
        {
            use core::arch::asm;
            let mut read_value: i64;
            unsafe {
                asm!(
                    "csrr {0}, {1}",
                    out(reg) read_value,
                    const $csrNum
                )
            }
            read_value
        }
    };
}
```

之后如果传入一个非法或者没有权限的csrNum就会导致qemu卡住，用gdb调试才发现的（gdb可以添加`-ex 'layout asm' -tui`，来获得更好的调试体验）

然后发现mtvec貌似在s mode下连读都读不了，于是还是用qemu吧，貌似用info reg可以看到所有寄存器，包括mtvec，或者gdb也可以看，gdb需要使用`info all-registers`，信息更详尽

（之后发现有一个叫做riscv的crate，里面也提供了一个read_csr的方法，但是他是用concat构造这条指令的，然后我还是打算用他的实现）

然后之后需要知道当前硬件的频率，看评论区是10mhz

10mhz换算成每周期1.0e-7s，说明1s需要1e7个周期，5s就是5e7个周期

然后貌似可以了

```rust
pub fn sleep(sec: i32) {
    let time_start = time::read();
    let time_end = time_start + ( FREQUNCY * 100_0000 * sec ) as usize;
    sbi_rt::set_timer(time_end as u64);
    riscv::asm::wfi();
}
```

差不多正好5s

[^1]: rustup是The Rust tool chain installer

[^2]: [Attributes](https://dhghomon.github.io/easy_rust/Chapter_52.html#attributes)其可以控制编译器的一些行为，使用#控制下一个语句，而#!控制整个文件
