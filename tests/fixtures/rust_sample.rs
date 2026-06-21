macro_rules! make_client {
    () => {};
}

#[derive(Debug)]
pub struct Client {
    value: String,
}

pub enum Mode {
    Fast,
    Slow,
}

pub trait Service {
    fn call(&self);
}

impl Client {
    pub async fn build(value: String) -> Self {
        make_client!();
        Self { value }
    }
}

mod tests {
    #[tokio::test(flavor = "current_thread")]
    async fn builds_client() {}
}
