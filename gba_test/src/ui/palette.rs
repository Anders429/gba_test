use crate::Outcome;

impl<Data> Outcome<Data> {
    pub(super) fn palette(&self) -> u8 {
        match self {
            Self::Passed => 1,
            Self::Ignored => 2,
            Self::Failed(_) => 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Outcome;
    use gba_test_macros::test;

    #[test]
    fn passed() {
        assert_eq!(Outcome::<()>::Passed.palette(), 1);
    }

    #[test]
    fn ignored() {
        assert_eq!(Outcome::<()>::Ignored.palette(), 2);
    }

    #[test]
    fn failed() {
        assert_eq!(Outcome::<()>::Failed(()).palette(), 3);
    }
}
