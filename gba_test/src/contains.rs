//! An implementation of the Rabin-Karp algorithm for determining if `PanicInfo` contains a
//! substring.
//!
//! This is a specialized implementation to operate specifically on `PanicInfo`. As such, there are
//! some inefficiencies, but in practice they shouldn't be noticeable. Specifically, there is no
//! way to index into the `PanicMessage`, and therefore all operations must be done using
//! `core::fmt::Write`, including removing old characters and equality comparison. The impact of
//! this is minimized by indexing into the strings given to the `write_str()` method in the most
//! efficient way possible, although *techincally* an input could be devised to make this run in
//! `O(n^2)`.

use core::{fmt::Write, panic::PanicInfo, str::Bytes};

pub(crate) fn contains(info: &PanicInfo, substring: &'static str) -> bool {
    if substring.is_empty() {
        return true;
    }

    if let Some(message_str) = info.message().as_str() {
        return message_str.contains(substring);
    }

    let mut searcher = RabinKarpSearch::new(substring, info);
    write!(searcher, "{}", info.message()).unwrap();

    searcher.found
}

const BASE: usize = 256;
const MODULUS: usize = 101;

fn hash(string: &str) -> usize {
    let mut result: usize = 0;
    for byte in string.bytes() {
        result = result.wrapping_mul(BASE);
        result %= MODULUS;
        result = result.wrapping_add(byte as usize);
        result %= MODULUS;
    }
    result
}

struct RabinKarpSearch<'a> {
    string: &'static str,
    hash: usize,
    length: usize,

    rolling_hash: usize,
    rolled: usize,
    index: usize,

    info: &'a PanicInfo<'a>,

    found: bool,
}

impl<'a> RabinKarpSearch<'a> {
    fn new(string: &'static str, info: &'a PanicInfo<'a>) -> Self {
        Self {
            string,
            hash: hash(string),
            length: string.len(),

            rolling_hash: 0,
            rolled: 0,
            index: 0,

            info,

            found: false,
        }
    }
}

impl Write for RabinKarpSearch<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        if self.found {
            return Ok(());
        }

        let mut bytes = s.bytes();

        // This is the initial rolling hash calculation. It calculates up to `length` bytes.
        while self.rolled < self.length {
            if let Some(byte) = bytes.next() {
                self.rolling_hash = self.rolling_hash.wrapping_mul(BASE);
                self.rolling_hash %= MODULUS;
                self.rolling_hash = self.rolling_hash.wrapping_add(byte as usize);
                self.rolling_hash %= MODULUS;
                self.rolled += 1;
            } else {
                // The rest is contained in a string that will be received later (or the substring
                // is bigger than the message).
                return Ok(());
            }
        }

        // For every new character, check if the current rolling hash is equal to the expected
        // hash. If it is, do an equality comparison. Otherwise, roll the next character.
        loop {
            if self.hash == self.rolling_hash {
                // Compare equality.
                let mut equality_comparison = EqualityComparison {
                    bytes: self.string.bytes(),
                    index: self.index,
                    result: None,
                };
                write!(equality_comparison, "{}", self.info.message()).unwrap();
                if let Some(true) = equality_comparison.result {
                    self.found = true;
                    return Ok(());
                }
            }
            if let Some(byte) = bytes.next() {
                // Remove the byte at the current index.
                let mut byte_remover = ByteRemover {
                    rolling_hash: &mut self.rolling_hash,
                    length: self.length,
                    index: self.index,
                    done: false,
                };
                write!(byte_remover, "{}", self.info.message()).unwrap();
                // Add the new byte.
                self.rolling_hash = self.rolling_hash.wrapping_mul(BASE);
                self.rolling_hash %= MODULUS;
                self.rolling_hash = self.rolling_hash.wrapping_add(byte as usize);
                self.rolling_hash %= MODULUS;
                self.index += 1;
            } else {
                // We will continue on another string we are given later.
                return Ok(());
            }
        }
    }
}

struct ByteRemover<'a> {
    rolling_hash: &'a mut usize,
    length: usize,
    index: usize,
    done: bool,
}

impl Write for ByteRemover<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        if self.done {
            return Ok(());
        }
        if self.index > s.len() {
            self.index -= s.len();
        } else {
            let mut removed_hash = s.as_bytes()[self.index] as usize;
            for _ in 1..self.length {
                removed_hash = removed_hash.wrapping_mul(BASE);
                removed_hash %= MODULUS;
            }
            *self.rolling_hash += MODULUS;
            *self.rolling_hash -= removed_hash;
            self.done = true;
        }
        Ok(())
    }
}

struct EqualityComparison {
    bytes: Bytes<'static>,
    index: usize,
    result: Option<bool>,
}

impl Write for EqualityComparison {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        if self.result.is_some() {
            return Ok(());
        }

        let mut bytes = s.bytes();
        // Get to the index we are checking.
        while self.index > 0 {
            if bytes.next().is_none() {
                return Ok(());
            }
            self.index -= 1;
        }
        loop {
            match (bytes.next(), self.bytes.next()) {
                (Some(left), Some(right)) => {
                    if left != right {
                        self.result = Some(false);
                        return Ok(());
                    }
                }
                (_, None) => {
                    // We've hit the end and found no inequalities.
                    self.result = Some(true);
                    return Ok(());
                }
                (None, _) => {
                    // We will continue with another string later.
                    return Ok(());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ByteRemover, EqualityComparison, hash};
    use claims::{assert_ok, assert_some};
    use core::fmt::Write;
    use gba_test::test;

    #[test]
    fn hash_hi() {
        // This is an example from the wikipedia page.
        assert_eq!(hash("hi"), 65);
    }

    #[test]
    fn hash_abr() {
        // This is an example from the wikipedia page.
        assert_eq!(hash("abr"), 4);
    }

    #[test]
    fn hash_bra() {
        // This is an example from the wikipedia page.
        assert_eq!(hash("bra"), 30);
    }

    #[test]
    fn byte_remover_abra() {
        // This is an example from the wikipedia page.
        let mut rolling_hash = 4; // Hash for `abr`.
        let mut byte_remover = ByteRemover {
            rolling_hash: &mut rolling_hash,
            length: 3,
            index: 0,
            done: false,
        };
        assert_ok!(write!(byte_remover, "abra"));
        assert!(byte_remover.done);
        assert_eq!(rolling_hash, 53); // 4 + 101 - ((97 * 256^2) % 101).
    }

    #[test]
    fn equality_comparison_simple_equal() {
        let mut equality_comparison = EqualityComparison {
            bytes: "abc".bytes().into_iter(),
            index: 0,
            result: None,
        };
        assert_ok!(write!(equality_comparison, "abc"));
        assert!(assert_some!(equality_comparison.result));
    }

    #[test]
    fn equality_comparison_simple_not_equal() {
        let mut equality_comparison = EqualityComparison {
            bytes: "abc".bytes().into_iter(),
            index: 0,
            result: None,
        };
        assert_ok!(write!(equality_comparison, "cba"));
        assert!(!assert_some!(equality_comparison.result));
    }

    #[test]
    fn equality_comparison_complex_equal() {
        let mut equality_comparison = EqualityComparison {
            bytes: "abc".bytes().into_iter(),
            index: 2,
            result: None,
        };
        assert_ok!(write!(equality_comparison, "ababc"));
        assert!(assert_some!(equality_comparison.result));
    }

    #[test]
    fn equality_comparison_complex_not_equal() {
        let mut equality_comparison = EqualityComparison {
            bytes: "abc".bytes().into_iter(),
            index: 2,
            result: None,
        };
        assert_ok!(write!(equality_comparison, "abababc"));
        assert!(!assert_some!(equality_comparison.result));
    }
}
