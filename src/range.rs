use crate::prelude::*;

pub struct Iter {
    range: std::ops::Range<u64>,
}

impl IntoParallelIterator for std::ops::Range<u64> {
    type Item = u64;
    type Iter = Iter;
    fn into_par_iter(self) -> Self::Iter {
        Iter { range: self }
    }
}

impl ParallelIterator for Iter {
    type Item = u64;
    type Controlled = True;
    type Enumerable = True;
    fn with_producer<CB>(self, callback: CB) -> CB::Output
    where
        CB: ProducerCallback<Self::Item>,
    {
        callback.call(self.range)
    }
}

impl Divisible for std::ops::Range<u64> {
    type Controlled = True;
    fn should_be_divided(&self) -> bool {
        (self.end - self.start) >= 2
    }
    fn divide(self) -> (Self, Self) {
        // We need to divide with the biggest half on the left, so instead of just doing
        // (end - start) / 2, which in cases of an odd length will put the biggest
        // half on the right, we need to make sure we take the ceiling of the value:
        // this is done using (end - start) - ((end - start) / 2).
        let index = (self.end - self.start) - ((self.end - self.start)/ 2);
        self.divide_at(index as usize)
    }
    fn divide_at(self, index: usize) -> (Self, Self) {
        let mid = self.start + (index as u64);
        (self.start..mid, mid..self.end)
    }
}

impl Producer for std::ops::Range<u64> {
    fn preview(&self, index: usize) -> Self::Item {
        self.start + index as u64
    }
}

impl PreviewableParallelIterator for Iter {}
