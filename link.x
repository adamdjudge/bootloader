ENTRY(_start)

SECTIONS
{
    . = 0x8000;

    .text :
    {
        KEEP(*(.text.start))
        *(.text .text.*)
    }

    .rodata :
    {
        *(.rodata)
    }

    .data :
    {
        *(.data)
    }

    .bss :
    {
        __bss_start = .;
        *(.bss)
    }

    __bss_end = .;

    /DISCARD/ :
    {
        *(.comment)
    }
}
