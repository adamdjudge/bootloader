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
        KEEP(*(.text.start))
        *(.text .text.*)
    } > LO_RAM :text

    .rodata :
    {
        *(.rodata .rodata.*)
    } > LO_RAM :text

    .data ALIGN(4K) :
    {
        *(.data .data.*)
    } > LO_RAM :data

    .bss ALIGN(4) :
    {
        __bss_start = .;
        *(.bss .bss.*)
    } > LO_RAM :data

    __bss_end = .;
    __loram_top = ORIGIN(LO_RAM) + LENGTH(LO_RAM);

    /DISCARD/ :
    {
        *(.comment)
        *(.eh_frame*)
        *(.note .note.*)
    }
}
