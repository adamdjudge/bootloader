/* Layout for PC low memory between the boot sector code and BIOS area */
MEMORY
{
    LO_RAM : ORIGIN = 0x8000, LENGTH = 0xA0000 - 0x8000
}

PHDRS
{
    text PT_LOAD;
    data PT_LOAD;
}

ENTRY(_start)

SECTIONS
{
    .text :
    {
        __bootloader_base = .;
        KEEP(*(.text.start))
        *(.text .text.*)
    } > LO_RAM :text

    .rodata :
    {
        *(.rodata .rodata.*)
    } > LO_RAM :text

    .data ALIGN(4K) :
    {
        __bootloader_data = .;
        *(.data .data.*)
    } > LO_RAM :data

    .bss ALIGN(4) :
    {
        __bss_start = .;
        *(.bss .bss.*)
    } > LO_RAM :data

    __bss_end = .;
    __bootloader_top = .;

    /DISCARD/ :
    {
        *(.comment)
        *(.eh_frame*)
        *(.note .note.*)
    }

    __loram_top = ORIGIN(LO_RAM) + LENGTH(LO_RAM);
}
