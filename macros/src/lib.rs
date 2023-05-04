use proc_macro::TokenStream;
use quote::quote;
use syn::{parse, ItemFn};

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
    let function: ItemFn = parse(item).unwrap();
    let name = function.sig.ident.clone();

    TokenStream::from(quote! {
        mod #name {
            use super::*;

            #function

            #[test_case]
            const TEST: ::gba_test_runner::Test = ::gba_test_runner::Test {
                name: stringify!(#name),
                test: #name,
            };
        }
    })
}
