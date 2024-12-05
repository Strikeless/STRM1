use std::{
    collections::HashMap,
    convert::Infallible,
    ops::{ControlFlow, Try},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Extra<T> {
    pub data: T,

    // TODO: Make extras less painful to use by automating or eliminating manual (de)serialization.
    pub extra: HashMap<String, Vec<u8>>,
}

impl<T> Extra<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            extra: HashMap::new(),
        }
    }

    pub fn map_data<D, F>(self, map_fn: F) -> Extra<D>
    where
        F: FnOnce(T) -> D,
    {
        Extra {
            data: map_fn(self.data),
            extra: self.extra,
        }
    }

    // The try trait really is something...
    pub fn try_map_data<F, D, R, E>(self, map_fn: F) -> Result<Extra<D>, E>
    where
        F: FnOnce(T) -> R,
        R: Try<Output = D, Residual = Result<Infallible, E>>,
    {
        match map_fn(self.data).branch() {
            ControlFlow::Continue(new_data) => Ok(Extra {
                data: new_data,
                extra: self.extra,
            }),
            ControlFlow::Break(e) => Err(e.unwrap_err()),
        }
    }

    pub fn with_extra<I>(mut self, key: &str, data: I) -> Self
    where
        I: IntoIterator<Item = u8>,
    {
        self.append_extra(key, data);
        self
    }

    pub fn append_extra<I>(&mut self, key: &str, data: I)
    where
        I: IntoIterator<Item = u8>,
    {
        if self.extra.contains_key(key) {
            panic!("Duplicate extra data on key '{}'", key);
        }

        self.extra
            .insert(key.to_string(), data.into_iter().collect());
    }
}
