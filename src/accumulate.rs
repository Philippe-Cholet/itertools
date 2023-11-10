use std::fmt;

use crate::size_hint::{self, SizeHint};

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
