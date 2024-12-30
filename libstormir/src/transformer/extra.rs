use std::{
    collections::HashMap,
    convert::Infallible,
    fmt::Debug,
    ops::{ControlFlow, Try},
};

use anyhow::Context;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};

pub const SUFFIX_MSGPACK: &'static str = ".msgpack";
pub const SUFFIX_RON: &'static str = ".ron";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Extras<T> {
    pub data: T,
    pub extras: HashMap<String, Vec<u8>>,
}

impl<T> Extras<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            extras: HashMap::new(),
        }
    }

    pub fn map_data<D, F>(self, map_fn: F) -> Extras<D>
    where
        F: FnOnce(T) -> D,
    {
        Extras {
            data: map_fn(self.data),
            extras: self.extras,
        }
    }

    // The try trait really is something...
    pub fn try_map_data<F, D, R, E>(self, map_fn: F) -> Result<Extras<D>, E>
    where
        F: FnOnce(T) -> R,
        R: Try<Output = D, Residual = Result<Infallible, E>>,
    {
        match map_fn(self.data).branch() {
            ControlFlow::Continue(new_data) => Ok(Extras {
                data: new_data,
                extras: self.extras,
            }),
            ControlFlow::Break(e) => Err(e.unwrap_err()),
        }
    }

    pub fn with_extra<E>(mut self, key: &str, extra: &E) -> Self
    where
        E: Serialize + Debug,
    {
        self.append_extra(key, extra);
        self
    }

    pub fn with_extra_raw(mut self, key: &str, extra_raw: Vec<u8>) -> Self {
        self.append_extra_raw(key, extra_raw);
        self
    }

    pub fn append_extra<E>(&mut self, key: &str, extra: &E)
    where
        E: Serialize + Debug,
    {
        let extra_rmp = rmp_serde::to_vec(&extra).expect("Failed to serialize extra to msgpack");

        // RON can serialize some things that JSON can't such as non-string map keys.
        let ron_config = PrettyConfig::new()
            .struct_names(true)
            .enumerate_arrays(true);
        let extra_ron = ron::ser::to_string_pretty(extra, ron_config)
            .expect("Failed to serialize extra to RON")
            .into_bytes();

        self.append_extra_raw(&[key, SUFFIX_MSGPACK].concat(), extra_rmp);
        self.append_extra_raw(&[key, SUFFIX_RON].concat(), extra_ron);
    }

    pub fn append_extra_raw(&mut self, key: &str, extra_raw: Vec<u8>) {
        if self.extras.contains_key(key) {
            panic!("Duplicate extra data on key '{}'", key);
        }

        self.extras.insert(key.to_owned(), extra_raw);
    }

    pub fn extra<'a, E>(&'a self, key: &str) -> Option<anyhow::Result<E>>
    where
        E: Deserialize<'a>,
    {
        let extra_msgpack = self.extras.get(&[key, SUFFIX_MSGPACK].concat())?;

        Some(
            rmp_serde::from_slice(extra_msgpack)
                .with_context(|| format!("Deserializing extra '{}' msgpack", key)),
        )
    }

    pub fn extra_raw(&self, key: &str) -> Option<&Vec<u8>> {
        self.extras.get(key)
    }
}
