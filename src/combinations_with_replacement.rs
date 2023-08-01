use alloc::vec::Vec;
use std::fmt;
use std::iter::FusedIterator;

use super::lazy_buffer::LazyBuffer;
use super::vec_items::{VecItems, CollectToVec, MapSlice};

/// An iterator to iterate through all the `n`-length combinations in an iterator, with replacement.
///
/// See [`.combinations_with_replacement()`](crate::Itertools::combinations_with_replacement)
/// for more information.
pub type CombinationsWithReplacement<I> = CombinationsWithReplacementBase<I, CollectToVec>;

/// TODO: COPY/UPDATE DOC
pub type CombinationsWithReplacementMap<I, F> = CombinationsWithReplacementBase<I, MapSlice<F, <I as Iterator>::Item>>;

pub struct CombinationsWithReplacementBase<I, F>
where
    I: Iterator,
    I::Item: Clone,
{
    manager: F,
    indices: Vec<usize>,
    pool: LazyBuffer<I>,
    first: bool,
}

impl<I, F> Clone for CombinationsWithReplacementBase<I, F>
where
    I: Iterator + Clone,
    I::Item: Clone,
    F: Clone,
{
    clone_fields!(manager, indices, pool, first);
}

impl<I, F> fmt::Debug for CombinationsWithReplacementBase<I, F>
where
    I: Iterator + fmt::Debug,
    I::Item: fmt::Debug + Clone,
{
    debug_fmt_fields!(Combinations, indices, pool, first);
}

/// Create a new `CombinationsWithReplacement` from a clonable iterator.
pub fn combinations_with_replacement<I>(iter: I, k: usize) -> CombinationsWithReplacement<I>
where
    I: Iterator,
    I::Item: Clone,
{
    let indices: Vec<usize> = alloc::vec![0; k];
    let pool: LazyBuffer<I> = LazyBuffer::new(iter);

    CombinationsWithReplacementBase {
        manager: CollectToVec,
        indices,
        pool,
        first: true,
    }
}

/// TODO: COPY/UPDATE DOC
pub fn combinations_with_replacement_map<I, F>(iter: I, k: usize, f: F) -> CombinationsWithReplacementMap<I, F>
where
    I: Iterator,
    I::Item: Clone,
{
    let indices: Vec<usize> = alloc::vec![0; k];
    let pool: LazyBuffer<I> = LazyBuffer::new(iter);

    CombinationsWithReplacementBase {
        manager: MapSlice::with_capacity(f, k),
        indices,
        pool,
        first: true,
    }
}

impl<I, F> Iterator for CombinationsWithReplacementBase<I, F>
where
    I: Iterator,
    I::Item: Clone,
    F: VecItems<I::Item>,
{
    type Item = F::Output;
    fn next(&mut self) -> Option<Self::Item> {
        // If this is the first iteration, return early
        if self.first {
            // In empty edge cases, stop iterating immediately
            return if !(self.indices.is_empty() || self.pool.get_next()) {
                None
            // Otherwise, yield the initial state
            } else {
                self.first = false;
                let Self { manager, ref indices, ref pool, .. } = self;
                Some(manager.new_item(indices.iter().map(|i| pool[*i].clone())))
            };
        }

        // Check if we need to consume more from the iterator
        // This will run while we increment our first index digit
        self.pool.get_next();

        // Work out where we need to update our indices
        let mut increment: Option<(usize, usize)> = None;
        for (i, indices_int) in self.indices.iter().enumerate().rev() {
            if *indices_int < self.pool.len()-1 {
                increment = Some((i, indices_int + 1));
                break;
            }
        }

        match increment {
            // If we can update the indices further
            Some((increment_from, increment_value)) => {
                // We need to update the rightmost non-max value
                // and all those to the right
                for indices_index in increment_from..self.indices.len() {
                    self.indices[indices_index] = increment_value;
                }
                let Self { manager, ref indices, ref pool, .. } = self;
                Some(manager.new_item(indices.iter().map(|i| pool[*i].clone())))
            }
            // Otherwise, we're done
            None => None,
        }
    }
}

impl<I, F> FusedIterator for CombinationsWithReplacementBase<I, F>
where
    I: Iterator,
    I::Item: Clone,
    F: VecItems<I::Item>,
{}
