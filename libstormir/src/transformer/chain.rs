use anyhow::Context;

use super::Transformer;

pub struct ChainTransformer<A, B> {
    a: A,
    b: B,
}

impl<A, B> Transformer for ChainTransformer<A, B>
where
    A: Transformer,
    B: Transformer<Input = A::Output>,
{
    type Input = A::Input;
    type Output = B::Output;

    fn transform(&mut self, input: Vec<Self::Input>) -> anyhow::Result<Vec<Self::Output>> {
        self.a
            .prepass(&input)
            .context("During prepass by transformer A of chain")?;
        let a_output = self
            .a
            .transform(input)
            .context("During transformation by transformer A of chain")?;

        self.b
            .prepass(&a_output)
            .context("During prepass by transformer B of chain")?;
        self.b
            .transform(a_output)
            .context("During transformation by transformer B of chain")
    }
}

pub trait TransformerChainExt {
    type This: Transformer;

    fn chain<O>(self, other: O) -> ChainTransformer<Self::This, O>;
}

impl<T> TransformerChainExt for T
where
    T: Transformer,
{
    type This = Self;

    fn chain<O>(self, other: O) -> ChainTransformer<Self::This, O> {
        ChainTransformer { a: self, b: other }
    }
}
