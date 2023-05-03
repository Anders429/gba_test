#![no_std]

use core::{fmt, panic::PanicInfo};
use serde::{ser::{Serialize, Serializer, SerializeStruct}, de, de::{Deserialize, Deserializer, SeqAccess, Unexpected, Visitor}};

/// The remaining tests to be run.
static mut TESTS: &[&dyn TestCase] = &[];

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
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_u8(*self as u8)
    }
}

impl<'de> Deserialize<'de> for Outcome {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        struct OutcomeVisitor;

        impl<'de> Visitor<'de> for OutcomeVisitor {
            type Value = Outcome;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a byte containing the value 0 or 1")
            }

            fn visit_u8<E>(self, value: u8) -> Result<Self::Value, E> where E: de::Error {
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
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut trial = serializer.serialize_struct("Trial", 2)?;

        trial.serialize_field("name", self.name)?;
        trial.serialize_field("outcome", &self.outcome)?;

        trial.end()
    }
}

impl<'de> Deserialize<'de> for Trial<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        struct TrialVisitor;

        impl<'de> Visitor<'de> for TrialVisitor {
            type Value = Trial<'de>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Trial")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de> {
                let name = seq.next_element()?.ok_or(de::Error::missing_field("name"))?;
                let outcome = seq.next_element()?.ok_or(de::Error::missing_field("outcome"))?;

                Ok(Trial {
                    name,
                    outcome,
                })
            }
        }

        deserializer.deserialize_struct("Trial", &["name", "outcome"], TrialVisitor)
    }
}

fn report_test_result(outcome: Outcome) {
    todo!("write the trial to the next location in SRAM")
}

/// Runs the remaining tests.
/// 
/// The current test being executed is tracked using global state. This allows the runner to
/// recover when a test panics.
fn run_tests() -> ! {
    // SAFETY: `TESTS` is only ever mutated on the main thread.
    while let Some((test, tests)) = unsafe {TESTS}.split_first() {
        // SAFETY: `TESTS` is only ever mutated on the main thread.
        unsafe {
            TESTS = tests;
        }
        test.run();
    }

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}

/// A test runner to execute tests as a Game Boy Advance ROM.
pub fn test_runner(tests: &'static [&'static dyn TestCase]) {
    // SAFETY: `TESTS` is only ever mutated on the main thread.
    unsafe {
        TESTS = tests;
    }

    run_tests();
}
