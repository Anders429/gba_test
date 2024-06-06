#[derive(Debug)]
#[repr(transparent)]
struct DmaControl(u16);

impl DmaControl {
    const fn new() -> Self {
        Self(0)
    }

    const fn with_transfer_32bit(self, value: bool) -> Self {
        DmaControl((self.0 & !(1 << 10)) | ((value as u16) << 10))
    }

    const fn with_enabled(self, value: bool) -> Self {
        DmaControl((self.0 & !(1 << 15)) | ((value as u16) << 15))
    }
}

const DMA_32_BIT_MEMCPY: DmaControl = DmaControl::new()
    .with_transfer_32bit(true)
    .with_enabled(true);
const DMA3_OFFSET: usize = 0x0000_00D4;
const IME_OFFSET: usize = 0x0000_0208;

#[naked]
#[no_mangle]
#[instruction_set(arm::a32)]
#[link_section = ".entrypoint"]
unsafe extern "C" fn __start() -> ! {
    core::arch::asm!(
        "b 1f",
        ".space 0xE0", /* space for header */
        "1:", /* post header */
        "mov r12, #{mmio_base}",
        "add r0, r12, #{waitcnt_offset}",
        "ldr r1, ={waitcnt_setting}",
        "strh r1, [r0]",

        /* iwram copy */
        "ldr r4, =__iwram_word_copy_count",
        "cmp r4, #0",
        "beq 2 f",
        "add r3, r12, #{dma3_offset}",
        "mov r5, #{dma3_setting}",
        "ldr r0, =__iwram_start",
        "ldr r2, =__iwram_position_in_rom",
        "str r2, [r3]", /* source */
        "str r0, [r3, #4]", /* destination */
        "strh r4, [r3, #8]", /* word count */
        "strh r5, [r3, #10]", /* set control bits */
        "2:",

        /* ewram copy */
        "ldr r4, =__ewram_word_copy_count",
        "cmp r4, #0",
        "beq 3 f",
        "add r3, r12, #{dma3_offset}",
        "mov r5, #{dma3_setting}",
        "ldr r0, =__ewram_start",
        "ldr r2, =__ewram_position_in_rom",
        "str r2, [r3]", /* source */
        "str r0, [r3, #4]", /* destination */
        "strh r4, [r3, #8]", /* word count */
        "strh r5, [r3, #10]", /* set control bits */
        "3:",

        /* bss zero */
        "ldr r4, =__bss_word_clear_count",
        "cmp r4, #0",
        "beq 4 f",
        "ldr r0, =__bss_start",
        "mov r2, #0",
        "2:",
        "str r2, [r0], #4",
        "subs r4, r4, #1",
        "bne 2b",
        "4:",

        /* assign the runtime irq handler */
        "ldr r1, ={runtime_irq_handler}",
        "str r1, [r12, #-4]",

         /* call to rust main */
        "ldr r0, =main",
        "bx r0",
        // main shouldn't return, but if it does just SoftReset
        "swi #0",
        mmio_base = const 0x0400_0000,
        waitcnt_offset = const 0x204,
        waitcnt_setting = const 0x4317 /*sram8,r0:3.1,r1:4.2,r2:8.2,no_phi,prefetch*/,
        dma3_offset = const DMA3_OFFSET,
        dma3_setting = const DMA_32_BIT_MEMCPY.0,
        runtime_irq_handler = sym runtime_irq_handler,
        options(noreturn)
    )
}

#[naked]
#[no_mangle]
#[instruction_set(arm::a32)]
#[link_section = ".iwram.runtime.irq.handler"]
unsafe extern "C" fn runtime_irq_handler() {
    // On Entry: r0 = 0x0400_0000 (mmio_base)
    core::arch::asm!(
      /* swap IME off, user can turn it back on if they want */
      "add r12, r0, #{ime_offset}",
      "mov r3, #0",
      "swp r3, r3, [r12]",

      /* Read/Update IE and IF */
      "ldr r0, [r12, #-8]",
      "and r0, r0, r0, LSR #16",
      "strh r0, [r12, #-6]",

      /* Read/Update BIOS_IF */
      "sub  r2, r12, #(0x208+8)",
      "ldrh r1, [r2]",
      "orr  r1, r1, r0",
      "strh r1, [r2]",

      /* Restore initial IME setting and return */
      "swp r3, r3, [r12]",
      "bx lr",
      ime_offset = const IME_OFFSET,
      options(noreturn)
    )
}

#[no_mangle]
pub fn __sync_synchronize() {}
