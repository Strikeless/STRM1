use std::{
    error::Error,
    io::{self, Write},
    str::FromStr,
};

use anyhow::anyhow;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CommandError {
    // Not thrown directly by the arg reader
    #[error("Unknown command")]
    UnknownCommand,

    #[error("Missing argument {0}")]
    MissingArgument(usize),

    #[error("Bad argument ({0})")]
    ParseError(String),
}

pub struct Command(String);

impl Command {
    pub fn prompt() -> anyhow::Result<Self> {
        print!("> ");
        io::stdout().flush()?;

        let line = io::stdin()
            .lines()
            .next()
            .ok_or_else(|| anyhow!("End of input"))? // This shouldn't really be an error, maybe change at some point.
            .map_err(|e| anyhow!("Couldn't read command from stdin: {}", e))?;

        Ok(Self(line))
    }

    pub fn args(&self) -> CommandArgs<impl Iterator<Item = &str>> {
        CommandArgs {
            iter: self.0.split_whitespace(),
            index: 0,
        }
    }
}

pub struct CommandArgs<'a, I>
where
    I: Iterator<Item = &'a str>,
{
    iter: I,
    index: usize,
}

impl<'a, I> CommandArgs<'a, I>
where
    I: Iterator<Item = &'a str>,
{
    pub fn next(&mut self) -> Result<&str, CommandError> {
        self.index += 1;

        self.iter
            .next()
            .ok_or_else(|| CommandError::MissingArgument(self.index))
    }

    pub fn next_parsed<T>(&mut self) -> Result<Result<T, CommandError>, CommandError>
    where
        T: FromStr,
        <T as FromStr>::Err: Error + 'static,
    {
        let arg_str = self.next()?;

        Ok(T::from_str(arg_str).map_err(|e| CommandError::ParseError(e.to_string())))
    }
}
