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
use syn::{parse, token, Attribute, ExprParen, Ident, ItemFn, Meta};

/// Structured representation of the configuration attributes provided for a test.
struct Attributes {
    ignore: Ident,
    ignore_message: Option<ExprParen>,
    should_panic: Ident,
}

impl Attributes {
    /// Returns the default configuration attributes for a test.
    fn new() -> Self {
        Self {
            ignore: Ident::new("No", Span::call_site()),
            ignore_message: None,
            should_panic: Ident::new("No", Span::call_site()),
        }
    }
}

impl From<&Vec<Attribute>> for Attributes {
    fn from(attributes: &Vec<Attribute>) -> Self {
        let mut result = Attributes::new();

        for attribute in attributes {
            if let Some(ident) = attribute.path().get_ident() {
                match ident.to_string().as_str() {
                    "ignore" => {
                        if let Meta::NameValue(name_value) = &attribute.meta {
                            result.ignore = Ident::new("YesWithMessage", Span::call_site());
                            result.ignore_message = Some(ExprParen {
                                attrs: Vec::new(),
                                paren_token: token::Paren::default(),
                                expr: Box::new(name_value.value.clone()),
                            });
                        } else {
                            result.ignore = Ident::new("Yes", Span::call_site());
                        }
                    }
                    "should_panic" => {
                        result.should_panic = Ident::new("Yes", Span::call_site());
                    }
                    _ => {
                        // Not supported.
                    }
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
///
/// The test macro supports the other testing attributes you would expect to use when writing unit
/// tests in Rust. Specifically, the `#[ignore]` and `#[should_panic]` attributes are supported.
///
/// # Example
/// ```
/// # #![feature(custom_test_frameworks)]
/// #
/// #[gba_test_macros::test]
/// #[ignore]
/// fn ignored() {
///     assert!(false);
/// }
///
/// #[gba_test_macros::test]
/// #[should_panic]
/// fn panics() {
///     panic!("expected panic");
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
    let ignore_message = attributes.ignore_message;
    let should_panic = attributes.should_panic;

    TokenStream::from(quote! {
        #function

        #[test_case]
        #[allow(non_upper_case_globals)]
        const #name: ::gba_test::Test = ::gba_test::Test {
            name: stringify!(#name),
            module: module_path!(),
            test: #name,
            ignore: ::gba_test::Ignore::#ignore #ignore_message,
            should_panic: ::gba_test::ShouldPanic::#should_panic,
        };
    })
}
