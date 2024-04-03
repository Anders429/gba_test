/// The outcome of a test.
#[derive(Debug)]
pub(crate) enum Outcome<Data> {
    /// The test passed.
    Passed,
    /// The test failed.
    Failed(Data),
    /// The test was excluded from the test run.
    Ignored,
}

impl<Data> Outcome<Data> {
    pub(crate) fn as_str(&self) -> &str {
        match self {
            Self::Passed => "ok",
            Self::Failed(_) => "FAILED",
            Self::Ignored => "ignored",
        }
    }
}
