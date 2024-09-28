pub mod chain;
pub mod repeat;

pub trait Transformer {
    type Input;
    type Output;

    fn prepass(&mut self, input: &Vec<Self::Input>) -> anyhow::Result<()> {
        Ok(())
    }

    fn transform(&mut self, input: Vec<Self::Input>) -> anyhow::Result<Vec<Self::Output>>;
}
