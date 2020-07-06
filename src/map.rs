use crate::prelude::*;

pub struct Map<I, F> {
    pub(crate) base: I,
    pub(crate) op: F,
}

impl<R, I, F> ParallelIterator for Map<I, F>
where
    R: Send,
    I: ParallelIterator,
    F: Fn(I::Item) -> R + Sync + Send,
{
    type Item = R;
    type Controlled = I::Controlled;
    type Enumerable = I::Enumerable;

    fn drive<C: Consumer<Self::Item>>(self, consumer: C) -> C::Result {
        let c = MapConsumer {
            op: self.op,
            base: consumer,
        };
        self.base.drive(c)
    }

    fn with_producer<CB>(self, callback: CB) -> CB::Output
    where
        CB: ProducerCallback<Self::Item>,
    {
        return self.base.with_producer(Callback {
            callback,
            op: self.op,
        });
        struct Callback<CB, F> {
            callback: CB,
            op: F,
        }
        impl<T, F, R, CB> ProducerCallback<T> for Callback<CB, F>
        where
            CB: ProducerCallback<R>,
            F: Fn(T) -> R + Sync,
        {
            type Output = CB::Output;
            fn call<P>(self, base: P) -> CB::Output
            where
                P: Producer<Item = T>,
            {
                let producer = MapProducer { base, op: &self.op };
                self.callback.call(producer)
            }
        }
    }
}

struct MapProducer<'f, I, F> {
    base: I,
    op: &'f F,
}

impl<'f, R, I, F> Iterator for MapProducer<'f, I, F>
where
    I: Iterator,
    F: Fn(I::Item) -> R,
{
    type Item = R;
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.base.size_hint()
    }
    fn next(&mut self) -> Option<Self::Item> {
        self.base.next().map(self.op)
    }
}

impl<'f, R, I, F> Divisible for MapProducer<'f, I, F>
where
    I: Producer,
    F: Fn(I::Item) -> R + Sync,
{
    type Controlled = I::Controlled;
    fn should_be_divided(&self) -> bool {
        self.base.should_be_divided()
    }
    fn divide(self) -> (Self, Self) {
        let (left, right) = self.base.divide();
        (
            MapProducer {
                base: left,
                op: self.op,
            },
            MapProducer {
                base: right,
                op: self.op,
            },
        )
    }
    fn divide_at(self, index: usize) -> (Self, Self) {
        let (left, right) = self.base.divide_at(index);
        (
            MapProducer {
                base: left,
                op: self.op,
            },
            MapProducer {
                base: right,
                op: self.op,
            },
        )
    }
}

impl<'f, R, I, F> Producer for MapProducer<'f, I, F>
where
    I: Producer,
    F: Fn(I::Item) -> R + Sync,
{
    fn preview(&self, index: usize) -> Self::Item {
        (self.op)(self.base.preview(index))
    }
}

impl<R, I, F> PreviewableParallelIterator for Map<I, F>
where
    R: Send,
    I: PreviewableParallelIterator,
    F: Fn(I::Item) -> R + Sync + Send,
{
}

struct MapConsumer<C, F> {
    op: F,
    base: C,
}

impl<R, Item, F, C> Consumer<Item> for MapConsumer<C, F>
where
    F: Fn(Item) -> R + Send + Sync,
    C: Consumer<R>,
{
    type Result = C::Result;
    fn fold<I>(&self, iterator: I) -> Self::Result
    where
        I: Iterator<Item = Item>,
    {
        self.base.fold(iterator.map(&self.op))
    }
    fn reduce(&self, left: Self::Result, right: Self::Result) -> Self::Result {
        self.base.reduce(left, right)
    }
    fn consume_producer<P>(&self, producer: P) -> Self::Result
    where
        P: Producer<Item = Item>,
    {
        let map_producer = MapProducer {
            op: &self.op,
            base: producer,
        };
        self.base.consume_producer(map_producer)
    }
}
