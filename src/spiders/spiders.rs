use async_trait::async_trait;

#[async_trait]
pub trait Spider: Send + Sync + 'static {
    type Item: Send + 'static;

    fn name(&self) -> &str;

    async fn scrape(
        &self,
        url: String,
    ) -> Result<(Vec<Self::Item>, Vec<String>), Box<dyn std::error::Error + Send + Sync>>;

    async fn process(
        &self,
        item: Self::Item,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
