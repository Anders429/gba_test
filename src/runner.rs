use crate::{Ignore, Outcome, RunningStatus, Status, TestCase, Trial};
use core::{fmt::Display, panic::PanicInfo, slice};
use serde::Serialize;

/// The current write position in SRAM.
static mut SRAM_POS: *mut u8 = 0x0E00_0000 as *mut u8;
/// The start of the SRAM.
const SRAM_START: *mut u8 = 0x0E00_0000 as *mut u8;
/// The end of the SRAM.
const SRAM_END: *mut u8 = 0x0E00_FFFF as *mut u8;

/// The remaining tests to be run.
static mut TESTS: &[&dyn TestCase] = &[];
/// The name of the current test.
static mut TEST_NAME: &str = "";
/// The running status of test execution.
///
/// This will be set to `Failure` if a test fails.
static mut STATUS: RunningStatus = RunningStatus::Success;

/// Write the status to SRAM.
///
/// This will always write a single byte at the start of SRAM.
fn write_status(status: Status) -> Result<(), postcard::Error> {
    let sram =
        unsafe { slice::from_raw_parts_mut(SRAM_START, SRAM_END as usize - SRAM_START as usize) };
    postcard::to_slice(&status, sram)?;
    Ok(())
}

/// Write data to SRAM.
///
/// This increments the current SRAM position, ensuring data is not overwritten on future calls.
fn write_to_sram<T>(value: T) -> Result<(), postcard::Error>
where
    T: Serialize,
{
    // SAFETY: `SRAM_POS` is guaranteed to be less than or equal to `SRAM_END`, and therefore will
    // point to a valid position in SRAM.
    let remaining_sram =
        unsafe { slice::from_raw_parts_mut(SRAM_POS, SRAM_END as usize - SRAM_POS as usize) };
    let used = postcard::to_slice(&value, remaining_sram)?;
    // SAFETY: `SRAM_POS` is only ever accessed on the main thread.
    unsafe {
        SRAM_POS = SRAM_POS.add(used.len());
    }
    Ok(())
}

/// Saves the serialized test result to SRAM.
fn report_test_result<FailedMessage>(outcome: Outcome<FailedMessage>)
where
    FailedMessage: Copy + Display,
{
    // TODO: Remove this unwrap. We shouldn't be panicking in this code!
    write_to_sram(Trial {
        // SAFETY: `TEST_NAME` is only ever accessed on the main thread.
        name: unsafe { TEST_NAME },
        outcome,
    })
    .unwrap();

    if matches!(outcome, Outcome::Failed { .. }) {
        // SAFETY: `STATUS` is only ever accessed on the main thread.
        unsafe {
            STATUS = RunningStatus::Failure;
        }
    }
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

    // TODO: Remove this unwrap.
    write_status(
        // SAFETY: `STATUS` is only ever accessed on the main thread.
        unsafe { STATUS }.into(),
    )
    .unwrap();

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
pub fn test_runner(tests: &'static [&'static dyn TestCase]) {
    // SAFETY: `TESTS` and `SRAM_POS` are only ever accessed on the main thread.
    unsafe {
        TESTS = tests;
        // It seems this value must be reinitialized, otherwise it is always nullptr.
        SRAM_POS = 0x0E00_0000 as *mut u8;
    }

    // Write the current status.
    // TODO: Remove this unwrap.
    write_to_sram(Status::Running).unwrap();

    // Write the number of expected results.
    // TODO: Remove this unwrap.
    write_to_sram(tests.len()).unwrap();

    run_tests();
}
