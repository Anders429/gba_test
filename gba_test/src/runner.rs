//! Logic for running tests on a Game Boy Advance.
//!
//! This module contains the actual test runner along with its associated utility functions. The
//! code here should only ever be run on a Game Boy Advance, and the safety considerations do not
//! apply for other targets.

use crate::{log, test_case::Ignore, ui, Outcome, ShouldPanic, TestCase, Tests};
use core::{arch::asm, fmt::Display, mem::MaybeUninit, panic::PanicInfo, ptr::addr_of};

// TODO: Make these more type-safe.
const DISPSTAT: *mut u16 = 0x0400_0004 as *mut u16;
const IME: *mut bool = 0x0400_0208 as *mut bool;
const IE: *mut u16 = 0x0400_0200 as *mut u16;

#[link_section = ".noinit"]
static mut INITIALIZED: bool = false;
#[link_section = ".noinit"]
static mut TESTS: MaybeUninit<Tests> = MaybeUninit::uninit();

/// Stores the outcome of the current test.
///
/// # Panics
/// If `TESTS` has not been initialized. Also if there is no currently active test to have an
/// outcome be reported on.
fn store_outcome<Data>(outcome: Outcome<Data>)
where
    Data: Display,
{
    if unsafe { INITIALIZED } {
        unsafe { TESTS.assume_init_mut().complete_test(outcome) };
    } else {
        panic!("attempted to write outcome, but `TESTS` is not initialized");
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
        // Resets everything besides EWRAM and IWRAM.
        asm! {
            "swi #0x01",
            "swi #0x00",
            in("r0") 0xFC,
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
    if unsafe { INITIALIZED } {
        if let Some(test) = unsafe { TESTS.assume_init_ref().current_test() } {
            // Panicked while executing a test. Handle the result.
            match test.should_panic() {
                ShouldPanic::No => {
                    log::info!("test failed");
                    store_outcome(Outcome::Failed(info));
                }
                ShouldPanic::Yes => {
                    log::info!("test passed");
                    store_outcome(Outcome::<&str>::Passed);
                }
            }

            // Soft resetting the system allows us to recover from the panicked state and continue testing.
            reset()
        }
    }

    // Panicked outside of executing a test.
    log::error!("{info}");
    ui::panic::display(info);
}

/// A test runner to execute tests as a Game Boy Advance ROM.
///
/// This runner can be used with the unstable
/// [`custom_test_frameworks`](https://doc.rust-lang.org/unstable-book/language-features/custom-test-frameworks.html)
/// feature. Simply provide the runner to the `#[test_runner]` attribute and call the test harness.
///
/// The test runner will never return. It first executes the tests, and then displays a user
/// interface to browse test results.
///
/// # Example
/// ```
/// #![no_std]
/// #![cfg_attr(test, no_main)]
/// #![cfg_attr(test, feature(custom_test_frameworks))]
/// #![cfg_attr(test, test_runner(gba_test::runner))]
/// #![cfg_attr(test, reexport_test_harness_main = "test_harness")]
///
/// #[cfg(test)]
/// #[no_mangle]
/// pub fn main() {
///     test_harness()
/// }
///
/// pub fn add(left: usize, right: usize) -> usize {
///     left + right
/// }
///
/// #[cfg(test)]
/// mod tests {
///     use gba_test::test;
///
///     #[test]
///     fn it_works() {
///         let result = add(2, 2);
///         assert_eq!(result, 4);
///     }
/// }
/// ```
pub fn runner(tests: &'static [&'static dyn TestCase]) -> ! {
    if unsafe { !INITIALIZED } {
        // Use the remaining unused space in ewram as our data heap.
        extern "C" {
            static __ewram_data_end: u8;
        }
        unsafe {
            TESTS = MaybeUninit::new(Tests::new(
                tests,
                (addr_of!(__ewram_data_end) as usize) as *mut u8,
            ));
            INITIALIZED = true;
        }
    }

    if let Some(test) = unsafe { TESTS.assume_init_mut().start_test() } {
        log::info!("running test: {}", test.name());
        match test.ignore() {
            Ignore::Yes | Ignore::YesWithMessage(_) => {
                log::info!("test ignored");
                store_outcome(Outcome::<&str>::Ignored);
            }
            Ignore::No => {
                test.run();
                match test.should_panic() {
                    ShouldPanic::No => {
                        log::info!("test passed");
                        store_outcome(Outcome::<&str>::Passed);
                    }
                    ShouldPanic::Yes => {
                        log::info!("test failed");
                        store_outcome(Outcome::Failed("note: test did not panic as expected"))
                    }
                }
            }
        }
        // Reset the system to ensure tests are not accidentally reliant on each other.
        //
        // Note that this will reset the program. This stops execution at this point and calls
        // `main()` all over again.
        reset();
    }

    log::info!("tests finished");
    let outcomes = unsafe { TESTS.assume_init_ref() }.outcomes();

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
    report_result(
        outcomes
            .iter()
            .any(|(_, outcome)| matches!(outcome, Outcome::Failed(_))) as usize,
    );

    ui::run(outcomes)
}
