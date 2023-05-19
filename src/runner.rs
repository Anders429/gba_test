//! Logic for running tests on a Game Boy Advance.
//!
//! This module contains the actual test runner along with its associated utility functions. The
//! code here should only ever be run on a Game Boy Advance, and the safety considerations do not
//! apply for other targets.

use crate::{display::SerializeDisplay, flavors::Sram, Ignore, Outcome, TestCase, Trial};
use core::{fmt::Display, panic::PanicInfo, ptr};
use serde::Serialize;
use voladdress::{Safe, Unsafe, VolAddress};

/// The current write position in SRAM.
///
/// This value begins at 1 byte past the start of SRAM, as the first byte is reserved for the
/// `Result` variant. The `Result` variant is not written until the tests have finished, which
/// signals that the written data is now valid `postcard` data.
static mut SRAM_POS: *mut u8 = 0x0E00_0001 as *mut u8;
/// The start of the SRAM.
const SRAM_START: *mut u8 = 0x0E00_0000 as *mut u8;

/// Wait state for interfacing with the GBA Cartridge.
///
/// This must be properly configured prior to interacting with the cartridge. Otherwise, garbage
/// data may be read/written.
const WAITCNT: VolAddress<u16, Safe, Unsafe> = unsafe { VolAddress::new(0x0400_0204) };

/// The remaining tests to be run.
static mut TESTS: &[&dyn TestCase] = &[];
/// The name of the current test.
static mut TEST_NAME: &str = "";

/// Write data at the beginning of SRAM.
///
/// This will overwrite whatever data is already written there.
fn write_to_sram<T>(value: T) -> Result<(), postcard::Error>
where
    T: Serialize,
{
    postcard::serialize_with_flavor(&value, unsafe { Sram::new(SRAM_START) })?;
    Ok(())
}

/// Write data to the end of SRAM.
///
/// This increments the current SRAM position, ensuring data is not overwritten on future calls.
fn append_to_sram<T>(value: T) -> Result<(), postcard::Error>
where
    T: Serialize,
{
    // SAFETY: `SRAM_POS` is guaranteed to be less than or equal to `SRAM_END`, and therefore will
    // point to a valid position in SRAM.
    let new_position = postcard::serialize_with_flavor(&value, unsafe { Sram::new(SRAM_POS) })?;
    // SAFETY: `SRAM_POS` is only ever accessed on the main thread.
    unsafe {
        SRAM_POS = new_position;
    }
    Ok(())
}

/// Handle an error that occurred during test execution.
///
/// We can't panic in this context, as that would cause the code to loop until the stack overflows.
/// Instead, this function attempts to write the error to SRAM.
fn handle_error<E>(error: E)
where
    E: Display,
{
    // If writing to SRAM fails here, there is not much else that can be done. Unwrapping the
    // result would lead to a panic loop, causing a stack overflow, so we simply ignore the error
    // if there is one.
    #[allow(unused_must_use)]
    {
        write_to_sram(Result::<(), _>::Err(SerializeDisplay(error)));
    }
}

/// Saves the serialized test result to SRAM.
fn report_test_result<FailedMessage>(outcome: Outcome<FailedMessage>)
where
    FailedMessage: Copy + Display,
{
    append_to_sram(Trial {
        // SAFETY: `TEST_NAME` is only ever accessed on the main thread.
        name: unsafe { TEST_NAME },
        outcome,
    })
    .unwrap_or_else(handle_error);
}

/// Runs the remaining tests.
///
/// The current test being executed is tracked using global state. This allows the runner to
/// recover when a test panics.
fn run_tests() -> ! {
    // SAFETY: `TESTS` is only ever mutated on the main thread.
    while let Some((test, tests)) = unsafe { TESTS }.split_first() {
        // SAFETY: `TESTS` and `TEST_NAME` are only ever mutated on the main thread.
        unsafe {
            TESTS = tests;
            TEST_NAME = test.name();
        }

        match test.ignore() {
            Ignore::No => {
                test.run();
                report_test_result(Outcome::<&str>::Passed);
            }
            Ignore::Yes => report_test_result(Outcome::<&str>::Ignored),
        }
    }

    write_to_sram(Ok::<(), ()>(())).unwrap_or_else(handle_error);

    unsafe {
        core::arch::asm!("swi #0x03",);
    }
    loop {}
}

/// Defines a panic handler for running tests.
///
/// This panic handler is configured to continue execution after a panic, allowing tests to
/// continue being run after the current test panics.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    report_test_result(Outcome::Failed { message: info });
    run_tests()
}

/// A test runner to execute tests as a Game Boy Advance ROM.
#[cfg_attr(
    doc_cfg,
    doc(cfg(all(feature = "runner", target = "thumbv4t-none-eabi")))
)]
pub fn runner(tests: &'static [&'static dyn TestCase]) {
    // SAFETY: `TESTS`, `SRAM_POS`, and `WAITCNT` are only ever accessed on the main thread.
    unsafe {
        TESTS = tests;
        // It seems this value must be reinitialized, otherwise it is always nullptr.
        SRAM_POS = 0x0E00_0001 as *mut u8;

        // Enable writes to SRAM.
        WAITCNT.write(3);
    }

    // Write the number of expected results.
    append_to_sram(tests.len()).unwrap_or_else(handle_error);

    run_tests();
}
