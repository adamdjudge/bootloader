; ==============================================================================
; Bootloader for Minix-formatted floppy
; Copyright 2025 Adam Judge
; ==============================================================================

bits 16
org 0x7C00

buffer equ 0x7C00 + 512

; Minix super block field offsets
s_ninodes          equ 0
s_nzones           equ 2
s_imap_blocks      equ 4
s_zmap_blocks      equ 6
s_first_data_zone  equ 8
s_log_zone_size    equ 10
s_max_size         equ 12
s_magic            equ 16
s_state            equ 18

; Minix inode field offsets
i_mode    equ 0
i_uid     equ 2
i_size    equ 4
i_time    equ 8
i_gid     equ 12
i_nlinks  equ 13
i_zones   equ 14

; ==============================================================================
; ENTRY POINT
; ==============================================================================

start:
    cli
    cld
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7000
    sti

; Load super block
    mov ax, 2
    mov cl, 1
    mov bx, buffer
    call read_floppy

; Load first inode block to get root dir inode
    mov ax, 2
    add ax, [buffer + s_imap_blocks]
    add ax, [buffer + s_zmap_blocks]
    shl ax, 1
    mov word [inodes_start], ax
    mov cl, 1
    mov bx, buffer
    call read_floppy

; Load root dir
; This only reads the first block, so kernel must be within the first block of
; directory entries
    mov ax, [buffer + i_zones]
    shl ax, 1
    mov cl, 2
    mov bx, buffer
    call read_floppy

; Find kernel inode number in root dir
    mov bx, buffer - 16
next_entry:
    add bx, 16
    mov ax, [bx]
    cmp ax, 0
    je no_kernel
    mov di, kernel_name
    mov si, bx
    add si, 2
    mov cx, 7
    rep cmpsb
    jne next_entry
    
; Calculate sector containing kernel inode and load it
    dec ax
    push ax
    shr ax, 4
    add ax, [inodes_start]
    mov bx, buffer
    mov cl, 1
    call read_floppy

; Calculate inode offset within sector
    pop bx
    and bx, 0x000F
    shl bx, 5
    add bx, buffer
    
; If the file size is large enough for the inode to have an indirect block,
; load the indirect blocks directly after the inode's direct blocks to
; concatenate them as one array for the following file load loop
    cmp word [bx + i_zones + 14], 0
    je start_load
    mov ax, [bx + i_zones + 14]
    shl ax, 1
    push bx
    add bx, i_zones + 14
    mov cl, 2
    call read_floppy
    pop bx

; Load kernel blocks
start_load:
    add bx, i_zones
next_block:
    mov ax, [bx]
    cmp ax, 0
    je done
    shl ax, 1
    push bx
    mov bx, [load_ptr]
    mov cl, 2
    call read_floppy
    pop bx
    add bx, 2
    add word [load_ptr], 1024
    jnc next_block
    mov ax, es
    add ax, 0x1000
    mov es, ax
    jmp next_block

; Check kernel signature and jump to kernel
done:
    xor ax, ax
    mov es, ax
    jmp 0x0000:0x8000

no_kernel:
    mov si, not_found
    call print
    jmp reboot

; ==============================================================================
; BOOTLOADER SUBROUTINES
; ==============================================================================

; Print a null-terminated string
; Params: SI = String pointer
; Return: None
print:
    pusha
    mov ah, 0xE
.repeat:
    lodsb
    cmp al, 0
    je .done
    int 0x10
    jmp short .repeat
.done:
    popa
    ret

; Load sectors from floppy into buffer
; Params: AX = start LBA, CL = num LBAs, ES:BX = buffer
; Return: Buffer is filled with data
read_floppy:
    push cx
    call lba_to_hts
    pop ax
    mov ah, 2
    pusha
.try_read:
    popa
    pusha
    stc
    int 0x13
    jc .reset
    popa
    ret
.reset:
    call reset_floppy
    jnc .try_read
    mov si, disk_err
    call print
    jmp reboot

; Calculate head, track, and sector for BIOS disk routines
; Params: AX = logical block
; Return: Registers set to BIOS parameters
lba_to_hts:
    push bx
    push ax
    mov bx, ax

    ; Get sector
    mov dx, 0
    div word [sectors_per_track]
    add dl, 1
    mov cl, dl
    mov ax, bx

    ; Get head and track
    mov dx, 0
    div word [sectors_per_track]
    mov dx, 0
    div word [num_sides]
    mov dh, dl
    mov ch, al

    pop ax
    pop bx
    mov dl, 0
    ret

; Reset the floppy controller
; Params: None
; Return: Carry set on error
reset_floppy:
    push ax
    push dx
    mov ax, 0
    mov dl, 0
    stc
    int 0x13
    pop dx
    pop ax
    ret

; Reboots the system
; Params: None
; Return: Doesn't
reboot:
    mov ax, 0
    int 0x16
    mov ax, 0
    int 0x19

; ==============================================================================
; STRINGS AND VARIABLES
; ==============================================================================

kernel_name        db "stage2", 0
disk_err           db "Disk error.", 0xD, 0xA, 0
not_found          db "Stage 2 not found.", 0xD, 0xA, 0

sectors_per_track  dw 18
num_sides          dw 2

inodes_start       dw 0
load_ptr           dw 0x8000

; ==============================================================================
; END OF BOOT SECTOR
; ==============================================================================

times 510-($-$$) db 0
dw 0xAA55
