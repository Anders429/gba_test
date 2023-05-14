use crate::{Ignore, Outcome, RawStatus, TestCase, Trial};
use core::{fmt::Display, panic::PanicInfo, ptr};
use postcard::ser_flavors::Flavor;
use serde::Serialize;
use voladdress::{Safe, Unsafe, VolAddress};

/// The current write position in SRAM.
static mut SRAM_POS: *mut u8 = 0x0E00_0000 as *mut u8;
/// The start of the SRAM.
const SRAM_START: *mut u8 = 0x0E00_0000 as *mut u8;
/// The end of the SRAM.
const SRAM_END: *mut u8 = 0x0E00_FFFF as *mut u8;

/// Wait state for interfacing with the GBA Cartridge.
///
/// This must be properly configured prior to interacting with the cartridge. Otherwise, garbage
/// data may be read/written.
const WAITCNT: VolAddress<u16, Safe, Unsafe> = unsafe { VolAddress::new(0x0400_0204) };

/// The remaining tests to be run.
static mut TESTS: &[&dyn TestCase] = &[];
/// The name of the current test.
static mut TEST_NAME: &str = "";

/// Storage within SRAM.
///
/// This struct manages writing serialized data directly to SRAM. It is a `postcard` flavor and can
/// therefore be used in combination with other flavors.
struct Sram {
    /// The current position in SRAM.
    cursor: *mut u8,
}

impl Sram {
    /// Create a new SRAM writer.
    ///
    /// This creates a writer to SRAM at the given pointer location.
    ///
    /// # Safety
    /// The pointer location must be a valid location within SRAM (0x0E00_0000 to 0x0E00_FFFF).
    unsafe fn new(ptr: *mut u8) -> Self {
        Self { cursor: ptr }
    }
}

impl Flavor for Sram {
    /// Returns the position of the cursor at the end of writing.
    type Output = *mut u8;

    fn try_push(&mut self, data: u8) -> postcard::Result<()> {
        // We can write up to and including SRAM_END.
        if self.cursor >= SRAM_END {
            return Err(postcard::Error::SerializeBufferFull);
        }
        // SAFETY: These writes will always be to a valid location.
        unsafe {
            ptr::write_volatile(self.cursor, data);
            self.cursor = self.cursor.add(1);
        }
        Ok(())
    }

    fn finalize(self) -> postcard::Result<Self::Output> {
        Ok(self.cursor)
    }
}

/// Write the status to SRAM.
///
/// This will always write a single byte at the start of SRAM.
fn write_status(status: RawStatus) -> Result<(), postcard::Error> {
    // SAFETY: `SRAM_START` is a valid location in SRAM.
    postcard::serialize_with_flavor(&status, unsafe { Sram::new(SRAM_START) })?;
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
    let new_position = postcard::serialize_with_flavor(&value, unsafe { Sram::new(SRAM_POS) })?;
    // SAFETY: `SRAM_POS` is only ever accessed on the main thread.
    unsafe {
        SRAM_POS = new_position;
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
    write_status(RawStatus::Completed).unwrap();

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
        SRAM_POS = 0x0E00_0000 as *mut u8;

        // Enable writes to SRAM.
        WAITCNT.write(3);
    }

    // Write the current status.
    // TODO: Remove this unwrap.
    write_to_sram(RawStatus::Running).unwrap();

    // Write the number of expected results.
    // TODO: Remove this unwrap.
    write_to_sram(tests.len()).unwrap();

    run_tests();
}
