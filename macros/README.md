# gba_test_macros
Provides the `#[test]` attribute for annotating tests that should be run on the Game Boy Advance.

## Installation
This crate is meant to be used with the `gba_test` crate. In most cases, it is easiest to
simply use `gba_test` with the `macros` feature enabled by specifying the following in your
`Cargo.toml`:

```
[dependencies]
gba_test = {version = "0.1.0", features = ["macros"]}
```

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
