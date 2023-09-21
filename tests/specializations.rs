use std::fmt::Debug;

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

/// Helper to check iterator specializations.
#[derive(Clone)]
pub struct SpecializationChecker<I: Iterator> {
    iter: I,
    unspec: Unspecialized<I>,
    cached_count: usize,
    iteration_limit: usize,
}

impl<I: Iterator + Clone> SpecializationChecker<I> {
    /// Create a new checker.
    pub fn new(iter: I) -> Self {
        let unspec = Unspecialized(iter.clone());
        Self {
            cached_count: unspec.clone().count(),
            iter,
            unspec,
            iteration_limit: usize::MAX,
        }
    }

    /// Limit test iterations to the ... first steps.
    pub fn with_iteration_limit(mut self, iteration_limit: usize) -> Self {
        self.iteration_limit = iteration_limit;
        self
    }

    fn check_every_step<F>(mut self, mut check_step: F)
    where
        F: FnMut(usize, &I, &Unspecialized<I>),
    {
        let mut step = 0;
        check_step(step, &self.iter, &self.unspec);
        while let Some(_) = self.iter.next() {
            if step == self.iteration_limit {
                return;
            }
            step += 1;
            let next_unspec = self.unspec.next();
            debug_assert!(next_unspec.is_some(), "unspec shorter than iter");
            check_step(step, &self.iter, &self.unspec);
        }
        step += 1;
        let next_unspec = self.unspec.next();
        debug_assert!(next_unspec.is_none(), "iter shorter than unspec");
        check_step(step, &self.iter, &self.unspec);
    }

    // Methods below immutably borrow `self` because a
    // "specialization check" must not interfere with the next ones.

    /// Check all size hints.
    pub fn exact_size_hints(&self) -> &Self {
        let unspec_count = self.cached_count;
        self.clone().check_every_step(|step, iter, _| {
            let sh = iter.size_hint();
            assert_eq!(
                Some(sh.0),
                sh.1,
                "Inexact size hint at step {}: {:?}",
                step,
                sh
            );
            let remaining_count = unspec_count.saturating_sub(step);
            assert_eq!(
                remaining_count, sh.0,
                "Wrong size hint change at step {}: {:?} but count={:?}",
                step, sh, remaining_count
            );
        });
        self
    }

    /// Check the count and compare with a value you expect if provided.
    pub fn count(&self, expected: impl Into<Option<usize>>) -> &Self {
        let unspec_count = self.cached_count;
        if let Some(value) = expected.into() {
            assert_eq!(
                unspec_count, value,
                "Count: expected {} but got {}",
                value, unspec_count
            );
        }
        self.clone().check_every_step(|step, iter, _| {
            let unspec_res = unspec_count.saturating_sub(step);
            let got = iter.clone().count();
            assert_eq!(
                unspec_res, got,
                "Count (from step {}): expected {:?} but got {:?}",
                step, unspec_res, got
            );
        });
        self
    }

    /// Check the last element.
    pub fn last(&self) -> &Self
    where
        I::Item: Clone + PartialEq + Debug,
    {
        let count = self.cached_count;
        let unspec_last = self.unspec.clone().last();
        self.clone().check_every_step(|step, iter, _| {
            let expected = if step < count {
                unspec_last.clone()
            } else {
                None
            };
            let got = iter.clone().last();
            assert_eq!(
                expected, got,
                "Last (from step {}): expected {:?} but got {:?}",
                step, expected, got
            );
        });
        self
    }

    /// Check the `nth(0)..=nth(10)` elements.
    pub fn nth(&self) -> &Self
    where
        I::Item: Clone + PartialEq + Debug,
    {
        let length = self.cached_count;
        let items: Vec<_> = self.unspec.clone().collect();
        self.clone().check_every_step(|step, iter, _| {
            let max_n = (length + 5).saturating_sub(step);
            // Way slower without limiting `max_n`.
            let max_n = max_n.min(10);
            (0..=max_n).for_each(|n| {
                let unspec_n = items.get(step + n).cloned();
                let got = iter.clone().nth(n);
                assert_eq!(
                    unspec_n, got,
                    "Nth (from step {}): expected {:?} but got {:?}",
                    step, unspec_n, got
                );
            })
        });
        self
    }

    /// Check the folded elements.
    pub fn fold<B, F>(&self, init: B, f: F) -> &Self
    where
        F: FnMut(B, I::Item) -> B + Clone,
        B: Clone + PartialEq + Debug,
    {
        let mut it_items = Vec::with_capacity(self.cached_count);
        let mut unspec_items = Vec::with_capacity(self.cached_count);
        self.clone().check_every_step(|step, iter, unspec| {
            // Re-use those vectors to save re-allocations.
            it_items.clear();
            unspec_items.clear();
            let mut it_f = f.clone();
            iter.clone().fold(init.clone(), |acc, item| {
                let res = it_f(acc, item);
                it_items.push(res.clone());
                res
            });
            let mut unspec_f = f.clone();
            unspec.clone().fold(init.clone(), |acc, item| {
                let res = unspec_f(acc, item);
                unspec_items.push(res.clone());
                res
            });
            assert_eq!(
                unspec_items, it_items,
                "Fold (from step {}): expected {:?} but got {:?}",
                step, unspec_items, it_items
            );
        });
        self
    }

    /// Check the collected elements.
    pub fn collect(&self) -> &Self
    where
        I::Item: Debug + PartialEq,
    {
        self.clone().check_every_step(|step, iter, unspec| {
            let unspec_items: Vec<_> = unspec.clone().collect();
            let it_items: Vec<_> = iter.clone().collect();
            assert_eq!(
                unspec_items, it_items,
                "Collect (from step {}): expected {:?} but got {:?}",
                step, unspec_items, it_items
            )
        });
        self
    }
}
