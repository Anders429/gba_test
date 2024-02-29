use core::{fmt, fmt::{Display, Write}, mem::size_of, ptr, str, slice};

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
#[derive(Debug)]
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
        let cursor = unsafe {self.current.cast::<u8>().add(size_of::<usize>())};
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
            let length = self.error_messages.current.cast::<usize>();
            log::info!("length: {}", self.cursor as usize - self.error_messages.current.cast::<u8>().add(4) as usize);
            *length = self.cursor as usize - self.error_messages.current.cast::<u8>().add(4) as usize;
            // Set new current to the next aligned location.
            self.error_messages.current = self.cursor.add(4 - (self.cursor as usize % 4) % 4).cast();
        }
    }
}

pub(crate) struct Outcomes {
    outcomes: *mut OutcomeVariant,
    current_outcome: *mut OutcomeVariant,
    length: usize,  // Can also be calculated by distance to next pointer (error_messages, that is).
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

    pub(crate) fn push_outcome<Data>(&mut self, outcome: Outcome<Data>) where Data: Display {
        unsafe {self.current_outcome.write_volatile((&outcome).into());
        self.current_outcome = self.current_outcome.add(1);}
        if let Outcome::Failed(data) = outcome {
            log::info!("data: {}", data);
            write!(self.error_messages.create_message(), "{}", data);
        }
    }

    pub(crate) fn iter_outcomes(&self) -> OutcomesIter {
        unsafe {OutcomesIter::new(self.outcomes, self.error_messages.start, self.length)}
    }
}

pub(crate) struct OutcomesIter {
    outcomes: *mut OutcomeVariant,
    error_messages: *mut (usize, u8),
    length: usize,
}

impl OutcomesIter {
    unsafe fn new(start: *mut OutcomeVariant, error_messages_start: *mut (usize, u8), length: usize) -> Self {
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
            let outcome_variant = unsafe {self.outcomes.read_volatile()};
            self.outcomes = unsafe {self.outcomes.add(1)};
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
                        self.error_messages = self.error_messages.byte_add(4 - (self.error_messages as usize % 4) % 4);
                        Outcome::Failed(data)
                    }
                },
            })
        } else {
            None
        }
    }
}
