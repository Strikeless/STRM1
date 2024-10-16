pub mod chain;
pub mod repeat;

pub trait Transformer {
    type Input;
    type Output;

    #[allow(unused_variables)] // It's a trait Rust why are you complaining about me not using input in this default implementation?
    fn prepass(&mut self, input: &Vec<Self::Input>) -> anyhow::Result<()> {
        Ok(())
    }

    fn transform(&mut self, input: Vec<Self::Input>) -> anyhow::Result<Vec<Self::Output>>;
}
