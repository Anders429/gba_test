use crate::{alignment::Align4, test_case::TestCase};
use core::{
    fmt,
    fmt::{Display, Write},
    marker::PhantomData,
    ptr, slice, str,
};

const EWRAM_MAX: usize = 0x0204_0000;

/// The outcome of a test.
#[derive(Debug)]
pub(crate) enum Outcome<Data> {
    /// The test passed.
    Passed,
    /// The test failed.
    Failed(Data),
    /// The test was excluded from the test run.
    Ignored,
    /// The test was excluded from the test run with a message to be displayed.
    IgnoredWithMessage(&'static str),
}

impl<Data> Outcome<Data> {
    pub(crate) fn as_str(&self) -> &str {
        match self {
            Self::Passed => "ok",
            Self::Failed(_) => "FAILED",
            Self::Ignored | Self::IgnoredWithMessage(_) => "ignored",
        }
    }
}

/// The outcome of a test, not including any associated data.
///
/// This is just for internal storage. It should not be used outside of this module.
#[derive(Clone, Debug)]
#[repr(u8)]
enum OutcomeVariant {
    /// The test passed.
    Passed,
    /// The test failed.
    Failed,
    /// The test was excluded from the test run.
    Ignored,
    /// The test was excluded from the test run with a message to be displayed.
    IgnoredWithMessage,
}

impl<'a, Data> From<&'a Outcome<Data>> for OutcomeVariant {
    fn from(outcome: &'a Outcome<Data>) -> Self {
        match outcome {
            Outcome::Passed => Self::Passed,
            Outcome::Failed(_) => Self::Failed,
            Outcome::Ignored => Self::Ignored,
            Outcome::IgnoredWithMessage(_) => Self::IgnoredWithMessage,
        }
    }
}

/// Writer for an error message.
///
/// This writes an error message in the form of: <length><data><length>. The length is stored as a
/// header and a footer, enabling both forward and backward traversal of the unsized heap of data.
struct ErrorMessage<'a> {
    /// The beginning of this error message.
    ///
    /// This is the location of the message's length, which is not written until the message is
    /// dropped. Using a mutable reference here ensures we can update the pointer for the `Tests`
    /// object as well when the writing is complete.
    ///
    /// Note that care must be taken to ensure this pointer is aligned.
    start: &'a mut *mut usize,
    /// The current cursor, pointing to where message bytes are being written.
    cursor: *mut u8,
}

impl<'a> ErrorMessage<'a> {
    /// Creates a new error message writer, starting at the given pointer.
    ///
    /// The pointer is passed via mutable reference to allow it to be updated automatically when
    /// this `ErrorMessage` is dropped.
    fn new(start: &'a mut *mut usize) -> Self {
        Self {
            cursor: unsafe { start.cast::<u8>().add(4) },
            start,
        }
    }
}

impl Write for ErrorMessage<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let b = s.as_bytes();

        // Ensure there is enough space to write.
        if unsafe { self.cursor.add(b.len()).add(4) } as usize > EWRAM_MAX {
            return Err(fmt::Error);
        }

        // Write data.
        unsafe {
            ptr::copy(b.as_ptr(), self.cursor, b.len());
            self.cursor = self.cursor.add(b.len());
        }

        Ok(())
    }
}

impl Drop for ErrorMessage<'_> {
    fn drop(&mut self) {
        // Write byte length.
        let length = self.cursor as usize - unsafe { (*self.start).add(1) } as usize;
        // Header.
        unsafe {
            self.start.write(length);
        }
        // Footer.
        self.cursor = self.cursor.align_forward();
        unsafe {
            self.cursor.cast::<usize>().write(length);
        }

        // Move the pointer reference to the next available space.
        *self.start = unsafe { self.cursor.cast::<usize>().add(1) };
    }
}

/// Stores the current testing context.
///
/// This stores a reference to the tests being run, the current test being run, and the location of
/// the test outcome data.
pub(crate) struct Tests {
    /// The index of the current test.
    index: usize,
    /// The actual tests that are being run.
    tests: &'static [&'static dyn TestCase],
    /// Whether a test is currently waiting to be completed.
    waiting_for_completion: bool,
    /// Pointer to an array of outcome variants.
    ///
    /// This array has the same length as `tests`. The length is not stored here to remove the need
    /// to store the length twice in the same struct.
    ///
    /// Only variants are stored here, as not all outcomes have associated data, and all of their
    /// associated data is stored on the `data` heap in an unsized fashion.
    outcomes: *mut OutcomeVariant,
    /// Data heap for associated outcome data.
    ///
    /// This includes data such as error messages. It is stored on a heap to allow for error
    /// messages of any size, as well as to only store data for variants that need it, saving
    /// memory.
    data: *mut usize,
}

impl Tests {
    /// Creates a new `Tests`, wrapping the given test and storing unsized data in `data`.
    ///
    /// # Safety
    /// `data` must be a valid pointer to an unused space in EWRAM. In other words, it must be
    /// between 0x0200_0000 and 0x0203_ffff. All memory from `data` to the end of EWRAM must be
    /// unused and considered owned by this struct (meaning you should only have one instance of
    /// this struct).
    ///
    /// # Panics
    /// If there is not enough memory available in `data` to store the outcome variants.
    pub(crate) unsafe fn new(tests: &'static [&'static dyn TestCase], data: *mut u8) -> Self {
        let unsized_data = data.byte_add(tests.len()).align_forward() as *mut usize;
        if unsized_data as usize > EWRAM_MAX {
            panic!("not enough memory available to store outcome variants; `data` starts at {:?}, and {} bytes are required to be stored for the variants", data, tests.len());
        }

        Self {
            index: 0,
            tests,
            waiting_for_completion: false,
            outcomes: data as *mut OutcomeVariant,
            data: unsized_data,
        }
    }

    /// Registers the next test to be run (if one exists) as the current test and returns a static
    /// reference to that test.
    ///
    /// If this returns `None`, then there are no more tests to be run.
    ///
    /// # Panics
    /// If a previous test was started and no call to `complete_test()` was made.
    pub(crate) fn start_test(&mut self) -> Option<&'static dyn TestCase> {
        if self.waiting_for_completion {
            panic!("previous test at index {} was not completed", self.index);
        }

        if let Some(&test) = self.tests.get(self.index) {
            self.waiting_for_completion = true;
            Some(test)
        } else {
            // There are no more tests left to execute.
            None
        }
    }

    /// Returns the test that is currently being executed.
    ///
    /// If this returns `None`, then no test is currently executing. This is relevant in contexts
    /// like panicking where we need to know whether we panicked while executing a test.
    pub(crate) fn current_test(&self) -> Option<&'static dyn TestCase> {
        if !self.waiting_for_completion {
            return None;
        }

        // SAFETY: We know that this is a valid index, because `self.waiting_for_completion` is
        // only enabled when `self.index` corresponds to a valid test in `self.tests`.
        Some(*unsafe { self.tests.get_unchecked(self.index) })
    }

    /// Marks the current test as complete, storing the given outcome as the outcome for the test.
    ///
    /// This must be called before a new test is started with `start_test()`.
    ///
    /// # Panics
    /// If a test is not currently executing.
    pub(crate) fn complete_test<Data>(&mut self, outcome: Outcome<Data>)
    where
        Data: Display,
    {
        if !self.waiting_for_completion {
            panic!("a test was attempted to be marked as complete, but no test is executing");
        }

        self.waiting_for_completion = false;

        // SAFETY: `self.outcomes` is guaranteed to be valid for the length of `self.tests`. Since
        // we are only processing this for each test one time, this means that these writes are
        // guaranteed to be valid.
        unsafe {
            self.outcomes.write((&outcome).into());
            self.outcomes = self.outcomes.add(1);
        }
        match outcome {
            Outcome::Failed(data) => {
                log::info!("data: {}", data);
                let mut error_message = ErrorMessage::new(&mut self.data);
                write!(error_message, "{}", data)
                    .expect("not enough space to store error message: {data}");
            }
            _ => {}
        }

        self.index += 1;
    }

    /// Returns the completed outcomes.
    ///
    /// # Panics
    /// If there are still tests that have not been executed.
    pub(crate) fn outcomes(&self) -> TestOutcomes {
        if self.index < self.tests.len() {
            panic!("not all tests have been executed");
        }

        TestOutcomes {
            tests: self.tests,
            outcomes: unsafe { self.outcomes.sub(self.tests.len()) },
            data: self.outcomes.align_forward().cast(),
        }
    }
}

pub(crate) struct TestOutcomes {
    tests: &'static [&'static dyn TestCase],
    outcomes: *mut OutcomeVariant,
    data: *mut usize,
}

impl TestOutcomes {
    pub(crate) fn iter(&self) -> TestOutcomesIter {
        TestOutcomesIter {
            tests: self.tests.iter(),
            outcomes: self.outcomes,
            data: self.data,
        }
    }
}

pub(crate) struct TestOutcomesIter {
    tests: slice::Iter<'static, &'static dyn TestCase>,
    outcomes: *mut OutcomeVariant,
    data: *mut usize,
}

impl Iterator for TestOutcomesIter {
    type Item = (&'static dyn TestCase, Outcome<&'static str>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(&test) = self.tests.next() {
            let outcome_variant = unsafe { self.outcomes.read() };
            self.outcomes = unsafe { self.outcomes.add(1) };
            let outcome = match outcome_variant {
                OutcomeVariant::Passed => Outcome::Passed,
                OutcomeVariant::Ignored => Outcome::Ignored,
                OutcomeVariant::IgnoredWithMessage => {
                    Outcome::IgnoredWithMessage(test.message().unwrap())
                }
                OutcomeVariant::Failed => {
                    // Extract the error message.
                    unsafe {
                        let length = self.data.read();
                        let bytes = self.data.add(1).cast::<u8>();
                        let data = str::from_utf8_unchecked(slice::from_raw_parts(bytes, length));
                        self.data = self.data.byte_add(length + 8).align_forward();
                        Outcome::Failed(data)
                    }
                }
            };
            Some((test, outcome))
        } else {
            None
        }
    }
}

pub(crate) trait Filter {
    fn filter(outcome: &Outcome<&'static str>) -> bool;
}

pub(crate) struct All;

impl Filter for All {
    fn filter(_outcome: &Outcome<&'static str>) -> bool {
        true
    }
}

pub(crate) struct Failed;

impl Filter for Failed {
    fn filter(outcome: &Outcome<&'static str>) -> bool {
        matches!(outcome, &Outcome::Failed(_))
    }
}

pub(crate) struct Passed;

impl Filter for Passed {
    fn filter(outcome: &Outcome<&'static str>) -> bool {
        matches!(outcome, &Outcome::Passed)
    }
}

pub(crate) struct Ignored;

impl Filter for Ignored {
    fn filter(outcome: &Outcome<&'static str>) -> bool {
        matches!(outcome, &Outcome::Ignored | &Outcome::IgnoredWithMessage(_))
    }
}

pub(crate) struct Window<Filter, const SIZE: usize> {
    test_case: *const &'static dyn TestCase,
    outcome: *const OutcomeVariant,
    error_message_front: *const (usize, u8),
    error_message_back: *const (usize, u8),

    length: usize,
    index: usize,

    filtered_length: usize,
    filtered_index: usize,

    filter: PhantomData<Filter>,
}

impl<Filter, const SIZE: usize> Window<Filter, SIZE> {
    fn next_error_message(error_message: &mut *const (usize, u8)) -> &'static str {
        unsafe {
            let length = error_message.cast::<usize>().read();
            let bytes = error_message.cast::<u8>().add(4);
            let next_error_message = bytes.add(length + 4);
            // Re-align.
            *error_message = next_error_message
                .byte_add(4 - (next_error_message as usize % 4) % 4)
                .cast();
            str::from_utf8_unchecked(slice::from_raw_parts(bytes, length))
        }
    }

    fn prev_error_message(error_message: &mut *const (usize, u8)) -> &'static str {
        unsafe {
            let error_message_length = error_message.cast::<usize>().sub(1);
            let length = error_message_length.read();
            let bytes = error_message_length.cast::<u8>().sub(length);
            let prev_error_message = bytes.sub(4);
            // Re-align.
            *error_message = prev_error_message
                .sub(prev_error_message as usize % 4)
                .cast();
            str::from_utf8_unchecked(slice::from_raw_parts(bytes, length))
        }
    }

    fn next_unfiltered(&mut self) -> Option<(&'static dyn TestCase, Outcome<&'static str>)> {
        if self.filtered_index == self.filtered_length.saturating_sub(SIZE) {
            return None;
        }

        unsafe {
            self.test_case = self.test_case.add(1);
            self.outcome = self.outcome.add(1);
        }
        let outcome = match unsafe { self.outcome.add(17).read() } {
            OutcomeVariant::Passed => Outcome::Passed,
            OutcomeVariant::Ignored => Outcome::Ignored,
            OutcomeVariant::IgnoredWithMessage => {
                Outcome::IgnoredWithMessage(unsafe { self.test_case.read() }.message().unwrap())
            }
            OutcomeVariant::Failed => {
                Outcome::Failed(Self::next_error_message(&mut self.error_message_back))
            }
        };
        // Check if the dropped outcome in the window requires moving the error message pointer.
        match unsafe { self.outcome.sub(1).read() } {
            OutcomeVariant::Failed => {
                Self::next_error_message(&mut self.error_message_front);
            }
            _ => {}
        }

        self.index += 1;

        Some((unsafe { self.test_case.read() }, outcome))
    }

    fn prev_unfiltered(&mut self) -> Option<(&'static dyn TestCase, Outcome<&'static str>)> {
        if self.filtered_index == 0 {
            return None;
        }

        unsafe {
            self.test_case = self.test_case.sub(1);
            self.outcome = self.outcome.sub(1);
        }
        let outcome = match unsafe { self.outcome.read() } {
            OutcomeVariant::Passed => Outcome::Passed,
            OutcomeVariant::Ignored => Outcome::Ignored,
            OutcomeVariant::IgnoredWithMessage => {
                Outcome::IgnoredWithMessage(unsafe { self.test_case.read() }.message().unwrap())
            }
            OutcomeVariant::Failed => {
                Outcome::Failed(Self::prev_error_message(&mut self.error_message_front))
            }
        };
        // Check if the dropped outcome in the window requires moving the error message pointer.
        match unsafe { self.outcome.add(SIZE).read() } {
            OutcomeVariant::Failed => {
                Self::prev_error_message(&mut self.error_message_back);
            }
            _ => {}
        }

        self.index -= 1;

        Some((unsafe { self.test_case.read() }, outcome))
    }
}

// We always show a max of SIZE elements on screen at once.
// Whenever we need a new one in either direction, we search using the filter until we find the next element.
// If there is not one, the functions return `None`.
// `get()` will get the outcome currently shown at the given `index`.
impl<Filter, const SIZE: usize> Window<Filter, SIZE>
where
    Filter: self::Filter,
{
    fn calculate_error_message_back(
        mut error_messages: *const (usize, u8),
        mut outcomes: *const OutcomeVariant,
        length: usize,
    ) -> *const (usize, u8) {
        let mut unfiltered_index = 0;
        let mut index = 0;
        while index < SIZE && unfiltered_index < length {
            let outcome = match unsafe { outcomes.read() } {
                OutcomeVariant::Passed => Outcome::Passed,
                OutcomeVariant::Ignored => Outcome::Ignored,
                OutcomeVariant::IgnoredWithMessage => Outcome::IgnoredWithMessage(""),
                OutcomeVariant::Failed => {
                    Outcome::Failed(Self::next_error_message(&mut error_messages))
                }
            };

            if Filter::filter(&outcome) {
                index += 1;
            }
            unfiltered_index += 1;
            unsafe {
                outcomes = outcomes.add(1);
            }
        }
        error_messages
    }

    pub(crate) fn new(test_outcomes: &TestOutcomes, length: usize) -> Self {
        Self {
            test_case: test_outcomes.tests.as_ptr(),
            outcome: test_outcomes.outcomes as *const OutcomeVariant,
            error_message_front: test_outcomes.data as *const (usize, u8),
            error_message_back: Self::calculate_error_message_back(
                test_outcomes.data as *const (usize, u8),
                test_outcomes.outcomes as *const OutcomeVariant,
                test_outcomes.tests.len(),
            ),

            length: test_outcomes.tests.len(),
            index: 0,

            filtered_length: length,
            filtered_index: 0,

            filter: PhantomData,
        }
    }

    pub(crate) fn next(&mut self) -> Option<(&'static dyn TestCase, Outcome<&'static str>)> {
        let old_self = self.clone();

        while let Some((test_case, outcome)) = self.next_unfiltered() {
            if Filter::filter(&outcome) {
                self.filtered_index += 1;
                return Some((test_case, outcome));
            }
        }
        // We reached the end of the list and found nothing not filtered.
        // Reset state and return nothing.
        *self = old_self;
        None
    }

    pub(crate) fn prev(&mut self) -> Option<(&'static dyn TestCase, Outcome<&'static str>)> {
        let old_self = self.clone();

        while let Some((test_case, outcome)) = self.prev_unfiltered() {
            if Filter::filter(&outcome) {
                self.filtered_index -= 1;
                return Some((test_case, outcome));
            }
        }
        // We reached the beginning of the list and found nothing not filtered.
        // Reset state and return nothing.
        *self = old_self;
        None
    }

    pub(crate) fn iter(
        &self,
    ) -> impl Iterator<Item = (&'static dyn TestCase, Outcome<&'static str>)> {
        TestOutcomesIter {
            tests: unsafe { slice::from_raw_parts(self.test_case, self.length - self.index) }
                .iter(),
            outcomes: self.outcome as *mut OutcomeVariant,
            data: self.error_message_front as *mut usize,
        }
        .filter(|(_, outcome)| Filter::filter(&outcome))
    }

    pub(crate) fn get(&self, index: usize) -> Option<(&dyn TestCase, Outcome<&'static str>)> {
        self.iter().skip(index).next()
    }
}

impl<Filter, const SIZE: usize> Clone for Window<Filter, SIZE> {
    fn clone(&self) -> Self {
        Self {
            test_case: self.test_case,
            outcome: self.outcome,
            error_message_front: self.error_message_front,
            error_message_back: self.error_message_back,

            length: self.length,
            index: self.index,

            filtered_length: self.filtered_length,
            filtered_index: self.filtered_index,

            filter: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Outcome, OutcomeVariant};
    use claims::assert_matches;
    use gba_test_macros::test;

    #[test]
    fn outcome_as_str_passed() {
        assert_eq!(Outcome::<()>::Passed.as_str(), "ok");
    }

    #[test]
    fn outcome_as_str_failed() {
        assert_eq!(Outcome::<()>::Failed(()).as_str(), "FAILED");
    }

    #[test]
    fn outcome_as_str_ignored() {
        assert_eq!(Outcome::<()>::Ignored.as_str(), "ignored");
    }

    #[test]
    fn outcome_as_str_ignored_with_message() {
        assert_eq!(Outcome::<()>::IgnoredWithMessage("").as_str(), "ignored");
    }

    #[test]
    fn outcome_into_outcome_variant_passed() {
        assert_matches!(
            OutcomeVariant::from(&Outcome::<&str>::Passed),
            OutcomeVariant::Passed
        );
    }

    #[test]
    fn outcome_into_outcome_variant_failed() {
        assert_matches!(
            OutcomeVariant::from(&Outcome::<&str>::Failed("foo")),
            OutcomeVariant::Failed
        );
    }

    #[test]
    fn outcome_into_outcome_variant_ignored() {
        assert_matches!(
            OutcomeVariant::from(&Outcome::<&str>::Ignored),
            OutcomeVariant::Ignored
        );
    }
}
