use crate::test_case::TestCase;
use core::{
    fmt,
    fmt::{Display, Write},
    marker::PhantomData,
    mem::size_of,
    ptr, slice, str,
};

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
}

impl<'a, Data> From<&'a Outcome<Data>> for OutcomeVariant {
    fn from(outcome: &'a Outcome<Data>) -> OutcomeVariant {
        match outcome {
            Outcome::Passed => OutcomeVariant::Passed,
            Outcome::Failed(_) => OutcomeVariant::Failed,
            Outcome::Ignored => OutcomeVariant::Ignored,
        }
    }
}

/// This basically acts as a bump allocator for error messages.
struct ErrorMessages {
    start: *mut (usize, u8),
    current: *mut (usize, u8),
}

impl ErrorMessages {
    unsafe fn new(start: *mut OutcomeVariant) -> Self {
        // Get alignment offset.
        let pointer = (start as *mut u8).add(4 - (start as usize % 4) % 4) as *mut (usize, u8);
        Self {
            start: pointer,
            current: pointer,
        }
    }

    fn create_message(&mut self) -> ErrorMessage {
        let cursor = unsafe { self.current.cast::<u8>().add(size_of::<usize>()) };
        ErrorMessage {
            error_messages: self,
            cursor,
        }
    }
}

// TODO: Handle errors here.
struct ErrorMessage<'a> {
    error_messages: &'a mut ErrorMessages,
    cursor: *mut u8,
}

impl Write for ErrorMessage<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let b = s.as_bytes();
        unsafe {
            ptr::copy(b.as_ptr(), self.cursor, b.len());
            self.cursor = self.cursor.add(b.len());
        }
        Ok(())
    }
}

impl Drop for ErrorMessage<'_> {
    fn drop(&mut self) {
        unsafe {
            // Set length of the written data.
            let length =
                self.cursor as usize - self.error_messages.current.cast::<u8>().add(4) as usize;
            let length_start = self.error_messages.current.cast::<usize>();
            let length_end = self
                .error_messages
                .current
                .cast::<usize>()
                .byte_add(4 + length + (4 - (length % 4) % 4));
            *length_start = length;
            *length_end = length;
            log::info!("length: {}", length);
            // Set new current to the next aligned location.
            self.error_messages.current = self
                .cursor
                .add((4 - (self.cursor as usize % 4) % 4) + 4)
                .cast();
        }
    }
}

pub(crate) struct Outcomes {
    outcomes: *mut OutcomeVariant,
    current_outcome: *mut OutcomeVariant,
    length: usize, // Can also be calculated by distance to next pointer (error_messages, that is).
    error_messages: ErrorMessages,
}

impl Outcomes {
    pub(crate) unsafe fn new(start: *mut u8, length: usize) -> Self {
        // TODO: Some kind of assertion on the length.
        let pointer = start as *mut OutcomeVariant;
        Self {
            outcomes: pointer,
            current_outcome: pointer,
            length,
            error_messages: ErrorMessages::new(pointer.add(length)),
        }
    }

    pub(crate) fn push_outcome<Data>(&mut self, outcome: Outcome<Data>)
    where
        Data: Display,
    {
        unsafe {
            self.current_outcome.write_volatile((&outcome).into());
            self.current_outcome = self.current_outcome.add(1);
        }
        if let Outcome::Failed(data) = outcome {
            log::info!("data: {}", data);
            write!(self.error_messages.create_message(), "{}", data);
        }
    }

    pub(crate) fn iter_outcomes(&self) -> OutcomesIter {
        unsafe { OutcomesIter::new(self.outcomes, self.error_messages.start, self.length) }
    }
}

pub(crate) struct OutcomesIter {
    outcomes: *const OutcomeVariant,
    error_messages: *const (usize, u8),
    length: usize,
}

impl OutcomesIter {
    unsafe fn new(
        start: *const OutcomeVariant,
        error_messages_start: *const (usize, u8),
        length: usize,
    ) -> Self {
        Self {
            outcomes: start,
            error_messages: error_messages_start,
            length,
        }
    }
}

impl Iterator for OutcomesIter {
    type Item = Outcome<&'static str>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.length > 0 {
            let outcome_variant = unsafe { self.outcomes.read_volatile() };
            self.outcomes = unsafe { self.outcomes.add(1) };
            self.length -= 1;
            Some(match outcome_variant {
                OutcomeVariant::Passed => Outcome::Passed,
                OutcomeVariant::Ignored => Outcome::Ignored,
                OutcomeVariant::Failed => {
                    // Extract the error message.
                    unsafe {
                        let length = *self.error_messages.cast::<usize>();
                        let bytes = self.error_messages.cast::<u8>().add(4);
                        let data = str::from_utf8_unchecked(slice::from_raw_parts(bytes, length));
                        self.error_messages = self.error_messages.byte_add(length + 4);
                        // Align.
                        self.error_messages = self
                            .error_messages
                            .byte_add(8 - (self.error_messages as usize % 4) % 4);
                        Outcome::Failed(data)
                    }
                }
            })
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
        matches!(outcome, &Outcome::Ignored)
    }
}

#[derive(Debug)]
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
        // TODO: This doesn't work with filters, because it treats some displayed values as though they are still undisplayed, resulting in the list scrolling too far.
        let outcome = match unsafe { self.outcome.add(17).read() } {
            OutcomeVariant::Passed => Outcome::Passed,
            OutcomeVariant::Ignored => Outcome::Ignored,
            OutcomeVariant::Failed => {
                Outcome::Failed(Self::next_error_message(&mut self.error_message_back))
            }
        };
        // Check if the dropped outcome in the window requires moving the error message pointer.
        if matches!(
            unsafe { self.outcome.sub(1).read() },
            OutcomeVariant::Failed
        ) {
            Self::next_error_message(&mut self.error_message_front);
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
            OutcomeVariant::Failed => {
                Outcome::Failed(Self::prev_error_message(&mut self.error_message_front))
            }
        };
        // Check if the dropped outcome in the window requires moving the error message pointer.
        if matches!(
            unsafe { self.outcome.add(SIZE).read() },
            OutcomeVariant::Failed
        ) {
            Self::prev_error_message(&mut self.error_message_back);
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

    pub(crate) fn new(
        tests: &'static [&'static dyn TestCase],
        outcomes: &Outcomes,
        length: usize,
    ) -> Self {
        Self {
            test_case: tests.as_ptr(),
            outcome: outcomes.outcomes,
            error_message_front: outcomes.error_messages.start,
            error_message_back: Self::calculate_error_message_back(
                outcomes.error_messages.start,
                outcomes.outcomes,
                tests.len(),
            ),

            length: tests.len(),
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

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&dyn TestCase, Outcome<&'static str>)> {
        unsafe { slice::from_raw_parts(self.test_case, self.length - self.index) }
            .into_iter()
            .copied()
            .zip(OutcomesIter {
                outcomes: self.outcome,
                error_messages: self.error_message_front,
                length: self.length - self.index,
            })
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
