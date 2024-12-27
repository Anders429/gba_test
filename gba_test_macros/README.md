# gba_test_macros

[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/Anders429/gba_test/gba_test_macros.yml?branch=master)](https://github.com/Anders429/gba_test/actions/workflows/gba_test_macros.yml?query=branch%3Amaster)
[![crates.io](https://img.shields.io/crates/v/gba_test_macros)](https://crates.io/crates/gba_test_macros)
[![docs.rs](https://docs.rs/gba_test_macros/badge.svg)](https://docs.rs/gba_test_macros)
[![License](https://img.shields.io/crates/l/gba_test_macros)](#license)

Provides the `#[test]` attribute for annotating tests that should be run on the Game Boy Advance.

## Installation
This crate is meant to be used with the `gba_test` crate. In most cases, it is easiest to
simply use `gba_test` with the `macros` feature enabled by specifying the following in your
`Cargo.toml`:

```
[dependencies]
gba_test = {version = "0.3.0", features = ["macros"]}
```

`gba_test`'s `macros` feature is enabled by default.

## Usage
You can use the provided `#[test]` attribute to write tests in the same way you would normally
[write tests in Rust](https://doc.rust-lang.org/book/ch11-01-writing-tests.html):

``` rust
#![feature(custom_test_frameworks)]

#[cfg(test)]
mod tests {
    use gba_test_runner::test;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
```

Note that you should use the `#[test]` attribute provided by this crate, **not** the default
`#[test]` attribute.

Also note that use of this macro currently depends on the
[`custom_test_frameworks`](https://doc.rust-lang.org/beta/unstable-book/language-features/custom-test-frameworks.html)
unstable Rust feature. As such, you will need to enable it in any crate that writes tests using
this crate.

## License
This project is licensed under either of

* Apache License, Version 2.0
([LICENSE-APACHE](https://github.com/Anders429/gba_test/blob/HEAD/LICENSE-APACHE) or
http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
([LICENSE-MIT](https://github.com/Anders429/gba_test/blob/HEAD/LICENSE-MIT) or
http://opensource.org/licenses/MIT)

at your option.

### Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
