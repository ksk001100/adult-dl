use super::Extractor;
use super::VideoInfo;
use async_trait::async_trait;
use regex::Regex;
use reqwest::{header, Client, Url};

#[derive(Debug)]
pub struct Xvideos {}

#[async_trait]
impl Extractor for Xvideos {
    async fn extract(&self, url: &str) -> Result<VideoInfo, Box<dyn std::error::Error>> {
        let client = Client::new();
        let re_url = Regex::new(r#"setVideoUrlHigh\('(.*?)'"#).unwrap();
        let re_title = Regex::new(r"<title>(.*?)</title>").unwrap();
        let resp = client.get(url).send().await?;
        let headers = resp.headers().clone();
        let html = resp.text().await?;
        let url = re_url
            .captures(&html)
            .unwrap()
            .get(1)
            .unwrap()
            .as_str()
            .to_string();
        let title = re_title
            .captures(&html)
            .unwrap()
            .get(1)
            .unwrap()
            .as_str()
            .to_string();

        let resp = client.head(&url).send().await?;
        let size = match resp.headers().get(header::CONTENT_LENGTH) {
            Some(s) => s.to_str().unwrap().parse().unwrap(),
            None => 0,
        };

        let filename = match headers.get(header::CONTENT_DISPOSITION) {
            Some(name) => name.to_str().unwrap().to_string(),
            None => {
                let parsed = Url::parse(&url).unwrap();
                parsed.path().split("/").last().unwrap().to_string()
            }
        };

        Ok(VideoInfo {
            url,
            title,
            size,
            filename,
        })
    }
}
