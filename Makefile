# ========= Config =========
ARCH ?= riscv
MODE ?= release

# Kernel entry
KERNEL_ENTRY_PA ?= 0x80200000

# ========= Per-ARCH Settings =========
ifeq ($(ARCH), riscv)
TARGET        := riscv64gc-unknown-none-elf
QEMU_NAME     := qemu-system-riscv64
SBI           ?= rustsbi
BOARD         ?= qemu
BOOTLOADER    := ./bootloader/$(SBI)-$(BOARD).bin

# Binutils
OBJDUMP       := rust-objdump --arch-name=riscv64
OBJCOPY       := rust-objcopy --binary-architecture=riscv64
GDB           := riscv64-none-elf-gdb

# QEMU
QEMU_ARGS     = -smp 1 \
                 -machine virt \
                 -nographic \
                 -bios $(BOOTLOADER) \
                 -device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA)

else ifeq ($(ARCH), loongarch)
TARGET        := loongarch64-unknown-none
QEMU_NAME     := qemu-system-loongarch64
BOOTLOADER    :=
OBJDUMP       := rust-objdump --arch-name=loongarch64
OBJCOPY       := rust-objcopy --binary-architecture=loongarch64
GDB           := loongarch64-unknown-linux-gnu-gdb

QEMU_ARGS     = -smp 8 \
                 -machine virt \
                 -nographic \
								 -kernel $(KERNEL_ELF)
else
$(error Unsupported ARCH=$(ARCH). Use 'riscv' or 'loongarch')
endif

# ========= Cargo/Paths =========
MODE_ARG      :=
ifeq ($(MODE), release)
	MODE_ARG := --release
endif

KERNEL_ELF    := target/$(TARGET)/$(MODE)/PianoOS
KERNEL_BIN    := $(KERNEL_ELF).bin
DISASM_TMP    := target/$(TARGET)/$(MODE)/asm

# ========= Rules =========
$(KERNEL_ELF):
	cargo build $(MODE_ARG) --target $(TARGET)

$(KERNEL_BIN): $(KERNEL_ELF)
	$(OBJCOPY) --strip-all $(KERNEL_ELF) -O binary $(KERNEL_BIN)

.PHONY: build
build: $(KERNEL_BIN)

.PHONY: clean
clean:
	@cargo clean

.PHONY: disasm
disasm: build
	@$(OBJDUMP) -S $(KERNEL_ELF)

.PHONY: run
run:
	$(QEMU_NAME) $(QEMU_ARGS)

.PHONY: qemu
qemu:
	$(QEMU_NAME) $(QEMU_ARGS)

.PHONY: gdbserver gdbclient
gdbserver: build
	$(QEMU_NAME) $(QEMU_ARGS) -s -S

gdbclient:
	$(GDB) \
	-ex 'file $(KERNEL_ELF)' \
	-ex 'set arch $(if $(filter $(ARCH),riscv),riscv:rv64,loongarch64)' \
	-ex 'target remote localhost:1234' \
	-ex 'layout asm' \
	-tui

# ========= Usage =========
# 默认 RISC-V 调试构建并运行：
#   make run
# RISC-V 发布构建：
#   make MODE=release build
# LoongArch 调试构建并运行：
#   make ARCH=loongarch run
# LoongArch 反汇编：
#   make ARCH=loongarch disasm
