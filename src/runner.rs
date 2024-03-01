//! Logic for running tests on a Game Boy Advance.
//!
//! This module contains the actual test runner along with its associated utility functions. The
//! code here should only ever be run on a Game Boy Advance, and the safety considerations do not
//! apply for other targets.

use crate::{TestCase, test_case::Ignore, Outcome, Outcomes, ui};
use core::{arch::asm, fmt::Display, ptr::addr_of, panic::PanicInfo};

// TODO: Make these more type-safe.
const DISPSTAT: *mut u16 = 0x0400_0004 as *mut u16;
const IME: *mut bool = 0x0400_0208 as *mut bool;
const IE: *mut u16 = 0x0400_0200 as *mut u16;

/// The index of the next test to be run.
#[link_section = ".noinit"]
static mut INDEX: usize = 0;

#[link_section = ".noinit"]
static mut OUTCOMES: Option<Outcomes> = None;

fn store_outcome<Data>(outcome: Outcome<Data>) where Data: Display {
    // TODO: Handle cases where `OUTCOMES` is not present.
    if let Some(outcomes) = unsafe {OUTCOMES.as_mut()} {
        outcomes.push_outcome(outcome);
    }
}

/// Perform a soft reset on the GBA.
/// 
/// This resets the entire system, although it does not clear `.noinit` data in EWRAM. This means
/// that the current testing context and previous results will persist through this reset.
#[inline]
#[instruction_set(arm::t32)]
fn reset() -> ! {
    unsafe {
        asm! {
            // "ldr r0, #128",
            // "swi #0x01",
            "swi #0x00",
            options(noreturn),
        }
    };
}

/// This calls SWI 0x27 (CustomHalt), triggering a halt (equivalent to SWI 0x02) until the next
/// interrupt.
/// 
/// SWI 0x27 is a nonstandard (and undocumented) BIOS instruction, with all official documentation
/// instead recommending to use SWI 0x02 and SWI 0x03. This means we are reasonably safe to use it
/// here specifically for reporting the test result to an emulated test runner that is listening
/// for this BIOS instruction. For more information about BIOS instructions, consult GBATEK.
/// 
/// The intention is for this to be used with `mgba-rom-test` (or another similar emulator
/// specialized for testing), configured to listen for SWI 0x27 with `r0` containing the exit code.
/// The standard in this library is to return `0` as a successful exit code, with any other value
/// indicating a test failure. With this behavior, tests can be run using `mgba-rom-test` or
/// similar in CI.
/// 
/// Outside of a test emulator, this function should not halt the display of the test results.
/// Since halting only persists until the next interrupt, the program will continue as soon as the
/// next vblank interrupt is triggered.
#[no_mangle]
#[inline]
#[instruction_set(arm::t32)]
fn report_result(result: usize) {
    unsafe {
        asm! {
            "swi #0x27",
            in("r0") result,
            in("r2") 0x00,
        }
    }
}

/// Defines a panic handler for running tests.
///
/// This panic handler is configured to continue execution after a panic, allowing tests to
/// continue being run after the current test panics.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // TODO: Need to handle when this is called outside of the test runner.
    log::info!("test failed");
    store_outcome(Outcome::Failed(info));

    // Soft resetting the system allows us to recover from the panicked state and continue testing.
    reset()
}

/// A test runner to execute tests as a Game Boy Advance ROM.
pub fn runner(tests: &'static [&'static dyn TestCase]) {
    mgba_log::init();

    if unsafe {OUTCOMES.is_none()} {
        extern "C" {
            static __ewram_data_end: u8;
        }
        unsafe {OUTCOMES = Some(Outcomes::new((addr_of!(__ewram_data_end) as usize) as *mut u8, tests.len()));}
    }
    
    let index = unsafe {INDEX};
    for test in &tests[index..] {
        unsafe {INDEX += 1;}
        log::info!("running test: {}", test.name());
        match test.ignore() {
            Ignore::Yes => {
                log::info!("test ignored");
                store_outcome(Outcome::<&str>::Ignored);
            }
            Ignore::No => {
                test.run();
                log::info!("test passed");
                store_outcome(Outcome::<&str>::Passed);
            }
        }
        // Reset the system to ensure tests are not accidentally reliant on each other.
        reset();
    }

    log::info!("tests finished");

    // Enable interrupts.
    unsafe {
        DISPSTAT.write_volatile(8);
        IE.write_volatile(1);
        IME.write(true);
    }

    // Report the test result.
    //
    // On normal hardware and non-test emulators, this will just temporarily halt the program until
    // the next vblank. On test emulators (such as `mgba-rom-test`), this will exit the emulator
    // with the `return_value` as the program's exit code if the emulator has been configured to
    // listen for SWI 0x27 with the exit code on `r0`.
    report_result(unsafe {OUTCOMES.as_ref().unwrap().iter_outcomes().any(|outcome| matches!(outcome, Outcome::Failed(_)))} as usize);

    ui::run(tests, unsafe {OUTCOMES.as_ref().unwrap()})
}
