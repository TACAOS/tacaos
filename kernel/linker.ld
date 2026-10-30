OUTPUT_FORMAT(elf64-x86-64)
ENTRY(_start)

PHDRS
{
    text PT_LOAD FLAGS(5);
    rodata PT_LOAD FLAGS(4);
    data PT_LOAD FLAGS(6);
}

SECTIONS
{
    . = 0x100000;
    __kernel_start = .;

    .text : ALIGN(4K) { *(.text._start) *(.text .text.*) } :text
    .rodata : ALIGN(4K) { *(.rodata .rodata.*) } :rodata
    .data : ALIGN(4K) { *(.data .data.*) } :data
    .bss : ALIGN(4K) { *(COMMON) *(.bss .bss.*) } :data

    __kernel_end = .;

    /DISCARD/ : { *(.comment) *(.note .note.*) }
}
