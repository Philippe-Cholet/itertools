use std::fmt;
use std::iter::FusedIterator;
use std::usize;

use super::combinations::{CombinationsBase, combinations, combinations_map};
use super::size_hint;
use super::vec_items::{VecItems, CollectToVec, MapSlice};

/// An iterator to iterate through the powerset of the elements from an iterator.
///
/// See [`.powerset()`](crate::Itertools::powerset) for more
/// information.
pub type Powerset<I> = PowersetBase<I, CollectToVec>;

/// TODO: COPY/UPDATE DOC
pub type PowersetMap<I, F> = PowersetBase<I, MapSlice<F, <I as Iterator>::Item>>;

#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct PowersetBase<I: Iterator, F> {
    combs: CombinationsBase<I, F>,
    // Iterator `position` (equal to count of yielded elements).
    pos: usize,
}

impl<I, F> Clone for PowersetBase<I, F>
    where I: Clone + Iterator,
          I::Item: Clone,
          F: Clone,
{
    clone_fields!(combs, pos);
}

impl<I, F> fmt::Debug for PowersetBase<I, F>
    where I: Iterator + fmt::Debug,
          I::Item: fmt::Debug,
{
    debug_fmt_fields!(PowersetBase, combs, pos);
}

/// Create a new `Powerset` from a clonable iterator.
pub fn powerset<I>(src: I) -> Powerset<I>
    where I: Iterator,
          I::Item: Clone,
{
    PowersetBase {
        combs: combinations(src, 0),
        pos: 0,
    }
}

/// TODO: COPY/UPDATE DOC
pub fn powerset_map<I, F>(src: I, f: F) -> PowersetMap<I, F>
    where I: Iterator,
          I::Item: Clone,
{
    PowersetBase {
        combs: combinations_map(src, 0, f),
        pos: 0,
    }
}

impl<I, F> Iterator for PowersetBase<I, F>
    where
        I: Iterator,
        I::Item: Clone,
        F: VecItems<I::Item>,
{
    type Item = F::Output;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(elt) = self.combs.next() {
            self.pos = self.pos.saturating_add(1);
            Some(elt)
        } else if self.combs.k() < self.combs.n()
            || self.combs.k() == 0
        {
            self.combs.reset(self.combs.k() + 1);
            self.combs.next().map(|elt| {
                self.pos = self.pos.saturating_add(1);
                elt
            })
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // Total bounds for source iterator.
        let src_total = size_hint::add_scalar(self.combs.src().size_hint(), self.combs.n());

        // Total bounds for self ( length(powerset(set) == 2 ^ length(set) )
        let self_total = size_hint::pow_scalar_base(2, src_total);

        if self.pos < usize::MAX {
            // Subtract count of elements already yielded from total.
            size_hint::sub_scalar(self_total, self.pos)
        } else {
            // Fallback: self.pos is saturated and no longer reliable.
            (0, self_total.1)
        }
    }
}

impl<I, F> FusedIterator for PowersetBase<I, F>
    where
        I: Iterator,
        I::Item: Clone,
        F: VecItems<I::Item>,
{}
