# ==============================================================================
# The SakuraOS Bootloader
# Copyright 2025 Adam Judge
# ==============================================================================

STACK_TOP = 0x8000

GDT_CS = 0x08
GDT_DS = 0x10

# ==============================================================================
# Real-mode entry point from boot sector
# ==============================================================================

.section .text.start

.code16
_start:
    cli
    movl $STACK_TOP, %esp

    # Enable A20 through the keyboard controller.
    call kbc_wait
    mov $0xD1, %al
    out %al, $0x64
    call kbc_wait
    mov $0xDF, %al
    out %al, $0x60
    call kbc_wait

    # Load GDT and enable protected mode.
    lgdt gdt_desc
    mov %cr0, %eax
    or $1, %al
    mov %eax, %cr0
    jmpl $GDT_CS, $enter_protected_mode

kbc_wait:
    mov $10000, %cx
1:
    loop 1b
    in $0x64, %al
    test $2, %al
    jnz kbc_wait
    ret

# Start of 32-bit protected mode code.
.code32
enter_protected_mode:
    mov $GDT_DS, %ax
    mov %ax, %ds
    mov %ax, %ss
    mov %ax, %es
    mov %ax, %fs
    mov %ax, %gs
    jmp main

# ==============================================================================
# Processor Data Structures
# ==============================================================================

.align 2

# GDT Descriptor
# Kept in .text.start to ensure it doesn't go out of range of 16-bit address
gdt_desc:
    .short gdt_end - gdt_start - 1
    .4byte gdt_start

.rodata
.align 8

# Global Descriptor Table
gdt_start:
    .quad 0x0000000000000000  # Null segment
    .quad 0x00CF9B000000FFFF  # Code segment
    .quad 0x00CF93000000FFFF  # Data segment
gdt_end:
