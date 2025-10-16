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

## 彩色化LOG

首先是ansi转义序列，虽然之前也听过，但是回想起来总是模糊，看到了一个比较好的知乎介绍https://zhuanlan.zhihu.com/p/570148970，才知道m原来是一个函数

基本上要做的就是两件事

- 根据log进行输出等级控制

- 颜色输出

还有两件事是实现完上述之后能做到

- 关闭所有输出

- 彩色输出os的内存布局

首先完成最简单的颜色输出

### 颜色输出

直接使用和println一样的实现方法

```rust
#[macro_export]
macro_rules! info {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!("\x1b[0;34m", "[INFO] ", $fmt, "\x1b[0m", "\n") $(, $($arg)+)?));
    };
}
```

但是看到他的实现中有打印出当前打印这个语句的hart id，甚至推荐我们打印线程信息

我查手册发现有一个csr叫做mhartid，但是在s mode下访问不了，查看有没有相关的sbi，有的，就叫做`sbi_get_marchid`，（那现在是直接在每次info的时候都读出hartid吗，这会不会造成性能损失呢？）

```rust
#[macro_export]
macro_rules! info {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        let hart_id = sbi::get_hartid();
        $crate::console::print(
            format_args!(
                concat!("\x1b[0;34m", "[INFO][{}] ", $fmt, "\x1b[0m", "\n") , hart_id $(, $($arg)+)?
            )
        );
    };
}
```

不太会用rust的fmt工具，感觉有点丑陋

但是error，info，trace...这么多，相当于需要重复上述代码多次，并且容易写错，于是抽象出一个共通的宏

```rust
#![macro_use]
macro_rules! log_message {
    ($level: literal, $fmt: literal $(, $($arg: tt)+)?) => {
        let ansi_color = match $level {
            "INFO"  => "\x1b[0;34m",
            "ERROR" => "\x1b[0;31m",
            "WARN"  => "\x1b[0;93m",
            "DEBUG" => "\x1b[0;32m",
            "TRACE" => "\x1b[0;90m",
            _       => "\x1b[0m"
        };
        let hart_id = sbi::get_hartid();
        $crate::console::print(
            format_args!(
                concat!("{}", "[{:<5}][{:<2}] ", $fmt, "\x1b[0m", "\n") , ansi_color, $level, hart_id $(, $($arg)+)?
            )
        );
    };
}
#[macro_export]
macro_rules! info {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        log_message!("INFO", $fmt $(, $($arg)+)?);
    };
}
...
```

其中的`[{:<5}][{:<2}]`可以实现向左5对齐和向左2对齐

## log等级控制

 首先就是他是一个config，然后这个config是从外部传入，然后内部程序能接收到这个config，然后转化为内部能理解的结构，然后所有的输出都能读取这个结构，来判断是否需要输出

首先我要知道怎么从外部传入一个config，这里的外部传入是运行前传入，而不是编译时传入，所以不是kconfig那一套东西首先明确（即一定是运行时判断，而不是编译时判断）

所以打算使用command line arugment，但是cli是基于os的，所以需要std，就很麻烦，现在运行时只有非常简陋的运行时环境，这意味着你甚至没有办法从文件中读config，就很麻烦，所以就更不太可能从command line拿值

头脑有点混乱，理清一下，我是在一个真实硬件上运行的，什么cli最多传到qemu，但是如果是一个真实硬件呢，那就cli肯定传不进来，我的外部环境只有硬件和抱团取暖的sbi，所以真实的外部环境是拨码选择（

打算看一眼参考实现

参考实现使用`core::option_env`宏，在complie time拿env变量，然后展开成Some()，然后他还使用了log crate，但是impl了某些方法，（我发现rust真的好多crate呀，基本很多基本功能都有crate，但是c肯定没有这么多东西，所以我的第一反应才是造轮子）

这个crate需要自己实现一个logger，然后使用set_logger来使用这个logger

然后成功了

这个成功了，那关闭所有输入就是LOG=OFF

然后就是输出os的内存布局

## 输出内存布局

很奇怪，输出的地址不对

```rust
unsafe extern "C" {
    static skernel: usize;
    static stext: usize;
    static etext: usize;
}

fn print_kernel_mem() {
    unsafe {
        info!("kernel base = {:#x}", skernel);
        info!(".text: [{:#x}, {:#x}]", stext, etext);
    }
}
```

发现把skernel的type改成函数fn就可以正确打印了，很奇怪

最后发现原因是如果声明为usize，那他的值会是这个地址的值，而不是地址，需要用`{:p}`+`&stext`来打印

所以如果要直接获取这个符号的地址，使用fn确实是一个很好的实践

之后发现了一个rustsbi-qemu上的一个写法错误

```rust
    // 全局初始化过程
    if GENESIS.swap(false, Ordering::AcqRel) {
        extern "C" {
            static mut sbss: u64;
            static mut ebss: u64;
        }
        unsafe {
            let mut ptr = sbss as *mut u64;
            let end = ebss as *mut u64;
            while ptr < end {
                ptr.write_volatile(0);
                ptr = ptr.offset(1);
            }
        }
```

这个ptr和end都是sbss和ebss的值而不是地址，需要改成`let mut ptr = &raw mut sbss;`发了一个pr，[fix: use valid ptr instead of its value by yizishun · Pull Request #68 · rustsbi/rustsbi-qemu · GitHub](https://github.com/rustsbi/rustsbi-qemu/pull/68)，但是问了罗师傅，罗师傅跟他们组长说了之后，就把这个repo archived了，emmm，那我也应该转一下，转到rustsbi/rustsbi了

## 将rustsbi-qemu转到rustsbi

正好的milk-v duo到了，也需要rustsbin

经过我的研究，用法如下

```shell
git clone git@github.com:rustsbi/rustsbi.git
cd rustsbi
cargo prototyper --jump
```

然后把生成出来的bin文件丢到之前的bootloader部分，就可以正常启动啦

看起来他的option有三种，jump，payload和dynamic

## challenge: 支持多核，实现多个核的 boot

这个貌似对我有点难度的，之前从没接触过多核，多核编程多核设计，对我都是空白的，所以能研究多少就研究多少吧

打算先读一下rustsbi启动时候是如何处理多核的，因为从启动信息来看，他打印了设备有多少核

```rust
#[unsafe(naked)]
#[unsafe(link_section = ".text.entry")]
#[unsafe(export_name = "_start")]
unsafe extern "C" fn start() -> ! {
    naked_asm!(
        ".option arch, +a",
        // 1. Turn off interrupt.
        "
        csrw    mie, zero",
        // 2. Initialize programming language runtime.
        // only clear bss if hartid matches preferred boot hart id.
        // Race
        "
            lla      t0, 6f
            li       t1, 1
            amoadd.w t0, t1, 0(t0)
            bnez     t0, 4f
            call     {relocation_update}",
        // 3. Boot hart clear bss segment.
        "1:
            lla     t0, sbi_bss_start
            lla     t1, sbi_bss_end",
        "2:
            bgeu    t0, t1, 3f
            sd      zero, 0(t0)
            addi    t0, t0, 8
            j       2b",
        // 3.1 Boot hart set bss ready signal.
        "3:
            lla     t0, 7f
            li      t1, 1
            amoadd.w t0, t1, 0(t0)
            j       5f",
        // 3.2 Other harts are waiting for bss ready signal.
        "4:
            lla     t0, 7f
            lw      t0, 0(t0)
            beqz    t0, 4b",
        // 4. Prepare stack for each hart.
        "5:
            call    {locate_stack}
            call    {main}
            csrw    mscratch, sp
            j       {hart_boot}
            .balign  4",
        "6:", // boot hart race signal.
        "  .word    0",
        "7:", // bss ready signal.
        "  .word    0",
        relocation_update = sym relocation_update,
        locate_stack = sym trap_stack::locate,
        main         = sym rust_main,
        hart_boot    = sym trap::boot::boot,
    )
}
```

首先他直接使用extern “C”+各种属性来将这个函数作为一个c函数全局导出（我感觉我也能这么写）

首先是naked attr，可以参考（https://blog.rust-lang.org/2025/07/03/stabilizing-naked-functions/），虽然不是手册，但是写的比较易懂，然后上面说extern “C”之类的函数一般没有办法被rust代码call，因为他没有一个确定的调用规范，所以一般在另一些asm中手动call

emmm，基本上看rustsbi的启动代码，就是让每一个核获得自己的stack，让第一个核做更多的事情（比如relocat一些代码）

然后查找sbi手册，发现了HSM扩展，就是对于多个hart进行管理，但是我没有找到在哪能知道有多少个hart，通过while循环来看sbi是否返回错误，判断出有8个hart，但是我才启动2个smp，感觉有点问题，应该不是这么做的，问gpt说要看sbi给的设备树，说a1寄存器会给设备树指针，感觉需要小心求证，于是开始看sbi的源码（因为手册上实在没有翻到）

源码上确实是这么说的，但是我始终没有找到对应的手册

sbi手册上说*The SBI specification doesn’t specify any method for hardware discovery. The supervisor software must rely on the other industry standard hardware discovery methods (i.e. Device Tree or ACPI) for that.* 然后我就不知道究竟去哪里找这个规范了

问了罗师傅，说可能在kernel的手册上定义了这个事情，但是大多还是约定俗成的，上述说的8个harts的问题可能是因为rustsbi的实现问题，然后我发现我获得当前hart的id的行为是错误的，应该是要获得mhartid而不是marchid，但是看起来这个没有纳入到规范确实有很多人有意见，有几个比较相关的讨论，https://github.com/riscv-non-isa/riscv-sbi-doc/issues/141，linux kernel的规范：https://www.kernel.org/doc/html/next/riscv/boot.html，

所以上述的的省流版就是rustsbi跳到kernel的时候，a0存放hartid，a1存放设备树地址，并且手册没有规范这件事情

然后rust_main要想拿到这两个参数，我想是不是需要将函数声明为extern "C"

然后我需要保存这两个参数，我一开始想的是用全局变量，但是我又发现，每一个hart都有可能覆盖这个全局变量，这意味着我至少需要n个这个全局变量

好在每个hart的寄存器是独立的，并且我可以找到一个叫做sscratch的寄存器，我打算每一个hart开辟一个空间保存他们诸如hartid这种信息，我打算学rustsbi定义一个最大的hart数量，然后就可以开始保存了，不然我根本没有办法call其他函数，但是我又发现一个事情，就是跳到kernel的时候，只有boot hart是start，这就意味着我一开始并不需要考虑很复杂的事情，基本上流程如下

- boot hart首先获得boot hartid+device tree地址（该把这个信息存在哪里比较符合rust风格呢？我现在就简单的存在全局变量里面了）

- boot hart开始清理bss段

- boot hart 做log系统初始化（optional）

- boot hart开始做设备树解析，找出有多少个cpu

- boot hart为所有cpu做环境的准备

- boot hart调用`sbi_hart_start`，让所有hart进入一个初始函数，并打印一些东西之后开始死循环

- boot hart做shutdown

parse device tree有点不会，打算先看看device tree的手册，然后再看一下rustsbi的实现

emm，打算抄袭一下rustsbi中解析设备树的实现（等我实现完我再看他的具体实现），但是使用时，遇到了

`error: no global memory allocator found but one is required; link to std or add `#[global_allocator]` to a static item that implements the GlobalAlloc trait`说我没有heap

直接提前读第四章：[Rust 中的动态内存分配 - rCore-Tutorial-Book-v3 3.6.0-alpha.1 文档](https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter4/1rust-dynamic-allocation.html?highlight=global)

## 真实硬件：milkv duo256m

在做challenge的时候新开了一个坑，用的是没有多hart boot的kernel

首先这个的手册在：https://milkv.io/docs/duo/getting-started/duo256m

有点麻烦的，我打算做成功之后做一个b站视频，感觉会有不错的流量

首先，这个板子的启动流程理论可以参考：[articles/20240329-duo-bootflow.md · unicornx/tblog - Gitee.com](https://gitee.com/unicornx/tblog/blob/master/articles/20240329-duo-bootflow.md) 

然后启动流程实践可以参考：[使用 Opensbi 引导自己的操作系统 - Duo - Milk-V Community](https://community.milkv.io/t/opensbi/681)

总而言之，这个板子启动时，首先是bootrom，他会加载fip.bin文件，首先加载fip.bin中的fsbl（bl2），然后bl2开始加载rustsbi（bl31），然后bl2开始加载我的kernel（bl33），然后就是跳转到rustsbi，然后就是普通的启动流程了

所以最关键的目的是获得custom的一个fip.bin文件

首先clone这个v2的sdk repo，cd进入，然后

```shell
> source ./device/milkv-duo256m-musl-riscv64-sd/boardconfig.sh 
> source ./build/envsetup_milkv.sh #（如果没有/bin/pwd,修改里面的硬编码路径）
Select a target to build:
1. milkv-duo-musl-riscv64-sd
2. milkv-duo256m-glibc-arm64-sd
3. milkv-duo256m-musl-riscv64-sd
4. milkv-duos-glibc-arm64-emmc
5. milkv-duos-glibc-arm64-sd
6. milkv-duos-musl-riscv64-emmc
7. milkv-duos-musl-riscv64-sd
Which would you like: 3
Target Top Config: /home/yzs/rcore/duo-buildroot-sdk-v2/build/boards/cv181x/sg2002_milkv_duo256m_musl_riscv64_sd/sg2002_milkv_duo256m_musl_riscv64_sd_defconfig
Target Board: milkv-duo256m-musl-riscv64-sd
Target Board Storage: sd
Target Board Config: /home/yzs/rcore/duo-buildroot-sdk-v2/device/target/boardconfig.sh
Target Board Type: duo256m
Target Image Config: /home/yzs/rcore/duo-buildroot-sdk-v2/device/target/genimage.cfg
Build tdl-sdk: 1
Output dir: /home/yzs/rcore/duo-buildroot-sdk-v2/install/soc_sg2002_milkv_duo256m_musl_riscv64_sd

> defconfig sg2002_milkv_duo256m_musl_riscv64_sd
 Run defconfig function 
Loaded configuration '/home/yzs/rcore/duo-buildroot-sdk-v2/build/boards/cv181x/sg2002_milkv_duo256m_musl_riscv64_sd/sg2002_milkv_duo256m_musl_riscv64_sd_defconfig'
No change to configuration in '.config'
Loaded configuration '.config'
No change to minimal configuration in '/home/yzs/rcore/duo-buildroot-sdk-v2/build/.defconfig'
~/rcore/duo-buildroot-sdk-v2/build ~/rcore/duo-buildroot-sdk-v2
~/rcore/duo-buildroot-sdk-v2

====== Environment Variables ======= 

  PROJECT: sg2002_milkv_duo256m_musl_riscv64_sd, DDR_CFG=ddr3_1866_x16
  CHIP_ARCH: CV181X, DEBUG=0
  SDK VERSION: musl_riscv64, RPC=0
  BOARD TYPE: duo256m
  ATF options: ATF_KEY_SEL=default, BL32=1
  Linux source folder:linux_5.10, Uboot source folder: u-boot-2021.10
  CROSS_COMPILE_PREFIX: riscv64-unknown-linux-musl-
  ENABLE_BOOTLOGO: 0
  Flash layout xml: /home/yzs/rcore/duo-buildroot-sdk-v2/build/boards/cv181x/sg2002_milkv_duo256m_musl_riscv64_sd/partition/partition_sd.xml
  Target Top Config: /home/yzs/rcore/duo-buildroot-sdk-v2/build/boards/cv181x/sg2002_milkv_duo256m_musl_riscv64_sd/sg2002_milkv_duo256m_musl_riscv64_sd_defconfig
  Sensor tuning bin: sms_sc2336
  Output path: /home/yzs/rcore/duo-buildroot-sdk-v2/install/soc_sg2002_milkv_duo256m_musl_riscv64_sd


> 
```

但是运行下一步时报错很麻烦，打算使用他的docker

```bash
set -euo pipefail    

echo "== Check binfmt mount =="
mount | grep -q binfmt_misc || sudo mount -t binfmt_misc binfmt_misc /proc/sys/fs/binfmt_misc
ls /proc/sys/fs/binfmt_misc || true

echo "== Reinstall emulators =="
docker run --privileged --rm tonistiigi/binfmt --uninstall qemu-* || true
docker run --privileged --rm tonistiigi/binfmt --install amd64,riscv64,arm64
docker run --privileged --rm tonistiigi/binfmt

echo "== Pull & run test images =="
docker pull --platform=linux/amd64 debian:stable-slim
docker run --rm --platform=linux/amd64 debian:stable-slim uname -m

docker pull --platform=linux/riscv64 debian:stable-slim
docker run --rm --platform=linux/riscv64 debian:stable-slim uname -m

sudo docker run -it --name duodocker \                                                                                  
  --platform=linux/amd64 \
  -v "$(pwd)":/home/work \
  guttatus314/milkv-duo:rust /bin/bash

sudo docker run -it --name duodocker \             
  --platform=linux/amd64 \
  -v "$(pwd)":/home/work \
  milkvtech/milkv-duo:latest /bin/bash
```

不知道为啥，我用docker来build也会在uboot编译报错，于是我直接更改了fip_v2.mk文件，让他不依赖uboot

```diff
diff --git a/build/scripts/fip_v2.mk b/build/scripts/fip_v2.mk
index 9a352403d..544fb5ea3 100644
--- a/build/scripts/fip_v2.mk
+++ b/build/scripts/fip_v2.mk
@@ -11,10 +11,10 @@ opensbi-clean:

 FSBL_OUTPUT_PATH = ${FSBL_PATH}/build/${PROJECT_FULLNAME}
 ifeq ($(call qstrip,${CONFIG_ARCH}),riscv)
-fsbl-build: opensbi
+fsbl-build: 
 endif
 ifeq (${CONFIG_ENABLE_FREERTOS},y)
-fsbl-build: rtos
+fsbl-build: 
 fsbl%: export BLCP_2ND_PATH=${FREERTOS_PATH}/cvitek/install/bin/cvirtos.bin
 fsbl%: export RTOS_DUMP_PRINT_ENABLE=$(CONFIG_ENABLE_RTOS_DUMP_PRINT)
 fsbl%: export RTOS_DUMP_PRINT_SZ_IDX=$(CONFIG_DUMP_PRINT_SZ_IDX)
@@ -39,14 +39,14 @@ fsbl%: export LOG_LEVEL=2
 endif

 ifeq (${CONFIG_ENABLE_BOOT0},y)
-fsbl-build: u-boot-build memory-map
+fsbl-build: memory-map
        $(call print_target)
        ${Q}mkdir -p ${FSBL_PATH}/build
        ${Q}ln -snrf -t ${FSBL_PATH}/build ${CVI_BOARD_MEMMAP_H_PATH}
        ${Q}$(MAKE) -j${NPROC} -C ${FSBL_PATH} O=${FSBL_OUTPUT_PATH} LOG_LEVEL=${LOG_LEVEL}
        ${Q}cp ${FSBL_OUTPUT_PATH}/boot0 ${OUTPUT_DIR}/
 else
-fsbl-build: u-boot-build memory-map
+fsbl-build: memory-map
        $(call print_target)
        ${Q}mkdir -p ${FSBL_PATH}/build
        ${Q}ln -snrf -t ${FSBL_PATH}/build ${CVI_BOARD_MEMMAP_H_PATH}
```

然后重新运行build_fsbl就可以生成bl2啦

然后运行

```shell
./plat/cv181x/fiptool.py -v genfip \
        '/home/work/fsbl/build/sg2002_milkv_duo256m_musl_riscv64_sd/fip.bin' \
        --MONITOR_RUNADDR="0x0000000080000000" \
        --CHIP_CONF='/home/work/fsbl/build/sg2002_milkv_duo256m_musl_riscv64_sd/chip_conf.bin' \
        --NOR_INFO='FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF' \
        --NAND_INFO='00000000'\
        --BL2='/home/work/fsbl/build/sg2002_milkv_duo256m_musl_riscv64_sd/bl2.bin' \
        --MONITOR='../rustsbi-prototyper-payload.bin' \
        --LOADER_2ND='../PianoOS.bin' \
```

我的PiannoOS不符合bl33格式，所以我直接删掉了这一行，毕竟我这个payload rustsbi本身就要有把我的os加载的功能，然后成功生成fip.bin

```shell
./plat/cv181x/fiptool.py -v genfip \
        '/home/work/fsbl/build/sg2002_milkv_duo256m_musl_riscv64_sd/fip.bin' \
        --MONITOR_RUNADDR="0x80000000" \
        --CHIP_CONF='/home/work/fsbl/build/sg2002_milkv_duo256m_musl_riscv64_sd/chip_conf.bin' \
        --NOR_INFO='FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF' \
        --NAND_INFO='00000000'\
        --BL2='/home/work/fsbl/build/sg2002_milkv_duo256m_musl_riscv64_sd/bl2.bin' \
        --MONITOR='../opensbi/build/platform/generic/firmware/fw_dynamic.bin' \
        --LOADER_2ND='/home/work/u-boot-2021.10/build/sg2002_milkv_duo256m_musl_riscv64_sd/u-boot-raw.bin' \
        --compress='lzma'
```

```shell
./plat/cv180x/fiptool.py -v genfip \
    'build/sg2002_milkv_duo256m_musl_riscv64_sd/fip.bin' \
    --MONITOR_RUNADDR="0x0" \
    --CHIP_CONF='build/sg2002_milkv_duo256m_musl_riscv64_sd/chip_conf.bin' \
    --NOR_INFO='FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF' \
    --NAND_INFO='00000000'\
    --BL2='build/sg2002_milkv_duo256m_musl_riscv64_sd/bl2.bin' \
    --LOADER_2ND='./bl33.bin'
```

```shell
> sudo ./plat/cv181x/fiptool.py -v genfip \
        '/home/yzs/rcore/duo-buildroot-sdk-v2/fsbl/build/sg2002_milkv_duo256m_musl_riscv64_sd/fip.bin' \
        --MONITOR_RUNADDR="0x0000000080000000" \
        --CHIP_CONF='/home/yzs/rcore/duo-buildroot-sdk-v2/fsbl/build/sg2002_milkv_duo256m_musl_riscv64_sd/chip_conf.bin' \
        --NOR_INFO='FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF' \
        --NAND_INFO='00000000'\
        --BL2='/home/yzs/rcore/duo-buildroot-sdk-v2/fsbl/build/sg2002_milkv_duo256m_musl_riscv64_sd/bl2.bin' \
        --MONITOR='../rustsbi-prototyper-payload.bin'
```

```shell
. /home/work/fsbl/build/cv1812cp_milkv_duo256m_sd/blmacros.env && \
./plat/cv181x/fiptool.py -v genfip \
        './build/cv1812cp_milkv_duo256m_sd/fip.bin' \
        --MONITOR_RUNADDR="0x80000000" \
        --CHIP_CONF='./build/cv1812cp_milkv_duo256m_sd/chip_conf.bin' \
        --NOR_INFO='FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF' \
        --NAND_INFO='00000000'\
        --BL2='./build/cv1812cp_milkv_duo256m_sd/bl2.bin' \
        --BLCP_IMG_RUNADDR=0x05200200 \
        --BLCP_PARAM_LOADADDR=0 \
        --BLCP=test/empty.bin \
        --DDR_PARAM='test/cv181x/ddr_param.bin' \
        --MONITOR='../rustsbi/target/riscv64gc-unknown-none-elf/release/rustsbi-prototyper-dynamic.bin' \
        --LOADER_2ND='../u-boot-2021.10/build/cv1812cp_milkv_duo256m_sd/u-boot-raw.bin' \
        --compress='lzma'


./plat/cv181x/fiptool.py -v genfip         './build/cv1812cp_milkv_duo256m_sd/fip.bin'         --MONITOR_RUNADDR="0x80000000"         --CHIP_CONF='./build/cv1812cp_milkv_duo256m_sd/chip_conf.bin'         --NOR_INFO='FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF'         --NAND_INFO='00000000'        --BL2='./build/cv1812cp_milkv_duo256m_sd/bl2.bin'         --BLCP_IMG_RUNADDR=0x05200200         --BLCP_PARAM_LOADADDR=0         --BLCP=test/empty.bin         --DDR_PARAM='test/cv181x/ddr_param.bin'         --MONITOR='../rustsbi/target/riscv64gc-unknown-none-elf/release/rustsbi-prototyper-dynamic.bin'         --LOADER_2ND='../PianoOS.bin'         --compress='lzma'

./plat/cv181x/fiptool.py -v genfip \
        '/home/work/fsbl/build/cv1812cp_milkv_duo256m_sd/fip.bin' \
        --MONITOR_RUNADDR="0x80000000" \
        --CHIP_CONF='/home/work/fsbl/build/cv1812cp_milkv_duo256m_sd/chip_conf.bin' \
        --NOR_INFO='FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF' \
        --NAND_INFO='00000000'\
        --BL2='/home/work/fsbl/build/cv1812cp_milkv_duo256m_sd/bl2.bin' \
        --BLCP_IMG_RUNADDR=0x05200200 \
        --BLCP_PARAM_LOADADDR=0 \
        --BLCP=test/empty.bin \
        --DDR_PARAM='test/cv181x/ddr_param.bin' \
        --MONITOR='../opensbi/build/platform/generic/firmware/fw_dynamic.bin' \
        --LOADER_2ND='../PianoOS.bin' \
        --compress='lzma'
```

现在需要将他烧到tf卡上，然后boot，所以至少需要以下工具

- tf卡

- tf卡读卡器

- usb-ttl串口线

- 

买回来了，第一件事是将fip.bin烧到我的tf卡上

他貌似需要tf卡有特定的格式，最简单的方式就是下载他的一个img,然后用dd写入tf卡，他会把格式之类的东西全部写入，按理来说，改变他boot分区的fip.bin然后删掉其他文件就可以了，之后用他的img成功点亮板卡，但是串口始终没有输出，我怀疑是我的串口线的问题，因为好像没有白色的线，于是重新买了一个

重新买了一个就可以了，但是又有一个新的问题，就是没有办法输入，至少给的img可以输出到终端，但是没有办法输入命令

然后使用我自己的fip.bin，发现串口输出乱码

然后尝试使用设备树

```shell
cpp -P -nostdinc -undef -D__DTS__ -x assembler-with-cpp \
  -I ../../../default/dts/cv181x_riscv/ \
  -I ../../../default/dts/ \
  -I ../../../../../build/boards/default/dts/cv181x \
  -I ../../../../../build/output/sg2002_milkv_duo256m_musl_riscv64_sd \
  -I ../../../../../linux_5.10/include \
  sg2002_milkv_duo256m_musl_riscv64_sd.dts \
  > ./tmp_sg2002_preprocessed.dts
> dtc -I dts -O dtb \
  -o sg2002_milkv_duo256m_musl_riscv64_sd.dtb \
  ./tmp_sg2002_preprocessed.dts
> cargo prototyper --payload /home/yzs/rcore/PianoOS/target/riscv64gc-unknown-none-elf/release/PianoOS.bin --fdt /home/yzs/rcore/duo-buildroot-sdk-v2/build/boards/cv181x/sg2002_milkv_duo256m_musl_riscv64_sd/dts_riscv/sg2002_milkv_duo256m_musl_riscv64_sd.fdt
```

但是仍然失败，但是发现了bl2的源码

按理来说，我看懂bl2的源码就基本知道问题所在了

bl2对于一些定义是硬编码的，但是按理来说应该不会有问题

之后我发现fip.bin是需要签名的，但是签名之后依然失败

md我真的怀疑是硬件问题了，但是用别人的镜像又没问题，所以还是fip.bin的问题，我猜测还是bl2的问题，但是我不知道问题在哪

然后我打算重新编译一次（全部编译，包括uboot等），但是发现报错

```
  [TARGET] rtos 
cd /home/work/freertos/cvitek && ./build_cv181x.sh
RUN TYPE:  CVIRTOS
RUN_ARCH:  riscv64
/home/work/freertos/cvitek/build/arch /home/work/freertos/cvitek
cmake: error while loading shared libraries: libstdc++.so.6: failed to map segment from shared object
make: *** [scripts/rtos.mk:3: rtos] Error 127
```

这个报错疑似是电脑太垃圾了

打算使用我的另一台电脑编译，普通编译，可以运行到opensbi和uboot，但是我神奇的发现，bl2依然是乱码，这意味着bl2确实有问题，导致我没有办法把rustsbi移植上去（rustsbi也是乱码）

bl2是乱码，但是opensbi和uboot不是，这意味着opensbi和uboot使用了编译时的设备树

哇，我真sb，啊不对，是这个sdk真sb，这个sdkv2有bug我感觉，他这个就是会导致bl2乱码，总之不要用就完事了

移植的时候发现他会跳到0x80200020，所以需要改掉我代码里所有依赖0x80200000的地方，这个之后肯定需要支持不同的地址，不对，我发现他会智能跳过这段bl33的校检代码，所以不需要该任何地方，然后现在我的kernel输出乱码，然后我不知道为啥我用他的rustsbi编译的文件不会报错，但是用我单独clone的会报错，emmm，pull到最新的master就可以了

### 串口驱动

之后发现，使用sbi的putchar会打印不出东西，虽然我知道打印正解是自己解析设备树，所以我打算先自己解析设备树，然后打印看一下sbi的putchar的返回值

然后如果自己解析设备树的然后写串口驱动的话，就必然会面临不同platform使用的uart的设备不同+base addr不同了，所以需要需要多套代码，所以又需要进行抽象，基本上是把putchar做抽象，底层使用不同的驱动程序，然后addr+driver分开

然后我打算看一下rcore第九章的某些内容学习一下

感觉组织上来说，就是首先把他规划到一个叫做driver/chardev或者uart的mod下，然后定义一套类似trait/interface的东西，然后就是每种串口型号分不同的mod，他们都会实现这个trait，但是，他们的base addr需要从外部传入，类似于预制菜需要加热一样，然后还有一个叫做board的mod，他就相当于是成品，就是他规范了每个board所有的uart实现+base addr，使不使用这个成品都无所谓，但是其实本质上是要通过解析设备树和chosen来决定怎么选择实现和base addr

然后qemu和milkv duo都是uart16550，但是qemu是u8，milkv是u32，所以他们的处理会稍微有点不一样

看了一位大佬写的：[GitHub - YdrMaster/awesome-device: 一种外设定义的合集](https://github.com/YdrMaster/awesome-device)，感觉写的很好，在mmio下，设备驱动就是仅仅提供一个结构体+操作结构体的一系列方法，而对于结构体的内存映射和操作，都应该是系统来做的事情，所以我打算使用他的uart16550

之后学了一下rustsbi的写法，然后分析一下他的层次

- 首先是一个全局的PLATFORM变量，里面记录了所有的BoardDevice

- BoardDevice里面有console这个抽象设备，他是一个KernelConsole

- KernelConsole里面封装了一个dyn的实现了ConsoleDevice trait的具体console的内存布局（比如uart16550的内存布局struct）

然后要PLATFORM中还有一个BoardInfo，从设备树解析选择具体的console device（当然现在可以硬编码）

 之后发现打印不出来的东西，估计是串口的缓冲区满掉了，所以每次打印需要查看成功打印的次数，然后对于没有成功打印的再继续打印，最后我实现了用uart16550u8或sbi驱动qemu virt打印，以及用uart16550u32或sbi驱动milkv duo打印

上述移植的过程确实是太恶心了，所以打算做一个视频来展示上述的过程

## 视频展示

第一次做视频，打算使用obs录屏+达芬奇剪辑，全部都是第一次接触，nixos没有aarch的达芬奇但是有obs

首先是obs，开始先分几个sence，第零个是milkv-duo启动步骤的简单讲解，第一个我觉得是软件准备，首先是duo sdk的准备，一个简单软件的准备（最小kernel），你自己kernel的准备，中间可以穿插milkv-duo的启动过程的描述

[^1]: rustup是The Rust tool chain installer

[^2]: [Attributes](https://dhghomon.github.io/easy_rust/Chapter_52.html#attributes)其可以控制编译器的一些行为，使用#控制下一个语句，而#!控制整个文件
