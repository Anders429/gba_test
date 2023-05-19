//! Types representing test results.

#[cfg(feature = "serde")]
use crate::display::SerializeDisplay;
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
        enum Field {
            Name,
            Outcome,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`name` or `outcome`")
                    }

                    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        match v {
                            "name" => Ok(Field::Name),
                            "outcome" => Ok(Field::Outcome),
                            _ => Err(E::unknown_field(v, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

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

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut name = None;
                let mut outcome = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Name => {
                            if name.is_some() {
                                return Err(A::Error::duplicate_field("name"));
                            }
                            name = Some(map.next_value()?);
                        }
                        Field::Outcome => {
                            if outcome.is_some() {
                                return Err(A::Error::duplicate_field("outcome"));
                            }
                            outcome = Some(map.next_value()?);
                        }
                    }
                }

                Ok(Trial {
                    name: name.ok_or_else(|| A::Error::missing_field("name"))?,
                    outcome: outcome.ok_or_else(|| A::Error::missing_field("outcome"))?,
                })
            }
        }

        const FIELDS: &[&str] = &["name", "outcome"];

        deserializer.deserialize_struct("Trial", FIELDS, TrialVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::{Outcome, Trial};
    use alloc::{borrow::ToOwned, vec};
    use claims::{assert_err_eq, assert_ok_eq};
    use serde::{de::Error as _, Deserialize, Serialize};
    use serde_assert::{de, Deserializer, Serializer, Token, Tokens};

    #[test]
    fn serialize_deserialize_outcome_passed() {
        let serializer = Serializer::builder().build();
        let tokens = assert_ok_eq!(
            Outcome::<&str>::Passed.serialize(&serializer),
            Tokens(vec![Token::UnitVariant {
                name: "Outcome",
                variant_index: 0,
                variant: "Passed"
            }])
        );

        let mut deserializer = Deserializer::builder().tokens(tokens).build();
        assert_ok_eq!(
            Outcome::<&str>::deserialize(&mut deserializer),
            Outcome::Passed
        );
    }

    #[test]
    fn serialize_deserialize_outcome_failed() {
        let serializer = Serializer::builder().build();
        let tokens = assert_ok_eq!(
            Outcome::Failed { message: "foo" }.serialize(&serializer),
            Tokens(vec![
                Token::StructVariant {
                    name: "Outcome",
                    variant_index: 1,
                    variant: "Failed",
                    len: 1
                },
                Token::Field("message"),
                Token::Str("foo".to_owned()),
                Token::StructVariantEnd
            ])
        );

        let mut deserializer = Deserializer::builder().tokens(tokens).build();
        assert_ok_eq!(
            Outcome::deserialize(&mut deserializer),
            Outcome::Failed { message: "foo" }
        );
    }

    #[test]
    fn serialize_deserialize_outcome_failed_display() {
        let serializer = Serializer::builder().build();
        let tokens = assert_ok_eq!(
            Outcome::Failed {
                message: format_args!("{} foo {}", 1, 2)
            }
            .serialize(&serializer),
            Tokens(vec![
                Token::StructVariant {
                    name: "Outcome",
                    variant_index: 1,
                    variant: "Failed",
                    len: 1
                },
                Token::Field("message"),
                Token::Str("1 foo 2".to_owned()),
                Token::StructVariantEnd
            ])
        );

        let mut deserializer = Deserializer::builder().tokens(tokens).build();
        assert_ok_eq!(
            Outcome::deserialize(&mut deserializer),
            Outcome::Failed { message: "1 foo 2" }
        );
    }

    #[test]
    fn serialize_deserialize_outcome_ignored() {
        let serializer = Serializer::builder().build();
        let tokens = assert_ok_eq!(
            Outcome::<&str>::Ignored.serialize(&serializer),
            Tokens(vec![Token::UnitVariant {
                name: "Outcome",
                variant_index: 2,
                variant: "Ignored"
            }])
        );

        let mut deserializer = Deserializer::builder().tokens(tokens).build();
        assert_ok_eq!(
            Outcome::<&str>::deserialize(&mut deserializer),
            Outcome::Ignored
        );
    }

    #[test]
    fn deserialize_outcome_unknown_variant() {
        let mut deserializer = Deserializer::builder()
            .tokens(Tokens(vec![Token::UnitVariant {
                name: "Outcome",
                variant_index: 3,
                variant: "Unknown",
            }]))
            .build();

        assert_err_eq!(
            Outcome::<&str>::deserialize(&mut deserializer),
            de::Error::unknown_variant("Unknown", &["Passed", "Failed", "Ignored"])
        );
    }

    #[test]
    fn deserialize_outcome_failed_missing_message() {
        let mut deserializer = Deserializer::builder()
            .tokens(Tokens(vec![
                Token::StructVariant {
                    name: "Outcome",
                    variant_index: 1,
                    variant: "Failed",
                    len: 0,
                },
                Token::StructVariantEnd,
            ]))
            .build();

        assert_err_eq!(
            Outcome::<&str>::deserialize(&mut deserializer),
            de::Error::missing_field("message")
        );
    }

    #[test]
    fn deserialize_outcome_failed_duplicate_message() {
        let mut deserializer = Deserializer::builder()
            .tokens(Tokens(vec![
                Token::StructVariant {
                    name: "Outcome",
                    variant_index: 1,
                    variant: "Failed",
                    len: 2,
                },
                Token::Field("message"),
                Token::Str("foo".to_owned()),
                Token::Field("message"),
                Token::Str("bar".to_owned()),
                Token::StructVariantEnd,
            ]))
            .build();

        assert_err_eq!(
            Outcome::<&str>::deserialize(&mut deserializer),
            de::Error::duplicate_field("message")
        );
    }

    #[test]
    fn serialize_deserialize_trial() {
        let serializer = Serializer::builder().build();
        let tokens = assert_ok_eq!(
            Trial {
                name: "foo",
                outcome: Outcome::<&str>::Passed,
            }
            .serialize(&serializer),
            Tokens(vec![
                Token::Struct {
                    name: "Trial",
                    len: 2
                },
                Token::Field("name"),
                Token::Str("foo".to_owned()),
                Token::Field("outcome"),
                Token::UnitVariant {
                    name: "Outcome",
                    variant_index: 0,
                    variant: "Passed"
                },
                Token::StructEnd
            ])
        );

        let mut deserializer = Deserializer::builder().tokens(tokens).build();
        assert_ok_eq!(
            Trial::deserialize(&mut deserializer),
            Trial {
                name: "foo",
                outcome: Outcome::<&str>::Passed,
            }
        );
    }

    #[test]
    fn deserialize_trial_different_order() {
        let mut deserializer = Deserializer::builder()
            .tokens(Tokens(vec![
                Token::Struct {
                    name: "Trial",
                    len: 2,
                },
                Token::Field("outcome"),
                Token::UnitVariant {
                    name: "Outcome",
                    variant_index: 0,
                    variant: "Passed",
                },
                Token::Field("name"),
                Token::Str("foo".to_owned()),
                Token::StructEnd,
            ]))
            .build();
        assert_ok_eq!(
            Trial::deserialize(&mut deserializer),
            Trial {
                name: "foo",
                outcome: Outcome::<&str>::Passed,
            }
        );
    }

    #[test]
    fn deserialize_trial_unknown_field() {
        let mut deserializer = Deserializer::builder()
            .tokens(Tokens(vec![
                Token::Struct {
                    name: "Trial",
                    len: 1,
                },
                Token::Field("unknown"),
                Token::StructEnd,
            ]))
            .build();
        assert_err_eq!(
            Trial::deserialize(&mut deserializer),
            de::Error::unknown_field("unknown", &["name", "outcome"])
        );
    }

    #[test]
    fn deserialize_trial_missing_field_name() {
        let mut deserializer = Deserializer::builder()
            .tokens(Tokens(vec![
                Token::Struct {
                    name: "Trial",
                    len: 1,
                },
                Token::Field("outcome"),
                Token::UnitVariant {
                    name: "Outcome",
                    variant_index: 0,
                    variant: "Passed",
                },
                Token::StructEnd,
            ]))
            .build();
        assert_err_eq!(
            Trial::deserialize(&mut deserializer),
            de::Error::missing_field("name")
        );
    }

    #[test]
    fn deserialize_trial_missing_field_outcome() {
        let mut deserializer = Deserializer::builder()
            .tokens(Tokens(vec![
                Token::Struct {
                    name: "Trial",
                    len: 1,
                },
                Token::Field("name"),
                Token::Str("foo".to_owned()),
                Token::StructEnd,
            ]))
            .build();
        assert_err_eq!(
            Trial::deserialize(&mut deserializer),
            de::Error::missing_field("outcome")
        );
    }

    #[test]
    fn deserialize_trial_duplicate_field_name() {
        let mut deserializer = Deserializer::builder()
            .tokens(Tokens(vec![
                Token::Struct {
                    name: "Trial",
                    len: 3,
                },
                Token::Field("name"),
                Token::Str("foo".to_owned()),
                Token::Field("outcome"),
                Token::UnitVariant {
                    name: "Outcome",
                    variant_index: 0,
                    variant: "Passed",
                },
                Token::Field("name"),
                Token::Str("bar".to_owned()),
                Token::StructEnd,
            ]))
            .build();
        assert_err_eq!(
            Trial::deserialize(&mut deserializer),
            de::Error::duplicate_field("name")
        );
    }

    #[test]
    fn deserialize_trial_duplicate_field_outcome() {
        let mut deserializer = Deserializer::builder()
            .tokens(Tokens(vec![
                Token::Struct {
                    name: "Trial",
                    len: 3,
                },
                Token::Field("outcome"),
                Token::UnitVariant {
                    name: "Outcome",
                    variant_index: 0,
                    variant: "Passed",
                },
                Token::Field("name"),
                Token::Str("foo".to_owned()),
                Token::Field("outcome"),
                Token::UnitVariant {
                    name: "Outcome",
                    variant_index: 2,
                    variant: "Ignored",
                },
                Token::StructEnd,
            ]))
            .build();
        assert_err_eq!(
            Trial::deserialize(&mut deserializer),
            de::Error::duplicate_field("outcome")
        );
    }
}
