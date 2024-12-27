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
use syn::{
    parse, parse2, parse_str, token, Attribute, Error, ExprParen, Ident, ItemFn, Meta, ReturnType,
    Type,
};

/// Structured representation of the configuration attributes provided for a test.
struct Attributes {
    ignore: Ident,
    ignore_message: Option<ExprParen>,
    should_panic: Ident,
    should_panic_message: Option<ExprParen>,
}

impl Attributes {
    /// Returns the default configuration attributes for a test.
    fn new() -> Self {
        Self {
            ignore: Ident::new("No", Span::call_site()),
            ignore_message: None,
            should_panic: Ident::new("No", Span::call_site()),
            should_panic_message: None,
        }
    }
}

impl TryFrom<&Vec<Attribute>> for Attributes {
    type Error = Error;

    fn try_from(attributes: &Vec<Attribute>) -> Result<Self, Self::Error> {
        let mut result = Attributes::new();

        for attribute in attributes {
            if let Some(ident) = attribute.path().get_ident() {
                match ident.to_string().as_str() {
                    "ignore" => {
                        match &attribute.meta {
                            Meta::NameValue(name_value) => {
                                result.ignore = Ident::new("YesWithMessage", Span::call_site());
                                result.ignore_message = Some(ExprParen {
                                    attrs: Vec::new(),
                                    paren_token: token::Paren::default(),
                                    expr: Box::new(name_value.value.clone()),
                                });
                            }
                            Meta::List(_) => return Err(Error::new_spanned(attribute, "valid forms for the attribute are `#[ignore]` and `#[ignore = \"reason\"]`")),
                            Meta::Path(_) => result.ignore = Ident::new("Yes", Span::call_site()),
                        }
                    }
                    "should_panic" => {
                        match &attribute.meta {
                            Meta::List(meta_list) => {
                                if let Ok(Meta::NameValue(name_value)) =
                                    parse2(meta_list.tokens.clone())
                                {
                                    if name_value.path == parse_str("expected").unwrap() {
                                        result.should_panic =
                                            Ident::new("YesWithMessage", Span::call_site());
                                        result.should_panic_message = Some(ExprParen {
                                            attrs: Vec::new(),
                                            paren_token: token::Paren::default(),
                                            expr: Box::new(name_value.value),
                                        });
                                    } else {
                                        return Err(Error::new_spanned(attribute, "argument must be of the form: `expected = \"error message\"`"));
                                    }
                                } else {
                                    return Err(Error::new_spanned(attribute, "argument must be of the form: `expected = \"error message\"`"));
                                }
                            }
                            Meta::NameValue(name_value) => {
                                result.should_panic =
                                    Ident::new("YesWithMessage", Span::call_site());
                                result.should_panic_message = Some(ExprParen {
                                    attrs: Vec::new(),
                                    paren_token: token::Paren::default(),
                                    expr: Box::new(name_value.value.clone()),
                                });
                            }
                            Meta::Path(_) => {
                                result.should_panic = Ident::new("Yes", Span::call_site());
                            }
                        }
                    }
                    _ => {
                        // Not supported.
                    }
                }
            }
        }

        Ok(result)
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
    let return_type = match &function.sig.output {
        ReturnType::Default => parse_str::<Type>("()").unwrap(),
        ReturnType::Type(_, return_type) => *return_type.clone(),
    };
    let attributes = match Attributes::try_from(&function.attrs) {
        Ok(attributes) => attributes,
        Err(error) => return error.into_compile_error().into(),
    };
    let ignore = attributes.ignore;
    let ignore_message = attributes.ignore_message;
    let should_panic = attributes.should_panic;
    let should_panic_message = attributes.should_panic_message;
    if return_type != parse_str::<Type>("()").unwrap()
        && should_panic != Ident::new("No", Span::call_site())
    {
        return Error::new_spanned(
            function,
            "functions using `#[should_panic]` must return `()`",
        )
        .into_compile_error()
        .into();
    }

    TokenStream::from(quote! {
        #[allow(dead_code)]
        #function

        #[test_case]
        #[allow(non_upper_case_globals)]
        const #name: ::gba_test::Test::<#return_type> = ::gba_test::Test::<#return_type> {
            name: stringify!(#name),
            module: module_path!(),
            test: #name,
            ignore: ::gba_test::Ignore::#ignore #ignore_message,
            should_panic: ::gba_test::ShouldPanic::#should_panic #should_panic_message,
        };
    })
}
