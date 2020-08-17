use super::Extractor;
use async_trait::async_trait;
use regex::Regex;

#[derive(Debug)]
pub struct Xvideos {}

#[async_trait]
impl Extractor for Xvideos {
    async fn extract(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        let re = Regex::new(r#"setVideoUrlHigh\('(.*?)'"#).unwrap();
        let resp = reqwest::get(url).await?;
        let body = resp.text().await?;
        let caps = re.captures(&body).unwrap();

        Ok(caps.get(1).unwrap().as_str().to_string())
    }
}
