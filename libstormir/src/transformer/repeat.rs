use anyhow::Context;

use super::Transformer;

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

    fn transform(&mut self, input: Vec<Self::Input>) -> anyhow::Result<Vec<Self::Output>> {
        let mut accumulator = input;

        for repeat in 0..self.times {
            self.inner
                .prepass(&accumulator)
                .context(format!("During prepass {}", repeat))?;
            accumulator = self
                .inner
                .transform(accumulator)
                .context(format!("During transformation {}", repeat))?;
        }

        Ok(accumulator)
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
