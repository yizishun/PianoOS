# CH2 批处理系统

# 1.特权级机制

这个其实我还是很了解，毕竟写过处理器

但是看他的导读还是有一些新的认识，一言以蔽之，就是所有当前程序不能handle的事情都应该转移到下层去处理，包括除0错误，seg fault等等

# 2.实现应用程序

就是实现那些u mode下的程序，在没有看文章之前的一些想法：感觉就是要使用ecall来完成任务的一些普通程序，然后调用约定和linux一样，然后做写计算，然后被编译成elf到bin，最终和kernel链接到一起，被qemu加载，然后被kernel加载到某个地方，为他配置运行时环境，当然这是后话

首先要加一个叫做user的Package，在cargo中，每个有cargo.toml的都叫做Package，Package中有很多target，比如binary，lib（一个Package中只能有一个），test，example等等，然后我用workspace来包括这两个Package，确保他们有相同的环境，r-a可以正常工作，并且可以直接cargo build将所有的全部build并放入相同文件夹

之后出现了一个问题，在build的时候，最小化到不同的target都需要不同的链接脚本，使用之前的config.toml来告诉cargo显得又些不太够（这个最多只能细化到每个Package中的不同arch_targe），于是使用build.rs来管理项目，学习了rustsbi的写法，将ld文件也硬编码到build.rs中，然后根据不同的[TARGET](https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-build-scripts) env var来选择不同的ld代码

之后将他的user代码复制过来，发现他用了一个有意思的feature，使用unstable feature需要使用nightly版本，是linkarg这个feature，然后关联的issue:[29603](https://github.com/rust-lang/rust/issues/29603) 

之后编译这些用户程序，然后用qemu-riscv64运行这些代码，全部段错误。有点怪，我用gdb调试，发现死在了core::fmt::write里面，死的那条指令是li指令，感觉特别诡异，总觉得应该不是我的问题，然后错误的发生点也是在ecall前面，不管了

build.rs会通过读某些文件，生成一个.S文件，然后src代码会include这个.S文件，这相当于build.rs吐出一个文件，src代码通过约定接收他，这有点不干净我感觉，之前的ld文件我也有点这种感觉，好在能解决，这个我也直接build了

```rust
fn main() {
    let arch = std::env::var("TARGET");
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let ld = &out.join("linker.ld");
    let app_data = &out.join("link_app.S");

    // choose linker script base on arch
    let _ = match arch.as_ref().unwrap().as_str() {
        "riscv64gc-unknown-none-elf" => fs::write(ld, RISCV_LINKER_SCRIPT),
        "loongarch64-unknown-none" => unimplemented!(),
        _ => panic!("
            Unsupported ARCH triple={}. 
            Use 'riscv64gc-unknown-none-elf' or 'loongarch64-unknown-none'", arch.unwrap())
    };
    std::fs::write(ld, RISCV_LINKER_SCRIPT).unwrap();

    // create app data section asm files
    std::fs::write(app_data, insert_app_data()).unwrap();
    let mut build = cc::Build::new();
    build.compiler("riscv64-none-elf-gcc");
    build.file(app_data).compile("app_data");

    println!("cargo:rustc-link-arg=-T{}", ld.display());
    println!("cargo:rustc-link-arg={}", "-Map=/tmp/pianoOSMap.map");
    println!("cargo:rustc-link-search={}", out.display());
}
```

# 3.实现批处理操作系统

## 对于全局变量的一些思考

然后之后他说了全局变量的事情，我思考了一下，有了一些想法

首先有三种改变一个变量的方式

- 1.有他的所有权，自然能改变
- 2.有他的可变引用
- 3.在unsafe block中绕过上述模型，直接进行赋值（全局变量）（raw ptr），这也是我们要规避的

首先，普通的static全局变量，假如他是不可变的，那其实他没有什么意义我感觉，和const没有什么区别，但是如果是mut的话，又很危险，因为按理来说，一个变量需要被修改，你需要获得他的所有权或者可变引用，但是全局变量他绕过了这个规则，他的读写都是在unsafe中对那段内存进行直接更改，绕过了借用检查模型，所以这也不是一个好方法，所以可以发现，不可变和可变是两个极端，一个不好用，一个危险

所以我们需要找一个中间位置，

首先引入RefCell，你可以把它声明为不可变的，但是你可以通过borrow_mut获取他的可变引用，但是你在获得这个可变引用的时候，会存在运行时检查是否有其他的引用（运行时借用检查），这其实本质上是将全局变量也拉入了借用规则之中，并且禁用了直接修改他的可能，此时能通过2来修改他，这相当于是不可变+借用规则实现所谓的内部可变性，这其实已经实现了我们的目标：能改变+很安全（只需要小小用借用规则限制一下你的使用）

这在单线程中确实已经算是最安全的用法了，但是这个类型他没有实现sync，sync叫做同步，其实本质上就是对一些东西进行安排顺序，就像是食堂排队，你有一个窗口，但是有很多人，那你就需要进行管理访问他的顺序，这个类型没有sync，这就意味着，如果a和b在几乎同时请求borrow_mut，由于他们没有被同步，他们可能同时获得这个mut

所以，在多线程环境中，我们需要有一个类型存在这种同步机制（当然也会带来更多的开销），就是Mutex互斥锁或者Rwlock，这种机制其实本质上是需要硬件支持的，硬件存在原子指令来对这种机制进行支持，这个时候硬件能保证在a，b同时请求的时候对他们进行选择并排序，比如mutex，他是独占的，就是它不存在什么borrow，他的所有借用全是borrow_mut，只要你获得了这个锁，你就能对他干任何事，但是你没有获得锁，你就什么也不能干除了自旋（当然还有其他的等待形式，比如直接休眠），而rwlock会更像refcell，他能获得多个读锁或者一个写锁，但是他们在违反借用规则的时候，refcell会直接panic，但是互斥锁会进行等待

然后我的实现用mutex替换refcell（因为我有多harts），然后用Once替换lazy_static!，因为后者好像有点过时

所以我最后的方案就是，结构体本身被Once包裹，通过他初始化结构体中只需要赋值一次的值，然后用Mutex包裹那些需要多次读写的值，也就是提供内部可变性

但是我一开始认为上述的想法是正确的，但是我发现rustsbi并没有把全局变量按照上述方式定义，而是还是使用了static mut，我感觉有点疑惑，于是去问了罗师傅，我发现我理解的还是有点问题，我的比较极端，就是所有的全局变量都必须要声明成static(no mut)，然后所有需要可变的字段都需要用锁来包裹，想改变他必须要先使用锁，但是我忘记锁是有代价的了，他会有性能损失，而如果有某些字段，你能保证多个线程（hart）绝对不会同时访问，同时写，那你就可以用unsafe包裹并不对他做同步，这是零成本行为，这意味着，其实只有某些字段，你的逻辑不确定他是否会被多个harts访问的时候，你才把这个托管责任交给mutex，mutex通过一些性能损失的保证能对这个字段的访问做同步

但是我还是认为static mut太武断了（心理洁癖），某些字段本来是不应该被修改的，但是也被声明为了mut，在使用的时候也需要使用unsafe，我认为这太粗放了

我认为比较合理的控制是：把整体声明为Once，能保证的修改字段用零成本的unsafecell，不能保证的用mutex

----------------

之后在链接生成的s文件的时候出现了abi不匹配的问题，-mabi的文章具体可以看这一篇https://blog.csdn.net/zoomdy/article/details/79353313

之后有一个load_app的函数实现，最后加了一个fence.i，我一开始也没想到，但总之，修改了inst mem，肯定就需要fence.i，不然切换程序的时候就肯定会出问题，但是fence.i是riscv的，架构相关的需要加一层抽象，la我没记错的话是ibar

# 4.实现特权级的切换



然后首先就是要写用户栈，然后还要写陷入函数，我在参考rustsbi的时候看到了一个框架，叫做fast-trap

他里面详细的讲解了一种思想，但是我实在是有点没看懂

来来回回看来几遍，可能是我这方面经验还是不足，所以没怎么看懂，唯一看懂的就是保存上下文的时候可以进行选择性的保存，然后有几个等级的保存，但是他里面对于栈的一些理解我没有办法get到暂时

我应该能理解怎么使用他的这些概念，但是他的描述我可能看不太懂，于是我也想把这个fast-trap的思想移到我的kernel中，但是我应该不会去使用它大部分的代码，至少trap_entry我会自己写

之后我花了一节课的时间把它所有的代码看了一遍，包括部分rustsbi使用这部分代码的代码，幸好不长，然后基本理解了

但是他给的库不能替换某些实现，但是我想想，有没有可以做替换的呢，我的需求是，对于在库中定义的某个struct，他实现的某些方法内部的实现是可以被用户override的，否则就使用它给的default的实现，我总感觉这个实现有点丑陋，就是有一个叫做isa的trait，然后库中定义的struct中的某个字段是需要实现了这个trait的，然后库中的方法会调用这个trait的实现，用户需要自己创建一个叫做isa的struct，然后实现isa这个trait

诶那我是不是可以用这个中思想去实现我kernel的hal，我现在用的方法是，提供一个common的实现，内部用cfg来调用不同的实现，这就导致了我内部特别多的cfg，有点不好看，更不好维护

如果我首先定义一个叫做ISA的trait，里面定义了一些通用方法，他们是对底层指令最简单的抽象，越简单越好，比如说简单到store都行（但这就会导致内部方法特别多），我发现学长貌似就是这么做的，然后每一个isa定义一个struct，然后给这个struct实现这个ISA trait，之后需要用到isa相关代码的地方就把这个struct拉上去，然后没有字段，所以是零开销的

我忽然想起ysyx的abstrat machine了，我打算回过头去看一下

但是这个分类我不知道该怎么分，我有一个ArchISA，但是我认为有些东西确实也不是ISA的范畴，比如说多核，一些和频率相关的东西，架构相关的东西，这些应该不能属于ISA，但是我想了很久，没有想到很好的分类方法，于是我直接选择最粗暴的方式，暂时不分类，全部东西先放在ISA里面，因为毕竟现在方法并不是很多，分类显得很没有意义，与其在这里做没有意义的纠结，不如多积攒点经验，可能一下子就清晰了（ysyx的am是从不同的功能上分类的，比如说trm之类的，但是我感觉这是很有教学风格的，不适合我）

诶我忽然觉得我之前的思想是不完善的，我之前认为我需要将isa（行为）整个抽象出来，但是我发现编译器实际上就是一个hal层！或者说高级编程语言本身就是一层hal，啊这可是刚学计算机就接触的内容呀，我竟然忘记了，有意思，这么说如果我能将isa抽象出来，我就设计了一个编程语言，但是编程语言仅是对于某些指令的抽象，大多是对于计算和分支这种简单指令的抽象，类似fence.i这种指令并没有被抽象，这是为什么呢，我只有一种感觉，就是这种指令他是更高级的指令，他并没有改变什么可变状态，不会因为有一个fencei，让原来的flow出现分叉，他像是一种权衡之计，他很和架构相关，比如一个没有icache的处理器，他就根本不需要fencei，或者说我有一个比icache更能加快指令获取的方式，那fencei指令就会消失，如果我有一种比多核更快的处理方式，所有的多核指令就会消失，他像是和架构实现本身有关的东西（即微架构），我发现这种东西高级编程语言就不会做抽象，但是我忽然发现其实不然，高级编程语言仍然有一些原子的数据结构，他们的内部或许就是fencei

所以我认为高级编程语言像是在操纵一个被抽象过一遍的硬件模型，他是一个巨大的hal，比如在rust的抽象世界中，指针不在是地址了，一个简单的指针包含着很多东西来保证他的安全性，他的硬件世界里有着检查借用规则应用于每一个语言中的对象，偶尔通过unsafe绕过，来完成一些这个抽象层之外的东西

那么我认为我的硬件抽象是没有意义的其实，用entry首先在真实硬件世界创造一个编程语言的世界，然后就可以跳到编程语言的世界了，只是偶尔，硬件世界的某些对象没有被高级编程语言的世界包裹，所以我需要对这些东西做抽象

但是其实又是有意义的，因为我就是在操控真实的硬件世界呀，我的硬件世界里，内存仅仅是一大块数组，他没有任何检查机制，然后他有icache，所以更改了某些部分内存，我需要使用同步机制刷掉cache中的值，这就有一个基本矛盾，我在一个理想的编程语言世界操控真实硬件世界，他们的内存模型都是完全不同的，真实硬件的内存模型就是上述说的，有程序猿不可见的cache等等，但是rust的内存模型是什么呢？https://doc.rust-lang.org/reference/memory-model.html

所以说处理好硬件世界和编程语言世界的矛盾，就是我应该在arch/这个文件夹下做的事情，而不是把整个isa都抽象一遍，或者换一种说法，这个文件夹需要构建一个真实硬件和编程语言世界的连接点（虫洞）

-------------------------

然后之后就是普通的boot，在一开始设置内核栈的时候，会有一个load方法设置sscratch，就是在最后，boot hart进入boot_entry，然后设置sepc为0x80400000，然后设置sstatus的spp为user，以至于可以让sret改特权级，最后是设置用户的sp，我也根据了hartid进行user stack的locat，然后就可以sret

sret到用户态之后，kernel就只需要相应用户态的expt了，在expt的时候，进入fast-trap的框架下

```rust
#[unsafe(naked)]
pub unsafe extern "C" fn trap_entry() {
        core::arch::naked_asm!(
                ".align 2",
                // 换栈
                exchange!(),
                // 加载上下文指针
                save!(a0 => sp[2]),
                load!(sp[0] => a0),
                // 保存尽量少的寄存器
                save!(ra => a0[0]),
                save!(t0 => a0[1]),
                save!(t1 => a0[2]),
                save!(t2 => a0[3]),
                save!(t3 => a0[4]),
                save!(t4 => a0[5]),
                save!(t5 => a0[6]),
                save!(t6 => a0[7]),
                // 调用快速路径函数
                //
                // | reg    | position
                // | ------ | -
                // | ra     | `TrapHandler.context`
                // | t0-t6  | `TrapHandler.context`
                // | a0     | `TrapHandler.scratch`
                // | a1-a7  | 参数寄存器
                // | sp     | sscratch
                // | gp, tp | gp, tp
                // | s0-s11 | 不支持
                //
                // > 若要保留陷入上下文，
                // > 必须在快速路径保存 a0-a7 到 `TrapHandler.context`，
                // > 并进入完整路径执行后续操作。
                // >
                // > 若要切换上下文，在快速路径设置 gp/tp/sscratch/sepc 和 sstatus。
                "mv   a0, sp",
                load!(sp[1] => ra),
                "jalr ra",
                "0:", // 加载上下文指针
                load!(sp[0] => a1),
                // 0：设置少量参数寄存器
                "   beqz  a0, 0f",
                // 1：设置所有参数寄存器
                "   addi  a0, a0, -1
                beqz  a0, 1f
                ",
                // 2：设置所有调用者寄存器
                "   addi  a0, a0, -1
                beqz  a0, 2f
                ",
                // 3：设置所有寄存器
                "   addi  a0, a0, -1
                beqz  a0, 3f
                ",
                // 4：完整路径
                save!(s0  => a1[16]),
                save!(s1  => a1[17]),
                save!(s2  => a1[18]),
                save!(s3  => a1[19]),
                save!(s4  => a1[20]),
                save!(s5  => a1[21]),
                save!(s6  => a1[22]),
                save!(s7  => a1[23]),
                save!(s8  => a1[24]),
                save!(s9  => a1[25]),
                save!(s10 => a1[26]),
                save!(s11 => a1[27]),
                // 调用完整路径函数
                //
                // | reg    | position
                // | ------ | -
                // | sp     | sscratch
                // | gp, tp | gp, tp
                // | else   | `TrapHandler.context`
                //
                // > 若要保留陷入上下文，
                // > 在完整路径中保存 gp/tp/sp/pc 到 `TrapHandler.context`。
                // >
                // > 若要切换上下文，在完整路径设置 gp/tp/sscratch/sepc 和 sstatus。
                "mv   a0, sp",
                load!(sp[2] => ra),
                "jalr ra",
                "j    0b",
                "3:", // 设置所有寄存器
                load!(a1[16] => s0),
                load!(a1[17] => s1),
                load!(a1[18] => s2),
                load!(a1[19] => s3),
                load!(a1[20] => s4),
                load!(a1[21] => s5),
                load!(a1[22] => s6),
                load!(a1[23] => s7),
                load!(a1[24] => s8),
                load!(a1[25] => s9),
                load!(a1[26] => s10),
                load!(a1[27] => s11),
                "2:", // 设置所有调用者寄存器
                load!(a1[ 0] => ra),
                load!(a1[ 1] => t0),
                load!(a1[ 2] => t1),
                load!(a1[ 3] => t2),
                load!(a1[ 4] => t3),
                load!(a1[ 5] => t4),
                load!(a1[ 6] => t5),
                load!(a1[ 7] => t6),
                "1:", // 设置所有参数寄存器
                load!(a1[10] => a2),
                load!(a1[11] => a3),
                load!(a1[12] => a4),
                load!(a1[13] => a5),
                load!(a1[14] => a6),
                load!(a1[15] => a7),
                "0:", // 设置少量参数寄存器
                load!(a1[ 8] => a0),
                load!(a1[ 9] => a1),
                exchange!(),
                r#return!(),
        )
}
```

也就是一开始只保存，t0-t6，ra，然后将a0-a7当作参数寄存器，然后跳到fast handler来做分发，syscall里面有一个exit值得说一下，他会设置sepc，并且会把下一个应用程序加载到相应位置上，然后同样也是restore

跑出来的感觉还是很不错的，有种之前写超标量处理器，跑出第一条指令的感觉，就是那种复杂思想竟然被我写出来了，然后他竟然还work，还是极大的增大了我的信心

# 5.实现新的syscall（练习）

这是练习题

> 1. ** 扩展内核，实现新系统调用get_taskinfo，能显示当前task的id和task name；实现一个裸机应用程序B，能访问get_taskinfo系统调用。

这算是自定义的一个syscall了吧，我打算把自定义的syscall都分配到1000以后

task id好说，直接把app id传出去就好了，但是task name是什么鬼，task name貌似没有任何途径传进去的说，除非改编译部分，把这个当作某个数据传进去

解决这个之前，我不太知道该怎么返回syscall的值，毕竟只能从a0返回（最多a1），于是我参考了linux的那些syscall，比如说多返回值的pipe，linux一般是通过直接修改用户给的一个结构体的指针来做返回的，于是我也可以学习他的做法，定义一个叫做task_info的结构体，然后返回这个结构体，然后a0在成功时是0

然后里面有一个字符串（task name），这个的返回有点麻烦，我看到有类似的syscall叫做getcwd，他的原型是 `char *getcwd(char *buf, size_t size);`，即将path copy进buf，如果path超过size，则返回null，如果成功，就返回buf的这个指针，所以我认为如果需要实现这个syscall最好是把它分为两个syscall，一个返回name一个返回id

当然，他的任务仅仅是让我显示这两个，并且我去看别人的实现，也都没有实现返回，并且也都没有打印出taskname，总之这个任务就感觉怪怪的，我打算就实现一个返回task id的syscall

但是我突然发现，如果是在多核运行的情况下，我竟然不知道当前运行的程序的id，原因是我只有一个全局上锁的变量指示当前运行到哪个程序了，而当前运行程序的id是hart local的，而且我并没有保存这个事情，所以会出问题，但是我现在暂时只是支持单核，所以我直接读这个全局变量是没问题的，但是之后需要修改他的实现

但是在实现过程中，用户程序报了一个load fault的异常，我gdb做调试，找到了出问题的那个地址，然后objdump，发现出现问题的是在`impl core::fmt::Display for i64`这个函数中，他load了一个0地址

woc，我改成debug mode编译，就没问题了？md这种石也能被我碰到，也是神了

好吧，在经过很长的时间的debug，我发现了我的bug，我首先发现在user lib的syscall中加一个`clobber_abi("C")`，就可以正常工作了，然后我发现这个是帮助我保存了所有caller需要保存的寄存器，包括a0-a7，t0-t7，然后我感觉不太对，这个寄存器这么多太保守了，然后我gdb观看ecall前后的寄存器，我发现a7变为0了，然后我发现我在fast_handler中只设置了ctx中的a0，其他的都还是0，如果我使用restore，就会导致a1-a7全部变为0，所以我需要在fast handler中手动保存这些寄存器，然后更改其中的a0，此时就一切正常了（差点以为是rust的bug hhh）

# 6.打印调用栈（panic时）（练习）

>  *** 实现一个裸机应用程序A，能打印调用栈。

在panic时能打印内核的调用栈方便调试，实际上这就是一个unwind的过程

首先第一个问题，我的代码中，貌似没有用fp？，为啥我看我的fp一直是0

我上网搜了一下，发现riscv其实并不需要使用fp，fp的作用就是在unwind的时候会发生作用，然后有一个编译选项`-fomit-frame-pointer`会让代码生成使用fp的代码

然后我打算定义一个feature，叫做unwind_in_panic，然后控制编译是否使用这个编译选项+是否在panic时unwind，发现这件事竟然做不到，太sb了，为什么不能再build.rs里面传编译选项

最后是在config.toml中传编译选项，然后在build.rs来侦测是否有这个编译选项，如果有，就开启一个cfg，然后代码中就可以条件编译了

然后通过了解fp-16保存着上一个fp，所以可以这样不断递归的去找fp

但是需要一个机制停下来，我发现一开始的fp应该是0的，所以就一直递归到检测到他是0就好了

然后写出来就好了，但是我发现我大多是直接用asm写的，但是参考答案是只在一开始用asm获取fp的值，然后之后全部使用了rust的方法，但是都无所谓了

# 7.统计访问系统调用的次数（练习）

> ** 扩展内核，能够统计多个应用的执行过程中系统调用编号和访问此系统调用的次数。

这个东西本身不难，按理来说只要一个全局变量就可以了，但是，如果是多核的情况下，全局变量肯定是不行的，这些信息肯定是hart local的，所以可以考虑把这个信息放到HartContext里面，包括上面的taskid，肯定是不能直接读全局变量的

然后要使在运行的时候可以访问HartContext，我认为比较方便的方式是把HartContext的地址存在TrapHandler中，然后把它作为ctx传给fast_handler

不过其实比较麻烦的事，就是在boot的时候，这些信息是存在在sscratch中的，在syscall的时候，这些信息是游离的，即不会存在一个固定的地方

所以我认为可以在syscall的时候多保存一个tp寄存器，然后用这个tp寄存器指向之前的sscratch，这样我可以封装两个对于hartContext的方法，一个是boot time，一个是trap time，然后设置两套logger，然后用dyn实现替换，就可以在运行时实现打印出hart id

对于taskid，可以在每次运行run_next_app的时候设置对应的HartContext字段，当然就也需要两个方法

然后同样的方式可以实现这个系统调用编号和次数，但是这个比较方便使用hashmap来表示，但是我在core中没有找打hashmap，但是我找到了一个alloc crate里面的Btreemap

# 8.统计应用运行时间（练习）

>  ** 扩展内核，能够统计每个应用执行后的完成时间。

这个任务看上去还挺困难的说，因为要加一个新的time的api

我之前只有一个sleep的api，这个的实现是通过设置一段时间的间隙，让处理器进入idle或者wfi状态，然后设置一个时钟中断

现在的话，比较合适的api设计是获取当前的时间，我认为这个绝对是有指令的，因为我记得la的我实现过

对于riscv貌似也很简单，就是mtime寄存器，但是我访问不了，但是有一个time reg，貌似就是mtime的映射，但是为啥要做一个mtime寄存器，是因为m mode可写他吗

总之挺简单的，然后mhz和ns的换算是f mhz = 1000/f ns，当然ns是周期，mhz是频率

然后我还把app相关的东西打包到app_info struct，然后相关方法也从HartContext分离开了，更清晰了

# 9.统计打印更清晰的用户程序错误信息（练习）

> *** 扩展内核，统计执行异常的程序的异常情况（主要是各种特权级涉及的异常），能够打印异常程序的出错的地址和指令等信息。

这个打印出出错的指令有点难，总之如果要显示指令，需要反汇编，感觉是有点麻烦的，因为需要多架构的反汇编，甚至还有不同长度的指令，比如压缩指令，所以不打印指令了

然后其他的就增加几条sepc，stval的打印，没什么难度

# 10.sys_write 安全检查（练习）

> ch2 中，我们实现了第一个系统调用 `sys_write`，这使得我们可以在用户态输出信息。但是 os 在提供服务的同时，还有保护 os 本身以及其他用户程序不受错误或者恶意程序破坏的功能。
>
> 由于还没有实现虚拟内存，我们可以在用户程序中指定一个属于其他程序字符串，并将它输出，这显然是不合理的，因此我们要对 sys_write 做检查：
>
> - sys_write 仅能输出位于程序本身内存空间内的数据，否则报错。
>
> #### 实验要求
>
> - 实现分支: ch2-lab
>
> - 目录要求不变
>
> - 为 sys_write 增加安全检查
>
>   在 os 目录下执行 `make run TEST=1` 测试 `sys_write` 安全检查的实现，正确执行目标用户测例，并得到预期输出（详见测例注释）。
>
>   注意：如果设置默认 log 等级，从 lab2 开始关闭所有 log 输出。

即需要对用户给的参数进行安全检查

做的时候真的发现出问题，但是按理来说不应该出错呀

有点像是打印了Kernel stack的东西

哦哦想起来了，我的user stack是在kernel stack边上，所以还是我的检测写错了

但是我更改之后还是报错，我发现貌似是真的超过了范围

之后gdb调试发现在locat user stack的时候，hartid本来是a0，但是由于使用之前调用了另一个函数，导致a0被覆盖了，更改了一下顺序就好了，然后将他给的测试程序复制过来，他测试程序里面有一个骚操作，如何在用户程序里面仅仅知道sp，就可以知道栈的top和bottom呢？如果栈的大小是0x1000，那只要清空低12位，就是bottom，但是这需要栈就是0x1000，当然0x10000也行

但是我把我的kernel的stack改成0x1000，我以为不会有什么变化，但是发现卡住了，改成0x2000，卡在更前面了，改成0x4000才能跑起来，感觉有bug，但是有点摸不着头脑的说，然后开始debug：

我首先改成0x1000，发现跑的时候打印的hartid一直在变，于是我watch这段内存（即stack的最低地址），然后发现他会在`core::alloc::layout::Layout::from_size_align_unchecked`中被改变，我backtrace查看，发现这个函数是在打印log!中被调用的，是在print里面需要alloc一个数据才调用的这个函数，改成layout asm，是被s0即fp改的，有点摸不着头脑，我尝试关闭fp这个寄存器，发现0x1000和0x2000全部死在最前面了（即kernel没有打印任何东西）

上述信息我有点怀疑是我在HartContext中使用了BtreeMap，这个数据结构需要调用alloc，于是我把他整个删掉，发现确实不会卡住了，但是0x1000的打印hartid会乱，并且在第一个应用程序处panic，0x2000会在第一个测试程序那里panic

虽然很不相信，但是不得不怀疑是不是栈溢出，还真是，md，4k byte大小的栈就这么被溢出了，笑死

然后可以试一下，直接watch $sp <= 栈顶地址，然后c，来看在运行过程有无栈溢出发生，然后确实是都爆了（我不知道爆掉4k的栈是不是正常情况说实话，还是说rust就是这样的）

要通过测例的话，需要将栈大小改为0x10000，然后按照64K对齐

但是其实不需要，我是user stack和kernel stack共用的一个大小，然后是时候分开了，因为其实他们两个本来就不是一个类型的，至少在他们身上的方法，很多都是kernel stack独有的

# 多核实现

> challenge: 支持多核，实现多个核运行用户程序。

我在前面做了很多方便这里实现的工作，主要是对于hart local的一些变量的一些处理，比如说此hart运行的app id等等，但是依然有些东西不好实现

首先明确一下，这个多个核运行用户程序的sync的吗，就是他们运行的是有序的吗，是一个程序一个程序接续运行，还是如果有8个核，就能8个程序同时运行，我认为前者没有什么意义，那我就默认这个是后者了

后者出现的第一个问题，就是他们同时运行的时候，他们把程序加载到哪里，如果他们的加载地址不是0x80400000，这就意味着这些程序都是位置无关的程序，即他们不应该依赖自己在0x80400000或者其他地方

通过搜索，找到了，这个属于rustc的codegen的option，https://doc.rust-lang.org/rustc/codegen-options/index.html#relocation-model，即在config.toml加一个` "-C", "relocation-model=pic",`我尝试把链接脚本里面的0x80400000删掉，然后加载到这个地址，看看能不能运行，看起来是可以的，objdump也能看到，原来的地址消失了

那这个第一个问题就差不多解决了，然后就是怎么加载，我感觉就可以加载到0x80400000的一个偏移处，相当于平移，然后load_app必须返回他加载到哪里了，从而指导后面的sepc的设置，并且这个信息最好保存在HartContext的app_info里面，方便之后比如说check_buf_valid在sys_write里面的检查，然后这个也能解决了，但是在运行时发现，在打印的时候触发了inst page fault，hello world程序：

```
[kernel] WARN [ 0] - Instruction PageFault in application, kernel killed it.
[kernel] WARN [ 0] - Illegal addr: 0x27c
[kernel] WARN [ 0] - excption pc: 0x27c
```

感觉还是有点问题，稍微有点走投无路，问了ai，说因为在链接时没有加-pie（但是文档上说是会自动加的，**只是**如果他不支持，会自动退回，所以就很恶心，因为他确实不支持），然后报错，说

```
error: linking with `rust-lld` failed: exit status: 1
  |
  = note:  "rust-lld" "-flavor" "gnu" "/tmp/rustcNc8TmL/symbols.o" "<1 object files omitted>" "--as-needed" "-Bstatic" "/home/yzs/rcore/PianoOS/target/riscv64gc-unknown-none-elf/release/deps/libuser_lib-74f3c3f6da611ea3.rlib" "<sysroot>/lib/rustlib/riscv64gc-unknown-none-elf/lib/{libcore-*,libcompiler_builtins-*}.rlib" "-L" "/tmp/rustcNc8TmL/raw-dylibs" "-Bdynamic" "-z" "noexecstack" "-L" "/home/yzs/rcore/PianoOS/target/riscv64gc-unknown-none-elf/release/build/user_lib-3a44cde3a76a61ed/out" "-o" "/home/yzs/rcore/PianoOS/target/riscv64gc-unknown-none-elf/release/deps/01store_fault-af76e34c012f3a3e" "--gc-sections" "-O1" "--strip-debug" "-T/home/yzs/rcore/PianoOS/target/riscv64gc-unknown-none-elf/release/build/user_lib-3a44cde3a76a61ed/out/linker.ld" "-Map=/tmp/UserMap.map" "-pie"
  = note: some arguments are omitted. use `--verbose` to show all linker arguments
  = note: rust-lld: error: relocation R_RISCV_64 cannot be used against symbol '.Lanon.505f3601e2136fe1a7296e3022027dc7.52'; recompile with -fPIC
          >>> defined in /home/yzs/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/riscv64gc-unknown-none-elf/lib/libcore-7bcf3123a0e9c15d.rlib(core-7bcf3123a0e9c15d.core.2ad1de27dc69bd30-cgu.0.rcgu.o)
          >>> referenced by core.2ad1de27dc69bd30-cgu.0
          >>>               core-7bcf3123a0e9c15d.core.2ad1de27dc69bd30-cgu.0.rcgu.o:(.Lanon.505f3601e2136fe1a7296e3022027dc7.295) in archive /home/yzs/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/riscv64gc-unknown-none-elf/lib/libcore-7bcf3123a0e9c15d.rlib
```

貌似具体是，这些libcore这些预编译好的crate没有用pic编译，所以需要手动重编这些crate，可参考：https://doc.rust-lang.org/cargo/reference/unstable.html#build-std

然后我发现其实rustsbi也用了这个build-std，但是他们用了一个叫做xtask的东西，这个东西貌似只是一个普通的crate，只是他会帮忙构建整个应用程序（其实感觉本质上就是一个帮助调用cargo的东西，因为他本质就是调用了一个sub process，这个process就是一个cargo程序，只是你能在这个程序中预定义很多cargo选项），这个其实本质上和cargo的那个config.toml是一样的

## xtask

在rustsbi中使用`cargo prototyper --jump`就可以进行编译，最主要的是他们的config.toml存在

```
[alias]
xtask = "run --package xtask --release --"
prototyper = "xtask prototyper"
test-kernel = "xtask test"
bench-kernel = "xtask bench"
```

然后最主要的就是这个xtask程序是怎么写的，然后我打算直接把他们的移植过来，xtask不应该使用riscv的target，但是我实在没找到怎么配置这种有多个package但是他们各自的target不同的，只能每次跑哪个的时候改target，哎

然后clap这个crate确实有点恶心，可以看他官方的一些例子学习：https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html

写着写着发现有点理不清了，所以明确一下，xtask仅仅只是cargo的调用，但是真的编译过程还是由build.rs控制的，所以说将user program链接进来的这个操作还是由build.rs来控制的，然后里面最重要的一段参数是

```rust
	let rustflags = 
		"-C relocation-model=pie -C link-arg=-pie -C force-frame-pointers=yes";

	let status = cargo::Cargo::new("build")
		.package(USER_PACKAGE_NAME)
		.target(arch)
		.unstable("build-std", ["core", "alloc"])
```

即重新编译core，使用rustflags的参数

之后打算用xtask来一键运行qemu，然后就可以抛弃Makefile了hhh

## 重定位

然后重新编译之后我发现每个程序都会触发0地址访问错误，貌似是因为，用户程序被这样编译之后，会变成dyn elf，此时需要在运行时重定位一些东西，rustsbi中相应的代码如下

```rust
// Handle relocations for position-independent code
#[unsafe(naked)]
unsafe extern "C" fn relocation_update() {
    naked_asm!(
        // Get load offset.
        "   li t0, {START_ADDRESS}",
        "   lla t1, sbi_start",
        "   sub t2, t1, t0",

        // Foreach rela.dyn and update relocation.
        "   lla t0, __rel_dyn_start",
        "   lla t1, __rel_dyn_end",
        "   li  t3, {R_RISCV_RELATIVE}",
        "1:",
        "   ld  t4, 8(t0)",
        "   bne t4, t3, 2f",
        "   ld t4, 0(t0)", // Get offset
        "   ld t5, 16(t0)", // Get append
        "   add t4, t4, t2", // Add load offset to offset add append
        "   add t5, t5, t2",
        "   sd t5, 0(t4)", // Update address
        "   addi t0, t0, 24", // Get next rela item
        "2:",
        "   blt t0, t1, 1b",
        "   fence.i",

        // Return
        "   ret",
        R_RISCV_RELATIVE = const R_RISCV_RELATIVE,
        START_ADDRESS = const cfg::SBI_LINK_START_ADDRESS,
    )
}
```

然后对照一下我的某个.rela.dyn：
```
Relocation section '.rela.dyn' at offset 0x1de8 contains 16 entries:
  Offset          Info           Type           Sym. Value    Sym. Name + Addend
000000001198  000000000003 R_RISCV_RELATIVE                     334
0000000011a0  000000000003 R_RISCV_RELATIVE                     12b0
0000000011a8  000000000003 R_RISCV_RELATIVE                     12b0
0000000011b0  000000000003 R_RISCV_RELATIVE                     f68
0000000011c0  000000000003 R_RISCV_RELATIVE                     f76
0000000011d0  000000000003 R_RISCV_RELATIVE                     f82
0000000011e0  000000000003 R_RISCV_RELATIVE                     f83
0000000011f0  000000000003 R_RISCV_RELATIVE                     f85
000000001218  000000000003 R_RISCV_RELATIVE                     27c
000000001220  000000000003 R_RISCV_RELATIVE                     19a
000000001228  000000000003 R_RISCV_RELATIVE                     238
000000001248  000000000003 R_RISCV_RELATIVE                     258
000000001250  000000000003 R_RISCV_RELATIVE                     fc6
000000001268  000000000003 R_RISCV_RELATIVE                     fda
000000001278  000000000003 R_RISCV_RELATIVE                     fb6
0000000012a0  000000000003 R_RISCV_RELATIVE                     ff5
```

应该意思就是把类似第一条：load base addr + 1198 = load base addr + 334

但是kernel加载的时候会出现你不知道每个用户程序的.rela.dyn在哪的情况，因为他们已经是bin文件链接进来的了，所以其实最好的方式是让kernel能加载elf文件，啊我讨厌解析elf文件

## elf文件的解析

其实首先可以把之前的app_start_addr进行封装，即封装每一个app自己的数据，然后可以用rust的elf crate做解析

然后忘记怎么load了，不过我记得以前pa的讲义里应该有写的，可以参考：[pa][https://ysyx.oscc.cc/docs/ics-pa/3.3.html#加载第一个用户程序]，最主要的是理解这一句：

> ELF文件提供了两个视角来组织一个可执行文件, 一个是面向链接过程的section视角, 这个视角提供了用于链接与重定位的信息(例如符号表); 另一个是面向执行的segment视角, 这个视角提供了用于加载可执行文件的信息. 

然后加载之后就是重定向，重定向需要把addr本身写进需要写入的地址，代码如下

```rust
		let rela_dyn_header = file.section_header_by_name(".rela.dyn")
			.expect("section table should be parseable")
			.expect("should have .rela.dyn unless this elf file is not pie");
		let rela_dyn = file.section_data_as_relas(&rela_dyn_header)
			.expect("section data not found")
			.filter(|e| e.r_type == R_RISCV_RELATIVE);
		for entry in rela_dyn {
			unsafe {
				let offset = dst_start.byte_add(entry.r_offset as usize) as *mut usize;
				let append = dst_start.byte_add(entry.r_addend as usize) as usize;
				*offset = append;
				ARCH.fencei();
			}

		}
```

终于成功了

## 多核执行

然后就是让除了boot hart的其他harts也开始load app并开始执行，现在他们在启动之后会进入hart_main，然后进入死循环，然后把他们的死循环改成加载程序，因为cur_app有锁，所以不用担心竞争的情况

写出来了，运行后有两个问题，1.打印的东西比较重合，虽然kernel打印的时候会标记出自己的hart number，但是还是有点乱，但是估计是很难解决。2.在有的hart还没执行完的时候，某个hart已经到了最后了，所以他就把整个shutdown了

第二个问题使用一个bitmap来指示是否完成，使用`AtomicBool`的array即可，在每个hart执行完之后，把他hart local记录的app id设置为已完成即可，最后退出的时候需要检测是否所有程序都已完成

## 最终输出

```
[PianoOS-xtask] INFO  - Boot QEMU: "qemu-system-riscv64" "-machine" "virt" "-smp" "8" "-bios" "./bootloader/rustsbi-qemu.bin" "-device" "loader,file=/home/yzs/rcore/PianoOS/target/riscv64gc-unknown-none-elf/release/PianoOS.bin,addr=0x80200000" "-nographic"
[RustSBI] INFO  - Hello RustSBI!
[RustSBI] INFO  - RustSBI version 0.4.0
[RustSBI] INFO  - .______       __    __      _______.___________.  _______..______   __
[RustSBI] INFO  - |   _  \     |  |  |  |    /       |           | /       ||   _  \ |  |
[RustSBI] INFO  - |  |_)  |    |  |  |  |   |   (----`---|  |----`|   (----`|  |_)  ||  |
[RustSBI] INFO  - |      /     |  |  |  |    \   \       |  |      \   \    |   _  < |  |
[RustSBI] INFO  - |  |\  \----.|  `--'  |.----)   |      |  |  .----)   |   |  |_)  ||  |
[RustSBI] INFO  - | _| `._____| \______/ |_______/       |__|  |_______/    |______/ |__|
[RustSBI] INFO  - Initializing RustSBI machine-mode environment.
[RustSBI] INFO  - Platform Name                 : riscv-virtio,qemu
[RustSBI] INFO  - Platform HART Count           : 8
[RustSBI] INFO  - Enabled HARTs                 : [0, 1, 2, 3, 4, 5, 6, 7]
[RustSBI] INFO  - Platform IPI Extension        : SiFiveClint (Base Address: 0x2000000)
[RustSBI] INFO  - Platform Console Extension    : Uart16550U8 (Base Address: 0x10000000)
[RustSBI] INFO  - Platform Reset Extension      : Available (Base Address: 0x100000)
[RustSBI] INFO  - Platform HSM Extension        : Available
[RustSBI] INFO  - Platform RFence Extension     : Available
[RustSBI] INFO  - Platform SUSP Extension       : Available
[RustSBI] INFO  - Platform PMU Extension        : Available
[RustSBI] INFO  - Memory range                  : 0x80000000 - 0x88000000
[RustSBI] INFO  - Platform Status               : Platform initialization complete and ready.
[RustSBI] INFO  - The patched dtb is located at 0x80054000 with length 0x3208.
[RustSBI] INFO  - PMP Configuration
[RustSBI] INFO  - PMP        Range      Permission      Address                       
[RustSBI] INFO  - PMP 0:     OFF        NONE            0x00000000
[RustSBI] INFO  - PMP 1-2:   TOR        RWX/RWX         0x80000000 - 0x80000000
[RustSBI] INFO  - PMP 3-5:   TOR        NONE/NONE       0x80021000 - 0x8002e000 - 0x80069000
[RustSBI] INFO  - PMP 6:     TOR        RWX             0x88000000
[RustSBI] INFO  - PMP 7:     TOR        RWX             0xffffffffffffffff
[RustSBI] INFO  - Boot HART ID                  : 5
[RustSBI] INFO  - Boot HART Privileged Version: : Version1_12
[RustSBI] INFO  - Boot HART MHPM Mask:          : 0x07ffff
[RustSBI] INFO  - Redirecting hart 5 to 0x00000080200000 in Supervisor mode.
[kernel] INFO [ 5] - Logging system init success
[kernel] INFO [ 5] - boot hartid: 5
[kernel] INFO [ 5] - device tree addr: 0x80054000
[kernel] INFO [ 5] - cpu number: 8
[kernel] INFO [ 5] - uart type is Uart16550U8, base addr is 0x10000000
[kernel] INFO [ 5] - kernel memory map:
[kernel] INFO [ 5] - kernel base = 0x80200000
[kernel] INFO [ 5] - .text      : [0x80200000, 0x80214000]
[kernel] INFO [ 5] - .rodata    : [0x80214000, 0x80218000]
[kernel] INFO [ 5] - .data      : [0x80218000, 0x80244000]
[kernel] INFO [ 5] - .bss.kstack: [0x80244000, 0x80264000]
[kernel] INFO [ 5] - .bss.ustack: [0x80264000, 0x8026c000]
[kernel] INFO [ 5] - .bss.heap  : [0x8026c000, 0x80274000]
[kernel] INFO [ 5] - .bss       : [0x80274000, 0x80275000]
[kernel] INFO [ 5] - kernel end = 0x80275000
[kernel] INFO [ 5] - Kernel app number: 8
[kernel] INFO [ 5] - app 0: [0x80219828, 0x8021e288]
[kernel] INFO [ 5] - app 1: [0x8021e288, 0x80222e78]
[kernel] INFO [ 5] - app 2: [0x80222e78, 0x80228118]
[kernel] INFO [ 5] - app 3: [0x80228118, 0x8022ccf0]
[kernel] INFO [ 5] - app 4: [0x8022ccf0, 0x802318c8]
[kernel] INFO [ 5] - app 5: [0x802318c8, 0x80236680]
[kernel] INFO [ 5] - app 6: [0x80236680, 0x8023cdc0]
[kernel] INFO [ 5] - app 7: [0x8023cdc0, 0x802434c0]
Hello, world!
Into Test store_fault, we will insert an invalid store operation...
Try to access privileged CSR in U Mode
[kernel] WARN [ 4][ 6] - buf out of scope
string from data section
Kernel should kill this application!
[kernel] WARN [ 4][ 6] - buf addr: 0x0
[kernel] TRACE[ 5][ 0] - ==== App(0) statistics ====
3[kernel] WARN [ 4][ 6] - buf size: 0xa
strin[kernel] TRACE[ 5][ 0] - Start addr: 0x80400000
task id ^[kernel] TRACE[ 5][ 0] - End addr  : 0x804012b0
[kernel] TRACE[ 3][ 4] - ==== App(4) statistics ====
string from stack section
5[kernel] TRACE[ 5][ 0] - Start time: 30578400ns
Kernel should kill this application!
[kernel] TRACE[ 5][ 0] - End time  : 31172900ns
strin
[kernel] TRACE[ 3][ 4] - Start addr: 0x804134c8
10000[kernel] TRACE[ 0][ 1] - ==== App(1) statistics ====
[kernel] TRACE[ 1][ 5] - ==== App(5) statistics ====
[kernel] TRACE[ 3][ 4] - End addr  : 0x80414818
[kernel] WARN [ 4][ 6] - ustack start: 0x80268000
[kernel] TRACE[ 3][ 4] - Start time: 30731300ns
[kernel] TRACE[ 1][ 5] - Start addr: 0x804180a0

Test write1 OK!
[kernel] WARN [ 4][ 6] - ustack end  : 0x80269000
[kernel] TRACE[ 0][ 1] - Start addr: 0x80404a60
[kernel] TRACE[ 1][ 5] - End addr  : 0x80419510
=[kernel] TRACE[ 2][ 7] - ==== App(7) statistics ====
[kernel] TRACE[ 0][ 1] - End addr  : 0x80405dc8
[kernel] TRACE[ 5][ 0] - Total time: 594500ns
[kernel] TRACE[ 3][ 4] - End time  : 31830700ns
[kernel] TRACE[ 5][ 0] - Syscall statistics --
[kernel] TRACE[ 0][ 1] - Start time: 30655200ns
Try to execute privileged instruction in U Mode
[kernel] TRACE[ 1][ 5] - Start time: 30754600ns
[kernel] TRACE[ 2][ 7] - Start addr: 0x80423598
[kernel] TRACE[ 0][ 1] - End time  : 32548900ns
Kernel should kill this application!
[kernel] TRACE[ 5][ 0] - Write: 1
5079[kernel] TRACE[ 3][ 4] - Total time: 1099400ns
[kernel] TRACE[ 6][ 3] - ==== App(3) statistics ====
[kernel] TRACE[ 2][ 7] - End addr  : 0x804252c0
[kernel] TRACE[ 0][ 1] - Total time: 1893700ns
(MOD [kernel] TRACE[ 3][ 4] - Syscall statistics --
10007[kernel] TRACE[ 1][ 5] - End time  : 32658500ns
[kernel] TRACE[ 0][ 1] - Syscall statistics --
[kernel] TRACE[ 2][ 7] - Start time: 30816000ns
[kernel] TRACE[ 0][ 1] - Write: 2
[kernel] TRACE[ 2][ 7] - End time  : 33935600ns
[kernel] TRACE[ 5][ 0] - Exit: 1
[kernel] TRACE[ 2][ 7] - Total time: 3119600ns
[kernel] TRACE[ 5][ 0] - GetTaskID: 0
[kernel] TRACE[ 6][ 3] - Start addr: 0x8040e8f0
[kernel] TRACE[ 2][ 7] - Syscall statistics --
)
[kernel] TRACE[ 3][ 4] - Write: 2
[kernel] TRACE[ 0][ 1] - Exit: 0
[kernel] TRACE[ 2][ 7] - Write: 6
[kernel] TRACE[ 1][ 5] - Total time: 1903900ns
[kernel] TRACE[ 5][ 0] - == App(0) statistics end ==
[kernel] TRACE[ 2][ 7] - Exit: 1
[kernel] TRACE[ 3][ 4] - Exit: 0
3[kernel] TRACE[ 1][ 5] - Syscall statistics --
[kernel] TRACE[ 2][ 7] - GetTaskID: 0
[kernel] INFO [ 5][ 0] - Application exited with code 0
[kernel] TRACE[ 2][ 7] - == App(7) statistics end ==
[kernel] TRACE[ 1][ 5] - Write: 3
^[kernel] INFO [ 2][ 7] - Application exited with code 0
[kernel] TRACE[ 0][ 1] - GetTaskID: 0
20000[kernel] TRACE[ 6][ 3] - End addr  : 0x8040fc40
[kernel] TRACE[ 3][ 4] - GetTaskID: 0
[kernel] TRACE[ 0][ 1] - == App(1) statistics end ==
[kernel] WARN [ 4][ 6] - app size: 0x6740
[kernel] TRACE[ 6][ 3] - Start time: 30701500ns
=[kernel] TRACE[ 3][ 4] - == App(4) statistics end ==
8202[kernel] TRACE[ 1][ 5] - Exit: 1
[kernel] TRACE[ 6][ 3] - End time  : 35626100ns
[kernel] WARN [ 3][ 4] - IllegalInstruction in application, kernel killed it.
top 0x[kernel] WARN [ 0][ 1] - PageFault in application, kernel killed it.
(MOD [kernel] TRACE[ 6][ 3] - Total time: 4924600ns
[kernel] WARN [ 3][ 4] - excption pc: 0x804135a0
80269000[kernel] TRACE[ 6][ 3] - Syscall statistics --
10007[kernel] WARN [ 0][ 1] - Illegal addr: 0x0
)
, bottom 0x[kernel] TRACE[ 6][ 3] - Write: 2
[kernel] TRACE[ 1][ 5] - GetTaskID: 1
[kernel] TRACE[ 6][ 3] - Exit: 0
3[kernel] TRACE[ 1][ 5] - == App(5) statistics end ==
[kernel] TRACE[ 6][ 3] - GetTaskID: 0
80268000[kernel] WARN [ 0][ 1] - excption pc: 0x80404b34
^
[kernel] TRACE[ 6][ 3] - == App(3) statistics end ==
30000[kernel] WARN [ 4][ 6] - buf out of scope
=[kernel] WARN [ 6][ 3] - IllegalInstruction in application, kernel killed it.
8824[kernel] WARN [ 4][ 6] - buf addr: 0x80268ffb
(MOD [kernel] INFO [ 1][ 5] - Application exited with code 0
10007[kernel] WARN [ 4][ 6] - buf size: 0xa
)
[kernel] WARN [ 6][ 3] - excption pc: 0x8040e9c4
[kernel] WARN [ 4][ 6] - ustack start: 0x80268000
3^40000[kernel] WARN [ 4][ 6] - ustack end  : 0x80269000
=5750[kernel] WARN [ 4][ 6] - app size: 0x6740
(MOD 10007)
[kernel] WARN [ 4][ 6] - buf out of scope
3^50000[kernel] WARN [ 4][ 6] - buf addr: 0x80267ffb
=3824[kernel] WARN [ 4][ 6] - buf size: 0xa
(MOD [kernel] WARN [ 4][ 6] - ustack start: 0x80268000
10007)
[kernel] WARN [ 4][ 6] - ustack end  : 0x80269000
3^[kernel] WARN [ 4][ 6] - app size: 0x6740
60000=8516(MOD 10007)
Test write0 OK!
3^70000=2510(MOD [kernel] TRACE[ 4][ 6] - ==== App(6) statistics ====
10007[kernel] TRACE[ 4][ 6] - Start addr: 0x8041ce58
)
[kernel] TRACE[ 4][ 6] - End addr  : 0x8041ebb0
3^80000[kernel] TRACE[ 4][ 6] - Start time: 30783200ns
=9379[kernel] TRACE[ 4][ 6] - End time  : 44848900ns
(MOD 10007[kernel] TRACE[ 4][ 6] - Total time: 14065700ns
)
[kernel] TRACE[ 4][ 6] - Syscall statistics --
3^90000[kernel] TRACE[ 4][ 6] - Write: 9
=2621[kernel] TRACE[ 4][ 6] - Exit: 1
(MOD [kernel] TRACE[ 4][ 6] - GetTaskID: 0
10007[kernel] TRACE[ 4][ 6] - == App(6) statistics end ==
)
[kernel] INFO [ 4][ 6] - Application exited with code 0
3^100000=2749(MOD 10007)
Test power OK!
[kernel] TRACE[ 7][ 2] - ==== App(2) statistics ====
[kernel] TRACE[ 7][ 2] - Start addr: 0x80409650
[kernel] TRACE[ 7][ 2] - End addr  : 0x8040ad10
[kernel] TRACE[ 7][ 2] - Start time: 30679600ns
[kernel] TRACE[ 7][ 2] - End time  : 46960800ns
[kernel] TRACE[ 7][ 2] - Total time: 16281200ns
[kernel] TRACE[ 7][ 2] - Syscall statistics --
[kernel] TRACE[ 7][ 2] - Write: 81
[kernel] TRACE[ 7][ 2] - Exit: 1
[kernel] TRACE[ 7][ 2] - GetTaskID: 0
[kernel] TRACE[ 7][ 2] - == App(2) statistics end ==
[kernel] INFO [ 7][ 2] - Application exited with code 0
[kernel] INFO [ 5][ 0] - All applications completed! Kennel shutdown
```

多核的话输出都是乱的，我也不知道咋办