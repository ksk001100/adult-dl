use super::Extractor;
use super::VideoInfo;
use async_trait::async_trait;
use regex::Regex;

#[derive(Debug)]
pub struct Pornhub {}

#[async_trait]
impl Extractor for Pornhub {
    async fn extract(&self, url: &str) -> Result<VideoInfo, Box<dyn std::error::Error>> {
        Ok(VideoInfo {
            url: String::new(),
            title: String::new(),
            size: 0,
            filename: String::new(),
        })
    }
}
