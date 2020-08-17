use super::Extractor;
use async_trait::async_trait;
use regex::Regex;

#[derive(Debug)]
pub struct Fc2 {}

#[async_trait]
impl Extractor for Fc2 {
    async fn extract(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        Ok(url.to_string())
    }
}
