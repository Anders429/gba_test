#![no_std]

use bincode::{config, error::EncodeError, serde::encode_into_slice};
use core::{fmt, panic::PanicInfo, slice};
use serde::{
    de,
    de::{Deserialize, Deserializer, SeqAccess, Unexpected, Visitor},
    ser::{Serialize, SerializeStruct, Serializer},
};

/// The current write position in SRAM.
static mut SRAM_POS: *mut u8 = 0x0E00_0000 as *mut u8;
/// The end of the SRAM.
const SRAM_END: *mut u8 = 0x0E00_FFFF as *mut u8;

/// Configuration for bincode encoding.
///
/// This definition ensures that the same configuration is used across all code.
const BINCODE_CONFIG: config::Configuration<config::LittleEndian, config::Fixint, config::NoLimit> =
    config::standard().with_fixed_int_encoding();

/// The remaining tests to be run.
static mut TESTS: &[&dyn TestCase] = &[];
/// The name of the current test.
static mut TEST_NAME: &str = "";

/// Defines a test case executable by the test runner.
pub trait TestCase {
    /// The name of the test.
    fn name(&self) -> &str;

    /// The actual test itself.
    ///
    /// If this method panics, the test is considered a failure. Otherwise, the test is considered
    /// to have passed.
    fn run(&self);
}

/// The outcome of a test.
#[derive(Clone, Copy, Debug)]
pub enum Outcome {
    Passed,
    Failed,
}

impl Serialize for Outcome {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

impl<'de> Deserialize<'de> for Outcome {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct OutcomeVisitor;

        impl<'de> Visitor<'de> for OutcomeVisitor {
            type Value = Outcome;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a byte containing the value 0 or 1")
            }

            fn visit_u8<E>(self, value: u8) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match value {
                    0 => Ok(Outcome::Passed),
                    1 => Ok(Outcome::Failed),
                    _ => Err(E::invalid_value(Unexpected::Unsigned(value.into()), &self)),
                }
            }
        }

        deserializer.deserialize_u8(OutcomeVisitor)
    }
}

/// A single test result.
#[derive(Debug)]
pub struct Trial<'a> {
    /// The name of the test.
    pub name: &'a str,
    /// The test's outcome.
    pub outcome: Outcome,
}

impl<'a> Serialize for Trial<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut trial = serializer.serialize_struct("Trial", 2)?;

        trial.serialize_field("name", self.name)?;
        trial.serialize_field("outcome", &self.outcome)?;

        trial.end()
    }
}

impl<'de> Deserialize<'de> for Trial<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TrialVisitor;

        impl<'de> Visitor<'de> for TrialVisitor {
            type Value = Trial<'de>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Trial")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let name = seq
                    .next_element()?
                    .ok_or(de::Error::missing_field("name"))?;
                let outcome = seq
                    .next_element()?
                    .ok_or(de::Error::missing_field("outcome"))?;

                Ok(Trial { name, outcome })
            }
        }

        deserializer.deserialize_struct("Trial", &["name", "outcome"], TrialVisitor)
    }
}

/// Write data to SRAM.
///
/// This increments the current SRAM position, ensuring data is not overwritten on future calls.
fn write_to_sram<T>(value: T) -> Result<(), EncodeError>
where
    T: Serialize,
{
    // SAFETY: `SRAM_POS` is guaranteed to be less than or equal to `SRAM_END`, and therefore will
    // point to a valid position in SRAM.
    let remaining_sram =
        unsafe { slice::from_raw_parts_mut(SRAM_POS, SRAM_END as usize - SRAM_POS as usize) };
    let encoded_bytes = encode_into_slice(value, remaining_sram, BINCODE_CONFIG)?;
    // SAFETY: `SRAM_POS` is only ever accessed on the main thread.
    unsafe {
        SRAM_POS = SRAM_POS.add(encoded_bytes);
    }
    Ok(())
}

/// Saves the serialized test result to SRAM.
fn report_test_result(outcome: Outcome) {
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
        test.run();
        report_test_result(Outcome::Passed);
    }

    // TODO: Sleep.
    loop {}
}

/// Defines a panic handler for running tests.
///
/// This panic handler is configured to continue execution after a panic, allowing tests to
/// continue being run after the current test panics.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    report_test_result(Outcome::Failed);
    run_tests()
}

/// A test runner to execute tests as a Game Boy Advance ROM.
pub fn test_runner(tests: &'static [&'static dyn TestCase]) {
    // SAFETY: `TESTS` is only ever mutated on the main thread.
    unsafe {
        TESTS = tests;
    }

    run_tests();
}
