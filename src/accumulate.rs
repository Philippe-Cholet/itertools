use std::fmt;
use std::iter::FusedIterator;

use crate::size_hint::{self, SizeHint};

pub(crate) fn accumulate<I, F>(iter: I, func: F) -> Accumulate<I::IntoIter, F>
where
    I: IntoIterator,
    F: FnMut(&I::Item, I::Item) -> I::Item,
{
    Accumulate {
        iter: iter.into_iter(),
        peeked: None,
        func,
    }
}

#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct Accumulate<I: Iterator, F> {
    iter: I,
    peeked: Option<Option<I::Item>>,
    func: F,
}

impl<I: Iterator, F> fmt::Debug for Accumulate<I, F>
where
    I: fmt::Debug,
    I::Item: fmt::Debug,
{
    debug_fmt_fields!(AccumulateFrom, iter, peeked);
}

impl<I: Iterator, F> Clone for Accumulate<I, F>
where
    I: Clone,
    I::Item: Clone,
    F: Clone,
{
    clone_fields!(iter, peeked, func);
}

impl<I, F> Iterator for Accumulate<I, F>
where
    I: Iterator,
    F: FnMut(&I::Item, I::Item) -> I::Item,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let Self { iter, peeked, func } = self;
        let accum = peeked.get_or_insert_with(|| iter.next());
        let acc = accum.as_ref()?;
        match iter.next() {
            Some(item) => {
                let b = func(acc, item);
                accum.replace(b)
            }
            None => accum.take(),
        }
    }

    fn size_hint(&self) -> SizeHint {
        match self.peeked {
            None => self.iter.size_hint(),
            Some(Some(_)) => size_hint::add_scalar(self.iter.size_hint(), 1),
            Some(None) => (0, Some(0)),
        }
    }
}

impl<I, F> FusedIterator for Accumulate<I, F>
where
    I: Iterator,
    F: FnMut(&I::Item, I::Item) -> I::Item,
{
}

impl<I, F> ExactSizeIterator for Accumulate<I, F>
where
    I: ExactSizeIterator,
    F: FnMut(&I::Item, I::Item) -> I::Item,
{
}

pub(crate) fn accumulate_from<I, B, F>(
    iter: I,
    init: B,
    func: F,
) -> AccumulateFrom<I::IntoIter, B, F>
where
    I: IntoIterator,
    F: FnMut(&B, I::Item) -> B,
{
    AccumulateFrom {
        iter: iter.into_iter(),
        accum: Some(init),
        func,
    }
}

#[derive(Clone)]
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct AccumulateFrom<I, B, F> {
    iter: I,
    accum: Option<B>,
    func: F,
}

impl<I, B, F> fmt::Debug for AccumulateFrom<I, B, F>
where
    I: fmt::Debug,
    B: fmt::Debug,
{
    debug_fmt_fields!(AccumulateFrom, iter, accum);
}

impl<I, B, F> Iterator for AccumulateFrom<I, B, F>
where
    I: Iterator,
    F: FnMut(&B, I::Item) -> B,
{
    type Item = B;

    fn next(&mut self) -> Option<Self::Item> {
        let acc = self.accum.as_ref()?;
        match self.iter.next() {
            Some(item) => {
                let b = (self.func)(acc, item);
                self.accum.replace(b)
            }
            None => self.accum.take(),
        }
    }

    fn size_hint(&self) -> SizeHint {
        if self.accum.is_none() {
            return (0, Some(0));
        }
        size_hint::add_scalar(self.iter.size_hint(), 1)
    }
}

impl<I, B, F> FusedIterator for AccumulateFrom<I, B, F>
where
    I: Iterator,
    F: FnMut(&B, I::Item) -> B,
{
}
