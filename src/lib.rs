#![no_std]
#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![cfg_attr(
    all(feature = "runner", target = "thumbv4t-none-eabi"),
    feature(panic_info_message)
)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(all(feature = "runner", any(target = "thumbv4t-none-eabi", doc)))]
mod runner;

#[cfg(all(feature = "runner", any(target = "thumbv4t-none-eabi", doc)))]
pub use runner::test_runner;

#[cfg(feature = "gba_test_macros")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "macros")))]
pub use gba_test_macros::test;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;
#[cfg(feature = "serde")]
use core::fmt;
use core::fmt::Display;
use core::str;
use serde::de::VariantAccess;
#[cfg(feature = "serde")]
use serde::{
    de,
    de::{
        Deserialize, Deserializer, EnumAccess, Error as _, MapAccess, SeqAccess, Unexpected,
        Visitor,
    },
    ser::{Serialize, SerializeStruct, SerializeStructVariant, Serializer},
};

/// Defines whether a test should be ignored or not.
#[derive(Clone, Copy, Debug)]
pub enum Ignore {
    /// The test should be run.
    No,
    /// The test should not be run.
    Yes,
}

/// Defines a test case executable by the test runner.
pub trait TestCase {
    /// The name of the test.
    fn name(&self) -> &str;

    /// The actual test itself.
    ///
    /// If this method panics, the test is considered a failure. Otherwise, the test is considered
    /// to have passed.
    fn run(&self);

    /// Whether the test should be excluded or not.
    ///
    /// If this method returns true, the test function will not be run at all (but it will still be
    /// compiled). This allows for time-consuming or expensive tests to be conditionally disabled.
    fn ignore(&self) -> Ignore;
}

/// A standard test.
///
/// This struct is created by the `#[test]` attribute. This struct is not to be used directly and
/// is not considered part of the public API. If you want to use a similar struct, you should
/// define one locally and implement `TestCase` for it directly.
#[doc(hidden)]
pub struct Test {
    /// The name of the test.
    pub name: &'static str,
    /// The test function itself.
    pub test: fn(),
    /// Whether the test should be excluded.
    ///
    /// This is set by the `#[ignore]` attribute.
    pub ignore: Ignore,
}

impl TestCase for Test {
    fn name(&self) -> &str {
        self.name
    }

    fn run(&self) {
        (self.test)()
    }

    fn ignore(&self) -> Ignore {
        self.ignore
    }
}

struct SerializeDisplay<T>(T);

impl<T> Serialize for SerializeDisplay<T>
where
    T: Display,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(&self.0)
    }
}

/// The outcome of a test.
#[derive(Debug, Eq, PartialEq)]
pub enum Outcome<FailedMessage> {
    /// The test passed.
    Passed,
    /// The test failed.
    Failed { message: FailedMessage },
    /// The test was excluded from the test run.
    Ignored,
}

#[cfg(feature = "serde")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
impl<FailedMessage> Serialize for Outcome<FailedMessage>
where
    FailedMessage: Display,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Passed => serializer.serialize_unit_variant("Outcome", 0, "Passed"),
            Self::Failed { message } => {
                let mut struct_variant =
                    serializer.serialize_struct_variant("Outcome", 1, "Failed", 1)?;
                struct_variant.serialize_field("message", &SerializeDisplay(message))?;
                struct_variant.end()
            }
            Self::Ignored => serializer.serialize_unit_variant("Outcome", 2, "Ignored"),
        }
    }
}

#[cfg(feature = "serde")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
impl<'de> Deserialize<'de> for Outcome<&'de str> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Variant {
            Passed,
            Failed,
            Ignored,
        }

        impl<'de> Deserialize<'de> for Variant {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct VariantVisitor;

                impl<'de> Visitor<'de> for VariantVisitor {
                    type Value = Variant;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`Passed`, `Failed`, or `Ignored`")
                    }

                    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            0 => Ok(Variant::Passed),
                            1 => Ok(Variant::Failed),
                            2 => Ok(Variant::Ignored),
                            _ => Err(E::invalid_value(Unexpected::Unsigned(value.into()), &self)),
                        }
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "Passed" => Ok(Variant::Passed),
                            "Failed" => Ok(Variant::Failed),
                            "Ignored" => Ok(Variant::Ignored),
                            _ => Err(E::unknown_variant(value, VARIANTS)),
                        }
                    }

                    fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            b"Passed" => Ok(Variant::Passed),
                            b"Failed" => Ok(Variant::Failed),
                            b"Ignored" => Ok(Variant::Ignored),
                            _ => {
                                if let Ok(value) = str::from_utf8(value) {
                                    Err(E::unknown_variant(value, VARIANTS))
                                } else {
                                    Err(E::invalid_value(Unexpected::Bytes(value), &self))
                                }
                            }
                        }
                    }
                }

                deserializer.deserialize_identifier(VariantVisitor)
            }
        }

        enum FailedField {
            Message,
        }

        const FAILED_FIELDS: &[&str] = &["message"];

        impl<'de> Deserialize<'de> for FailedField {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FailedFieldVisitor;

                impl<'de> Visitor<'de> for FailedFieldVisitor {
                    type Value = FailedField;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`message`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "message" => Ok(FailedField::Message),
                            _ => Err(E::unknown_field(value, FAILED_FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FailedFieldVisitor)
            }
        }

        struct FailedVisitor;

        impl<'de> Visitor<'de> for FailedVisitor {
            type Value = Outcome<&'de str>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("variant Outcome::Failed")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                Ok(Outcome::Failed {
                    message: seq
                        .next_element()?
                        .ok_or_else(|| de::Error::invalid_length(0, &self))?,
                })
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut message = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        FailedField::Message => {
                            if message.is_some() {
                                return Err(A::Error::duplicate_field("message"));
                            }
                            message = Some(map.next_value()?);
                        }
                    }
                }

                Ok(Outcome::Failed {
                    message: message.ok_or_else(|| A::Error::missing_field("message"))?,
                })
            }
        }

        struct OutcomeVisitor;

        impl<'de> Visitor<'de> for OutcomeVisitor {
            type Value = Outcome<&'de str>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("enum Outcome")
            }

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                match data.variant()? {
                    (Variant::Passed, variant) => variant.unit_variant().and(Ok(Outcome::Passed)),
                    (Variant::Failed, variant) => {
                        variant.struct_variant(FAILED_FIELDS, FailedVisitor)
                    }
                    (Variant::Ignored, variant) => variant.unit_variant().and(Ok(Outcome::Ignored)),
                }
            }
        }

        const VARIANTS: &[&str] = &["Passed", "Failed", "Ignored"];

        deserializer.deserialize_enum("Outcome", VARIANTS, OutcomeVisitor)
    }
}

/// A single test result.
#[derive(Debug, Eq, PartialEq)]
pub struct Trial<'a, FailedMessage> {
    /// The name of the test.
    pub name: &'a str,
    /// The test's outcome.
    pub outcome: Outcome<FailedMessage>,
}

#[cfg(feature = "serde")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
impl<'a, FailedMessage> Serialize for Trial<'a, FailedMessage>
where
    FailedMessage: Display,
{
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

#[cfg(feature = "serde")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
impl<'de> Deserialize<'de> for Trial<'de, &'de str> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TrialVisitor;

        impl<'de> Visitor<'de> for TrialVisitor {
            type Value = Trial<'de, &'de str>;

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

/// Status of test execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Status {
    /// Tests are currently running.
    Running,
    /// All tests either passed or were ignored.
    Success,
    /// One or more tests failed.
    Failure,
}

#[cfg(feature = "serde")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
impl Serialize for Status {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Running => serializer.serialize_unit_variant("Status", 0, "Running"),
            Self::Success => serializer.serialize_unit_variant("Status", 1, "Success"),
            Self::Failure => serializer.serialize_unit_variant("Status", 2, "Failure"),
        }
    }
}

#[cfg(feature = "serde")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
impl<'de> Deserialize<'de> for Status {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Variant {
            Running,
            Success,
            Failure,
        }

        impl<'de> Deserialize<'de> for Variant {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct VariantVisitor;

                impl<'de> Visitor<'de> for VariantVisitor {
                    type Value = Variant;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`Running`, `Success`, or `Failure`")
                    }

                    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            0 => Ok(Variant::Running),
                            1 => Ok(Variant::Success),
                            2 => Ok(Variant::Failure),
                            _ => Err(E::invalid_value(Unexpected::Unsigned(value.into()), &self)),
                        }
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "Running" => Ok(Variant::Running),
                            "Success" => Ok(Variant::Success),
                            "Failure" => Ok(Variant::Failure),
                            _ => Err(E::unknown_variant(value, VARIANTS)),
                        }
                    }

                    fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            b"Running" => Ok(Variant::Running),
                            b"Success" => Ok(Variant::Success),
                            b"Failure" => Ok(Variant::Failure),
                            _ => {
                                if let Ok(value) = str::from_utf8(value) {
                                    Err(E::unknown_variant(value, VARIANTS))
                                } else {
                                    Err(E::invalid_value(Unexpected::Bytes(value), &self))
                                }
                            }
                        }
                    }
                }

                deserializer.deserialize_identifier(VariantVisitor)
            }
        }

        struct StatusVisitor;

        impl<'de> Visitor<'de> for StatusVisitor {
            type Value = Status;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("enum Status")
            }

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                match data.variant()? {
                    (Variant::Running, variant) => variant.unit_variant().and(Ok(Status::Running)),
                    (Variant::Success, variant) => variant.unit_variant().and(Ok(Status::Success)),
                    (Variant::Failure, variant) => variant.unit_variant().and(Ok(Status::Failure)),
                }
            }
        }

        const VARIANTS: &[&str] = &["Running", "Success", "Failure"];

        deserializer.deserialize_enum("Status", VARIANTS, StatusVisitor)
    }
}

/// Status of the currently-running tests.
///
/// This is a subset of `Status`, as it only expresses the success or failure of the tests. This
/// can be converted to a full `Status` using the `From` implementation.
#[cfg(all(feature = "runner", any(target = "thumbv4t-none-eabi", doc, test)))]
#[derive(Clone, Copy, Debug)]
enum RunningStatus {
    Success,
    Failure,
}

#[cfg(all(feature = "runner", any(target = "thumbv4t-none-eabi", doc, test)))]
impl From<RunningStatus> for Status {
    fn from(running_status: RunningStatus) -> Self {
        match running_status {
            RunningStatus::Success => Self::Success,
            RunningStatus::Failure => Self::Failure,
        }
    }
}

/// Contains information about the entire test run.
///
/// When tests are run on the Game Boy Advance, the results available in SRAM are an encoded
/// `bincode` representation of this struct.
#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
#[derive(Debug, Eq, PartialEq)]
pub struct Conclusion<'a> {
    pub status: Status,
    pub trials: Vec<Trial<'a, &'a str>>,
}

#[cfg(all(feature = "alloc", feature = "serde"))]
#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
impl<'a> Serialize for Conclusion<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut conclusion = serializer.serialize_struct("Conclusion", 2)?;

        conclusion.serialize_field("status", &self.status)?;
        conclusion.serialize_field("trials", &self.trials)?;

        conclusion.end()
    }
}

#[cfg(all(feature = "alloc", feature = "serde"))]
#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
impl<'de> Deserialize<'de> for Conclusion<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ConclusionVisitor;

        impl<'de> Visitor<'de> for ConclusionVisitor {
            type Value = Conclusion<'de>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Conclusion")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let status = seq
                    .next_element()?
                    .ok_or(de::Error::missing_field("status"))?;
                let trials = seq
                    .next_element()?
                    .ok_or(de::Error::missing_field("trials"))?;

                Ok(Conclusion { status, trials })
            }
        }

        const FIELDS: &[&str] = &["status", "trials"];

        deserializer.deserialize_struct("Conclusion", FIELDS, ConclusionVisitor)
    }
}

#[cfg(test)]
#[cfg(feature = "runner")]
mod tests {
    use super::{RunningStatus, Status};

    #[test]
    fn running_status_success_into_status() {
        assert_eq!(Status::from(RunningStatus::Success), Status::Success);
    }

    #[test]
    fn running_status_failure_into_status() {
        assert_eq!(Status::from(RunningStatus::Failure), Status::Failure);
    }
}
