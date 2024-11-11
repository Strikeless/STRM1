use anyhow::Context;

use super::{extra::Extra, Transformer};

pub struct TransformerRunner<'a, T>
where
    T: Transformer,
{
    transformer: &'a mut T,
}

impl<'a, T> TransformerRunner<'a, T>
where
    T: Transformer,
{
    pub fn new(transformer: &'a mut T) -> Self {
        Self { transformer }
    }

    pub fn run(&mut self, input: T::Input) -> anyhow::Result<Extra<T::Output>> {
        self.run_with_extras(Extra::new(input))
    }

    pub fn run_with_extras(&mut self, input: Extra<T::Input>) -> anyhow::Result<Extra<T::Output>> {
        for (prepass_name, prepass_fn) in T::PREPASSES {
            prepass_fn(&mut self.transformer, &input)
                .with_context(|| format!("During prepass '{}'", prepass_name))?;
        }

        self.transformer
            .transform(input)
            .context("During transformation")
    }
}

pub trait TransformerRunnerExt: Transformer + Sized {
    fn runner(&mut self) -> TransformerRunner<Self>;
}

impl<T> TransformerRunnerExt for T
where
    T: Transformer,
{
    fn runner(&mut self) -> TransformerRunner<Self> {
        TransformerRunner::new(self)
    }
}
