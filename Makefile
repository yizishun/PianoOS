# Building
TARGET := riscv64gc-unknown-none-elf
MODE := debug
KERNEL_ELF := target/$(TARGET)/$(MODE)/PianoOS
KERNEL_BIN := $(KERNEL_ELF).bin
DISASM_TMP := target/$(TARGET)/$(MODE)/asm

# Building mode argument
ifeq ($(MODE), release)
	MODE_ARG := --release
endif

# BOARD
BOARD := qemu
SBI ?= rustsbi
BOOTLOADER := ./bootloader/$(SBI)-$(BOARD).bin

# Kernel
KERNEL_ENTRY_PA := 0x80200000

# Binutils
OBJDUMP := rust-objdump --arch-name=riscv64
OBJCOPY := rust-objcopy --binary-architecture=riscv64
GDB := riscv64-none-elf-gdb

#Qemu
QEMU_ARGS := -smp 8 \
			 -machine virt \
			 -nographic \
			 -bios $(BOOTLOADER) \
			 -device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA)
QEMU_NAME := qemu-system-riscv64

$(KERNEL_ELF):
	cargo build $(MODE_ARG)
$(KERNEL_BIN): $(KERNEL_ELF)
	$(OBJCOPY) \
	--strip-all \
	$(KERNEL_ELF) \
	-O binary \
	$(KERNEL_BIN)

.PHONY: build
build: $(KERNEL_BIN)

.PHONY: clean
clean:
	@cargo clean

.PHONY: disasm
disasm: build
	@$(OBJDUMP) -S $(KERNEL_ELF)

.PHONY: run
run: build
	$(QEMU_NAME) $(QEMU_ARGS)

.PHONY: qemu
qemu:
	$(QEMU_NAME) $(QEMU_ARGS)

.PHONY: gdbserver gdbclient
gdbserver: build
	qemu-system-riscv64 $(QEMU_ARGS) -s -S

gdbclient:
	$(GDB) \
	-ex 'file $(KERNEL_ELF)' \
	-ex 'set arch riscv:rv64' \
	-ex 'target remote localhost:1234' \
	-ex 'layout asm' \
	-tui
