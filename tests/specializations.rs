/// Wraps an iterator to unspecialize all its (DoubleEnded)Iterator methods.
#[derive(Clone)]
pub struct Unspecialized<I>(pub I);

impl<I: Iterator> Iterator for Unspecialized<I> {
    type Item = I::Item;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    // All other methods are unspecialized!
}

// Should we care?
// impl<I: FusedIterator> FusedIterator for Unspecialized<I> {}

// NOTE: size_hint is unspecialized so we can't implement ExactSizeIterator.

impl<I: DoubleEndedIterator> DoubleEndedIterator for Unspecialized<I> {
    #[inline(always)]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }

    // All other methods are unspecialized!
}
