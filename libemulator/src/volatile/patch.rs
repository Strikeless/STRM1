#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VolatilePatch<T> {
    pub new_value: T,
}
