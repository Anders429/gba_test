#![no_std]
#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![cfg_attr(
    all(feature = "runner", target = "thumbv4t-none-eabi"),
    feature(panic_info_message)
)]

#[cfg(any(feature = "alloc", test))]
extern crate alloc;

#[cfg(all(feature = "runner", any(target = "thumbv4t-none-eabi", doc)))]
mod runner;
mod test_case;
mod trial;

#[cfg(all(feature = "runner", any(target = "thumbv4t-none-eabi", doc)))]
pub use runner::runner;
pub use test_case::{Ignore, Test, TestCase};
pub use trial::{Outcome, Trial};

#[cfg(feature = "gba_test_macros")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "macros")))]
pub use gba_test_macros::test;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;
#[cfg(feature = "serde")]
use core::{fmt, str};
use serde::de::VariantAccess;
#[cfg(all(feature = "serde", feature = "alloc"))]
use serde::ser::SerializeTupleStruct;
#[cfg(feature = "serde")]
use serde::{
    de,
    de::{Deserialize, Deserializer, EnumAccess, Unexpected, Visitor},
    ser::{Serialize, Serializer},
};

/// Separate enum for deserializing the variant of a Status.
///
/// This is separate from `RawStatus` because of the deserialization context in which it is used.
/// `StatusVariant` is used to deserialize the variant, which `RawStatus` and `Status` are used to
/// deserialize the full `enum` based on which variant is deserialized here.
enum StatusVariant {
    Running,
    Completed,
}

const STATUS_VARIANTS: &[&str] = &["Running", "Completed"];

impl<'de> Deserialize<'de> for StatusVariant {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StatusVariantVisitor;

        impl<'de> Visitor<'de> for StatusVariantVisitor {
            type Value = StatusVariant;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("`Running` or `Completed`")
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match value {
                    0 => Ok(StatusVariant::Running),
                    1 => Ok(StatusVariant::Completed),
                    _ => Err(E::invalid_value(Unexpected::Unsigned(value.into()), &self)),
                }
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match value {
                    "Running" => Ok(StatusVariant::Running),
                    "Completed" => Ok(StatusVariant::Completed),
                    _ => Err(E::unknown_variant(value, STATUS_VARIANTS)),
                }
            }

            fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match value {
                    b"Running" => Ok(StatusVariant::Running),
                    b"Completed" => Ok(StatusVariant::Completed),
                    _ => {
                        if let Ok(value) = str::from_utf8(value) {
                            Err(E::unknown_variant(value, STATUS_VARIANTS))
                        } else {
                            Err(E::invalid_value(Unexpected::Bytes(value), &self))
                        }
                    }
                }
            }
        }

        deserializer.deserialize_identifier(StatusVariantVisitor)
    }
}

/// Raw status of test execution.
///
/// This is essentially just the Status variant. This allows serialization of only the variant,
/// which allows it to be used in contexts where allocation is not available.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RawStatus {
    /// Tests are currently running.
    Running,
    /// The test runner successfully executed all tests.
    Completed,
}

#[cfg(feature = "serde")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
impl Serialize for RawStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            RawStatus::Running => serializer.serialize_unit_variant("RawStatus", 0, "Running"),
            RawStatus::Completed => serializer.serialize_unit_variant("RawStatus", 1, "Completed"),
        }
    }
}

#[cfg(feature = "serde")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
impl<'de> Deserialize<'de> for RawStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RawStatusVisitor;

        impl<'de> Visitor<'de> for RawStatusVisitor {
            type Value = RawStatus;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("enum RawStatus")
            }

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                match data.variant()? {
                    (StatusVariant::Running, variant) => {
                        variant.unit_variant().and(Ok(RawStatus::Running))
                    }
                    (StatusVariant::Completed, variant) => {
                        variant.unit_variant().and(Ok(RawStatus::Completed))
                    }
                }
            }
        }

        deserializer.deserialize_enum("RawStatus", STATUS_VARIANTS, RawStatusVisitor)
    }
}

/// Test execution status.
///
/// This enum encodes the current execution status, including test results upon completion.
///
/// The data stored by the [`runner()`] can be directly deserialized into this struct using [`postcard`].
#[cfg(feature = "alloc")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "alloc")))]
#[derive(Debug, Eq, PartialEq)]
pub enum Status<'a> {
    Running,
    Completed(Vec<Trial<'a, &'a str>>),
}

#[cfg(feature = "alloc")]
impl Serialize for Status<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Running => serializer.serialize_unit_variant("Status", 0, "Running"),
            Self::Completed(trials) => {
                let mut tuple_variant = serializer.serialize_tuple_struct("Status", 1)?;
                tuple_variant.serialize_field(trials)?;
                tuple_variant.end()
            }
        }
    }
}

#[cfg(feature = "alloc")]
impl<'de> Deserialize<'de> for Status<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StatusVisitor;

        impl<'de> Visitor<'de> for StatusVisitor {
            type Value = Status<'de>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("enum Status")
            }

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                match data.variant()? {
                    (StatusVariant::Running, variant) => {
                        variant.unit_variant().and(Ok(Status::Running))
                    }
                    (StatusVariant::Completed, variant) => {
                        variant.newtype_variant().map(Status::Completed)
                    }
                }
            }
        }

        deserializer.deserialize_enum("Status", STATUS_VARIANTS, StatusVisitor)
    }
}
