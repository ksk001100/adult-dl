use super::Extractor;
use super::VideoInfo;
use async_trait::async_trait;
use regex::Regex;

#[derive(Debug)]
pub struct Fc2 {}

#[async_trait]
impl Extractor for Fc2 {
    async fn extract(&self, url: &str) -> Result<VideoInfo, Box<dyn std::error::Error>> {
        Ok(VideoInfo { url: String::new(), title: String::new() })
    }
}
