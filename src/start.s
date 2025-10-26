# ==============================================================================
# The SakuraOS Bootloader
# Copyright 2025 Adam Judge
# ==============================================================================

GDT_CS = 0x08
GDT_DS = 0x10

# ==============================================================================
# Real-mode entry point from boot sector
# ==============================================================================

.section .text.start

.global _start

.code16
_start:
    cli

    # Load GDT and enable protected mode.
    lgdt gdt_desc
    mov %cr0, %eax
    or $1, %al
    mov %eax, %cr0
    jmpl $GDT_CS, $enter_protected_mode

# Start of 32-bit protected mode code.
.code32
enter_protected_mode:
    mov $GDT_DS, %ax
    mov %ax, %ds
    mov %ax, %ss
    mov %ax, %es
    mov %ax, %fs
    mov %ax, %gs
    mov $__loram_top, %esp
    jmp main

# ==============================================================================
# Processor Data Structures
# ==============================================================================

.align 2

# GDT Descriptor
# Kept in .text.start to ensure it doesn't go out of range of 16-bit address
gdt_desc:
    .short gdt_end - gdt_start - 1
    .int gdt_start

.rodata
.align 8

# Global Descriptor Table
gdt_start:
    .quad 0x0000000000000000  # Null segment
    .quad 0x00CF9B000000FFFF  # Code segment
    .quad 0x00CF93000000FFFF  # Data segment
gdt_end:
