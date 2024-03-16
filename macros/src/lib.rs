//! Provides the `#[test]` attribute for annotating tests that should be run on the Game Boy
//! Advance.
//!
//! ## Usage
//! You can use the provided `#[test]` attribute to write tests in the same way you would normally
//! [write tests in Rust](https://doc.rust-lang.org/book/ch11-01-writing-tests.html):
//!
//! ``` rust
//! #![feature(custom_test_frameworks)]
//!
//! #[cfg(test)]
//! mod tests {
//!     use gba_test_macros::test;
//!
//!     #[test]
//!     fn it_works() {
//!         let result = 2 + 2;
//!         assert_eq!(result, 4);
//!     }
//! }
//! ```
//!
//! Note that you should use the `#[test]` attribute provided by this crate, **not** the default
//! `#[test]` attribute.
//!
//! Also note that use of this macro currently depends on the
//! [`custom_test_frameworks`](https://doc.rust-lang.org/beta/unstable-book/language-features/custom-test-frameworks.html)
//! unstable Rust feature. As such, you will need to enable it in any crate that writes tests using
//! this crate.

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse, Attribute, Ident, ItemFn};

/// Structured representation of the configuration attributes provided for a test.
struct Attributes {
    ignore: Ident,
}

impl Attributes {
    /// Returns the default configuration attributes for a test.
    fn new() -> Self {
        Self {
            ignore: Ident::new("No", Span::call_site()),
        }
    }
}

impl From<&Vec<Attribute>> for Attributes {
    fn from(attributes: &Vec<Attribute>) -> Self {
        let mut result = Attributes::new();

        for attribute in attributes {
            if let Some(ident) = attribute.path().get_ident() {
                if ident.to_string().as_str() == "ignore" {
                    result.ignore = Ident::new("Yes", Span::call_site());
                }
            }
        }

        result
    }
}

/// Defines a test to be executed on a Game Boy Advance.
///
/// # Example
/// ```
/// # #![feature(custom_test_frameworks)]
/// #
/// #[gba_test_macros::test]
/// fn foo() {
///     assert!(true);
/// }
/// ```
#[proc_macro_attribute]
pub fn test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let function: ItemFn = match parse(item) {
        Ok(function) => function,
        Err(error) => return error.into_compile_error().into(),
    };
    let name = function.sig.ident.clone();
    let attributes = Attributes::from(&function.attrs);
    let ignore = attributes.ignore;

    TokenStream::from(quote! {
        mod #name {
            use super::*;

            #function

            #[test_case]
            const TEST: ::gba_test::Test = ::gba_test::Test {
                name: module_path!(),
                test: #name,
                ignore: ::gba_test::Ignore::#ignore,
            };
        }
    })
}
