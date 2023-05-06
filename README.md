# gba_test_runner
An emulator-agnostic test runner for the Game Boy Advance.

Currently a work-in-progress.

## Goals
- Allow users to easily define their own tests.
- Runnable on actual hardware, rather than only working on specific emulators.
- Compatibility with custom test runners for specific environments.
- Display test results on screen.
- Feature parity with Rust's built-in testing.

## Strategy
Tests will be run single-threaded, recovering after panics. The test results will be serialized directly to external memory. This will allow writing a max of 64 KiB of data, so a serialization format that is very concise will need to be used.

`bincode` is probably the most straight-forward solution. The upcoming version 2.0 supports `no_std` environments, and still interoperates with `serde`. However, it's questionable whether they will continue supporting `serde` as a first-class citizen, as they've introduced their own traits to seemingly replace `serde`'s. This project should stick to `serde` if possible, as that allows easy switching of the serialization in the future, and `serde` is easy to customize. At this point, it is not clear to me how to customize `bincode` in the same ways.

Another alternative is `postcard`. When making this decision, it really comes down to what I can use to serialize the smallest amount of data for a test result.

After each test is run, the result can be written directly to SRAM. If displaying is enabled, the result will also be written to the screen.

After all tests are run, display mode can display the number of tests that passed and failed, and allow a user to scroll through them.

The format of data written to SRAM should also include the number of tests run at the beginning. This can be known ahead of time by simply examining the `tests` array provided to the test runner.

## Development
To run the integration tests, you need [`mgba-rom-test`](https://github.com/mgba-emu/mgba/blob/master/src/platform/test/rom-test-main.c). Install it by running the following within a copy of the `mgba` source:

```
$ cmake .. -DUSE_LIBZIP=OFF -DBUILD_ROM_TEST=ON -DUSE_PNG=OFF
$ make
```

Make sure to add `mgba-rom-test` to your `PATH`. The cargo configuration file for the tests should use it automatically.
