//! Types representing test results.

use core::{fmt, fmt::Display, str};
#[cfg(feature = "serde")]
use serde::{
    de,
    de::{
        Deserialize, Deserializer, EnumAccess, Error as _, MapAccess, SeqAccess, Unexpected,
        VariantAccess, Visitor,
    },
    ser::{Serialize, SerializeStruct, SerializeStructVariant, Serializer},
};

/// Wrapper for serializing a type that implements [`Display`] using [`Serializer::collect_str()`].
///
/// This type can later be deserialized as a [`&'de str`].
///
/// [`&'de str`]: str
#[cfg(feature = "serde")]
#[derive(Debug)]
struct SerializeDisplay<T>(T);

#[cfg(feature = "serde")]
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

#[cfg(test)]
mod tests {
    use super::SerializeDisplay;
    use alloc::{borrow::ToOwned, vec};
    use claims::assert_ok_eq;
    use serde::Serialize;
    use serde_assert::{Serializer, Token, Tokens};

    #[test]
    fn serialize_display() {
        let serializer = Serializer::builder().build();
        assert_ok_eq!(
            SerializeDisplay(format_args!("{} foo {}", 1, 2)).serialize(&serializer),
            Tokens(vec![Token::Str("1 foo 2".to_owned())])
        );
    }
}
