pub trait LogResult<T> {
    fn with_log(self) -> Self;
}

impl<T> LogResult<anyhow::Result<T>> for anyhow::Result<T> {
    fn with_log(self) -> Self {
        self
    }
}
