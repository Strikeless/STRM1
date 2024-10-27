use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Extra<T> {
    pub data: T,
    pub extra: HashMap<String, Vec<u8>>,
}

impl<T> Extra<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            extra: HashMap::new(),
        }
    }

    pub fn new_preserve_extras<D>(self, data: D) -> Extra<D> {
        Extra {
            data,
            extra: self.extra,
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
