use extra::Extra;

pub mod chain;
pub mod extra;
pub mod repeat;
pub mod runner;

pub type PrepassFn<S> =
    fn(this: &mut S, input: &Extra<<S as Transformer>::Input>) -> anyhow::Result<()>;

pub trait Transformer: 'static {
    type Input;
    type Output;

    const PREPASSES: &[(&'static str, PrepassFn<Self>)] = &[];

    fn transform(&mut self, input: Extra<Self::Input>) -> anyhow::Result<Extra<Self::Output>>;
}
