use super::{extra::Extras, runner::TransformerRunnerExt, Transformer};

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

    fn transform(&mut self, input: Extras<Self::Input>) -> anyhow::Result<Extras<Self::Output>> {
        let a_output = self.a.runner().run_with_extras(input)?;
        self.b.runner().run_with_extras(a_output)
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
