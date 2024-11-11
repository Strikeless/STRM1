use anyhow::Context;

use super::{extra::Extra, runner::TransformerRunnerExt, Transformer};

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

    fn transform(&mut self, input: Extra<Self::Input>) -> anyhow::Result<Extra<Self::Output>> {
        (0..self.times).try_fold(input, |accumulator, repeat| {
            self.inner
                .runner()
                .run_with_extras(accumulator)
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
