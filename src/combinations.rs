use std::fmt;
use std::iter::FusedIterator;

use super::lazy_buffer::LazyBuffer;
use super::vec_items::{VecItems, CollectToVec, MapSlice};
use alloc::vec::Vec;

/// An iterator to iterate through all the `k`-length combinations in an iterator.
///
/// See [`.combinations()`](crate::Itertools::combinations) for more information.
pub type Combinations<I> = CombinationsBase<I, CollectToVec>;

/// TODO: COPY/UPDATE DOC
pub type CombinationsMap<I, F> = CombinationsBase<I, MapSlice<F, <I as Iterator>::Item>>;

#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct CombinationsBase<I: Iterator, F> {
    manager: F,
    indices: Vec<usize>,
    pool: LazyBuffer<I>,
    first: bool,
}

impl<I, F> Clone for CombinationsBase<I, F>
    where I: Clone + Iterator,
          I::Item: Clone,
          F: Clone,
{
    clone_fields!(manager, indices, pool, first);
}

impl<I, F> fmt::Debug for CombinationsBase<I, F>
    where I: Iterator + fmt::Debug,
          I::Item: fmt::Debug,
{
    debug_fmt_fields!(CombinationsBase, indices, pool, first);
}

/// Create a new `Combinations` from a clonable iterator.
pub fn combinations<I>(iter: I, k: usize) -> Combinations<I>
    where I: Iterator
{
    let mut pool = LazyBuffer::new(iter);
    pool.prefill(k);

    CombinationsBase {
        manager: CollectToVec,
        indices: (0..k).collect(),
        pool,
        first: true,
    }
}

/// TODO: COPY/UPDATE DOC
pub fn combinations_map<I, F>(iter: I, k: usize, f: F) -> CombinationsMap<I, F>
    where I: Iterator
{
    let mut pool = LazyBuffer::new(iter);
    pool.prefill(k);

    CombinationsBase {
        manager: MapSlice::with_capacity(f, k),
        indices: (0..k).collect(),
        pool,
        first: true,
    }
}

impl<I: Iterator, F> CombinationsBase<I, F> {
    /// Returns the length of a combination produced by this iterator.
    #[inline]
    pub fn k(&self) -> usize { self.indices.len() }

    /// Returns the (current) length of the pool from which combination elements are
    /// selected. This value can change between invocations of [`next`](Combinations::next).
    #[inline]
    pub fn n(&self) -> usize { self.pool.len() }

    /// Returns a reference to the source iterator.
    #[inline]
    pub(crate) fn src(&self) -> &I { &self.pool.it }

    /// Resets this `Combinations` back to an initial state for combinations of length
    /// `k` over the same pool data source. If `k` is larger than the current length
    /// of the data pool an attempt is made to prefill the pool so that it holds `k`
    /// elements.
    pub(crate) fn reset(&mut self, k: usize) {
        self.first = true;

        if k < self.indices.len() {
            self.indices.truncate(k);
            for i in 0..k {
                self.indices[i] = i;
            }

        } else {
            for i in 0..self.indices.len() {
                self.indices[i] = i;
            }
            self.indices.extend(self.indices.len()..k);
            self.pool.prefill(k);
        }
    }
}

impl<I, F> Iterator for CombinationsBase<I, F>
    where I: Iterator,
          I::Item: Clone,
          F: VecItems<I::Item>,
{
    type Item = F::Output;
    fn next(&mut self) -> Option<Self::Item> {
        if self.first {
            if self.k() > self.n() {
                return None;
            }
            self.first = false;
        } else if self.indices.is_empty() {
            return None;
        } else {
            // Scan from the end, looking for an index to increment
            let mut i: usize = self.indices.len() - 1;

            // Check if we need to consume more from the iterator
            if self.indices[i] == self.pool.len() - 1 {
                self.pool.get_next(); // may change pool size
            }

            while self.indices[i] == i + self.pool.len() - self.indices.len() {
                if i > 0 {
                    i -= 1;
                } else {
                    // Reached the last combination
                    return None;
                }
            }

            // Increment index, and reset the ones to its right
            self.indices[i] += 1;
            for j in i+1..self.indices.len() {
                self.indices[j] = self.indices[j - 1] + 1;
            }
        }

        // Create result vector based on the indices
        let Self { manager, indices, pool, .. } = self;
        Some(manager.new_item(indices.iter().map(|i| pool[*i].clone())))
    }
}

impl<I, F> FusedIterator for CombinationsBase<I, F>
    where I: Iterator,
          I::Item: Clone,
          F: VecItems<I::Item>,
{}
