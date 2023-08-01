pub trait VecItems<T> {
    type Output;
    fn new_item<I: Iterator<Item = T>>(&mut self, iter: I) -> Self::Output;
}

#[derive(Clone)]
pub struct CollectToVec;

pub struct MapSlice<F, T> {
    func: F,
    vec: Vec<T>,
}

impl<F, T> MapSlice<F, T> {
    pub fn with_capacity(func: F, capacity: usize) -> Self {
        Self { func, vec: Vec::with_capacity(capacity) }
    }
}

impl<T> VecItems<T> for CollectToVec {
    type Output = Vec<T>;
    #[inline]
    fn new_item<I: Iterator<Item = T>>(&mut self, iter: I) -> Self::Output {
        iter.collect()
    }
}

impl<R, F, T> VecItems<T> for MapSlice<F, T>
where
    F: FnMut(&[T]) -> R,
{
    type Output = R;
    #[inline]
    fn new_item<I: Iterator<Item = T>>(&mut self, iter: I) -> Self::Output {
        debug_assert!(self.vec.is_empty());
        self.vec.extend(iter);
        let result = (self.func)(&self.vec);
        self.vec.clear();
        result
    }
}
