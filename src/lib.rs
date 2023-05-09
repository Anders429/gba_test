#![no_std]
#![cfg_attr(doc_cfg, feature(doc_cfg))]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "bincode")]
mod bincode_config;
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
#[cfg(feature = "serde")]
use serde::{
    de,
    de::{Deserialize, Deserializer, Expected, SeqAccess, Unexpected, Visitor},
    ser::{Serialize, SerializeStruct, Serializer},
};

#[cfg(feature = "bincode")]
pub use bincode_config::BINCODE_CONFIG;

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
}

impl TestCase for Test {
    fn name(&self) -> &str {
        self.name
    }

    fn run(&self) {
        (self.test)()
    }
}

/// The outcome of a test.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Outcome {
    /// The test passed.
    Passed,
    /// The test failed.
    Failed,
}

#[cfg(feature = "serde")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
impl Serialize for Outcome {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

#[cfg(feature = "serde")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
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
#[derive(Debug, Eq, PartialEq)]
pub struct Trial<'a> {
    /// The name of the test.
    pub name: &'a str,
    /// The test's outcome.
    pub outcome: Outcome,
}

#[cfg(feature = "serde")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
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

#[cfg(feature = "serde")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
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

pub enum StatusFromU8Error {
    InvalidValue(u8),
}

impl StatusFromU8Error {
    #[cfg(feature = "serde")]
    fn into_serde_error<E>(self, exp: &dyn Expected) -> E
    where
        E: de::Error,
    {
        match self {
            Self::InvalidValue(value) => E::invalid_value(Unexpected::Unsigned(value.into()), exp),
        }
    }
}

#[cfg(feature = "serde")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
impl Serialize for Status {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

#[cfg(feature = "serde")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
impl<'de> Deserialize<'de> for Status {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StatusVisitor;

        impl<'de> Visitor<'de> for StatusVisitor {
            type Value = Status;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a byte containing the value 0, 1, or 2")
            }

            fn visit_u8<E>(self, value: u8) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                value
                    .try_into()
                    .map_err(|err: StatusFromU8Error| err.into_serde_error(&self))
            }
        }

        deserializer.deserialize_u8(StatusVisitor)
    }
}

impl TryFrom<u8> for Status {
    type Error = StatusFromU8Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Status::Running),
            1 => Ok(Status::Success),
            2 => Ok(Status::Failure),
            _ => Err(StatusFromU8Error::InvalidValue(value)),
        }
    }
}

#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
#[derive(Debug, Eq, PartialEq)]
pub struct Conclusion<'a> {
    pub status: Status,
    pub trials: Vec<Trial<'a>>,
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

        deserializer.deserialize_struct("Conclusion", &["status", "trials"], ConclusionVisitor)
    }
}
