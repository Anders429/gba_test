use core::fmt::Display;
use serde::{Serialize, Serializer};

/// Wrapper for serializing a type that implements [`Display`] using [`Serializer::collect_str()`].
///
/// This type can later be deserialized as a [`&'de str`].
///
/// [`&'de str`]: str
#[cfg(feature = "serde")]
#[derive(Debug)]
pub(crate) struct SerializeDisplay<T>(pub(crate) T);

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
