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
        self.transformer.prepass(&input).context("During prepass")?;

        self.transformer
            .transform(input)
            .context("During transformation")
    }
}
