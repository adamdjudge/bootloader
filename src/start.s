# ==============================================================================
# The SakuraOS Bootloader
# Copyright 2026 Adam Judge
# ==============================================================================

# Segment offsets
GDT_CS = 0x08
GDT_DS = 0x10

# BIOS E820 call memory region structure size
REGION_STRUCT_SIZE = 24

# ==============================================================================
# Real-mode entry point from boot sector
# ==============================================================================

.section .text.start

.code16
_start:
    cli

    # Detect memory regions using the BIOS E820 call.
    xor %ebx, %ebx
    mov $e820_regions, %di
.e820_loop:
    mov $0xE820, %eax
    mov $REGION_STRUCT_SIZE, %ecx
    mov $0x534D4150, %edx
    int $0x15

    jc .e820_done
    cmp $0x534D4150, %eax
    jne .e820_done
    cmp $0, %ebx
    je .e820_done

    add $REGION_STRUCT_SIZE, %di
    incl e820_regions_count
    jmp .e820_loop

.e820_done:
    clc

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

    # Clear out the BSS before starting Rust.
    mov $__bss_start, %edi
    mov $__bss_end, %ecx
    sub $__bss_start, %ecx
    add $3, %ecx
    shr $2, %ecx
    xor %eax, %eax
    rep stosl

    # Set 32-bit stack pointer and jump to Rust execution.
    mov $__loram_top, %esp
    jmp main

# ==============================================================================
# Data Structure Definitions
# Kept in .text.start to ensure they remain within 16-bit addressing
# ==============================================================================

# Global Descriptor Table
.align 8
gdt_start:
    .quad 0x0000000000000000  # Null segment
    .quad 0x00CF9B000000FFFF  # Code segment
    .quad 0x00CF93000000FFFF  # Data segment
gdt_end:

# GDT Descriptor
.align 2
gdt_desc:
    .short gdt_end - gdt_start - 1
    .int gdt_start

# BIOS memory region array
.align 4
e820_regions:
    .space REGION_STRUCT_SIZE * 16
e820_regions_count:
    .space 4
