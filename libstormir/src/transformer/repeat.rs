use anyhow::Context;

use super::{runner::TransformerRunner, Transformer};

pub struct RepeatTransformer<T> {
    inner: T,
    times: usize,
}

impl<T, D> Transformer for RepeatTransformer<T>
where
    T: Transformer<Input = D, Output = D>,
{
    type Input = T::Input;
    type Output = T::Output;

    fn transform(&mut self, input: Self::Input) -> anyhow::Result<Self::Output> {
        (0..self.times).try_fold(input, |accumulator, repeat| {
            TransformerRunner::new(&mut self.inner)
                .run(accumulator)
                .with_context(|| format!("On repeat {}", repeat))
        })
    }
}

pub trait TransformerRepeatExt {
    type This: Transformer;

    fn repeat(self, times: usize) -> RepeatTransformer<Self::This>;
}

impl<T> TransformerRepeatExt for T
where
    T: Transformer,
{
    type This = T;

    fn repeat(self, times: usize) -> RepeatTransformer<Self::This> {
        RepeatTransformer { inner: self, times }
    }
}
