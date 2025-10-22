/* Layout for PC low memory between the boot sector code and BIOS area */
MEMORY
{
    LO_RAM : ORIGIN = 0x8000, LENGTH = 0xA0000 - 0x8000
}

ENTRY(_start)

SECTIONS
{
    .text :
    {
        KEEP(*(.text.start))
        *(.text .text.*)
    } > LO_RAM

    .rodata :
    {
        *(.rodata .rodata.*)
    } > LO_RAM

    .data :
    {
        *(.data .data.*)
    } > LO_RAM

    .bss :
    {
        __bss_start = .;
        *(.bss .bss.*)
    } > LO_RAM

    __bss_end = .;

    /DISCARD/ :
    {
        *(.comment)
    }
}
