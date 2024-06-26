ENTRY(__start)

/* The bios reserves the final 256 bytes of iwram for its exclusive use, so we
 * need to avoid writing there */
__bios_reserved_iwram = 256;

MEMORY {
    ewram (w!x) : ORIGIN = 0x2000000, LENGTH = 256K
    iwram (w!x) : ORIGIN = 0x3000000, LENGTH = 32K - __bios_reserved_iwram
    rom (rx)    : ORIGIN = 0x8000000, LENGTH = 32M
}

SECTIONS {
    . = ORIGIN(rom);

    .entrypoint : {
        *(.entrypoint);
        . = ALIGN(4);
    } >rom = 0x00

    .text : {
        *(.text .text.*);
        . = ALIGN(4);
    } >rom = 0x00

    .rodata : {
        *(.rodata .rodata.*);
        . = ALIGN(4);
    } >rom = 0x00

    . = ALIGN(4);
    __iwram_position_in_rom = .;
    .iwram : {
        __iwram_start = ABSOLUTE(.);

        *(.iwram .iwram.*);
        . = ALIGN(4);
        
        __iwram_end = ABSOLUTE(.);
    } > iwram AT>rom = 0x00

    . = ALIGN(4);
    __ewram_position_in_rom = __iwram_position_in_rom + (__iwram_end - __iwram_start);
    .ewram : {
        __ewram_start = ABSOLUTE(.);

        *(.ewram .ewram.*);
        . = ALIGN(4);

        *(.data .data.*);
        . = ALIGN(4);

        __ewram_end = ABSOLUTE(.);
    } > ewram AT>rom = 0x00

    . = ALIGN(4);
    __bss_position_in_rom = __ewram_position_in_rom + (__ewram_end - __ewram_start);
    .bss : {
        __bss_start = ABSOLUTE(.);

        *(.bss .bss.*);
        . = ALIGN(4);

        __bss_end = ABSOLUTE(.);
    } > ewram

    /* This section is for values that need to persist between tests. */
    . = ALIGN(4);
    .noinit : {
        *(.noinit .noinit.*);
        . = ALIGN(4);
    } > ewram

    __iwram_word_copy_count = (__iwram_end - __iwram_start) / 4;
    __ewram_word_copy_count = (__ewram_end - __ewram_start) / 4;
    __bss_word_clear_count = (__bss_end - __bss_start) / 4;
    __ewram_data_end = ADDR(.ewram) + SIZEOF(.ewram) + SIZEOF(.bss) + SIZEOF(.noinit);

    .shstrtab : {
       *(.shstrtab)
    }

        /* debugging sections */
    /* Stabs */
    .stab            0 : { *(.stab) }
    .stabstr         0 : { *(.stabstr) }
    .stab.excl       0 : { *(.stab.excl) }
    .stab.exclstr    0 : { *(.stab.exclstr) }
    .stab.index      0 : { *(.stab.index) }
    .stab.indexstr   0 : { *(.stab.indexstr) }
    .comment         0 : { *(.comment) }
    /* DWARF 1 */
    .debug           0 : { *(.debug) }
    .line            0 : { *(.line) }
    /* GNU DWARF 1 extensions */
    .debug_srcinfo   0 : { *(.debug_srcinfo) }
    .debug_sfnames   0 : { *(.debug_sfnames) }
    /* DWARF 1.1 and DWARF 2 */
    .debug_aranges   0 : { *(.debug_aranges) }
    .debug_pubnames  0 : { *(.debug_pubnames) }
    /* DWARF 2 */
    .debug_info      0 : { *(.debug_info) }
    .debug_abbrev    0 : { *(.debug_abbrev) }
    .debug_line      0 : { *(.debug_line) }
   	.debug_frame     0 : { *(.debug_frame) }
    .debug_str       0 : { *(.debug_str) }
    .debug_loc       0 : { *(.debug_loc) }
    .debug_macinfo   0 : { *(.debug_macinfo) }
    /* SGI/MIPS DWARF 2 extensions */
    .debug_weaknames 0 : { *(.debug_weaknames) }
    .debug_funcnames 0 : { *(.debug_funcnames) }
    .debug_typenames 0 : { *(.debug_typenames) }
    .debug_varnames  0 : { *(.debug_varnames) }

    .debug_ranges   0 : { *(.debug_ranges) }

    /* discard anything not already mentioned */
    /DISCARD/ : { *(*) }
}