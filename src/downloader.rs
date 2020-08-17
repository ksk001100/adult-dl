use futures::{stream, StreamExt};
use reqwest::header;
use reqwest::Client;
use reqwest::Url;
use tokio::prelude::*;
use std::io::Read;
use std::io::Write;

#[derive(Debug)]
pub struct Downloader {
    client: Client,
    url: String,
    title: Option<String>,
    filename: String,
    temp_size: usize,
    content_length: usize,
}

#[derive(Debug)]
pub struct PartialRange {
    index: usize,
    range: String,
}

impl PartialRange {
    pub fn new(index: usize, range: String) -> Self {
        Self { index, range }
    }
}

impl Downloader {
    pub async fn new(url: String) -> Result<Self, Box<dyn std::error::Error>> {
        let mut s = Self {
            client: Client::new(),
            url,
            title: None,
            filename: String::new(),
            temp_size: 300000,
            content_length: 0,
        };

        s.set_meta_data().await?;
        Ok(s)
    }

    async fn range_headers(&self) -> Result<Vec<PartialRange>, Box<dyn std::error::Error>> {
        let content_length = self.content_length;
        let split_num = content_length / self.temp_size;
        let ranges: Vec<usize> = (0..split_num)
            .map(|n| (content_length + n) / split_num)
            .collect();

        Ok((&ranges)
            .into_iter()
            .enumerate()
            .map(|(index, x)| {
                let s = match index {
                    0 => 0,
                    _ => (&ranges[..index]).iter().fold(0, |sum, y| sum + *y) + 1,
                };
                let e = (&ranges[..index]).iter().fold(0, |sum, y| sum + *y) + *x;
                let range = format!("bytes={}-{}", s, e);
                PartialRange::new(index, range)
            })
            .collect())
    }

    async fn set_meta_data(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let res = self.client.head(&self.url).send().await?;
        let length: usize = res
            .headers()
            .get(header::CONTENT_LENGTH)
            .unwrap()
            .to_str()
            .unwrap()
            .parse()
            .unwrap();
        let filename = match res.headers().get(header::CONTENT_DISPOSITION) {
            Some(name) => name.to_str().unwrap().to_string(),
            None => {
                let parsed = Url::parse(&self.url).unwrap();
                parsed.path().split("/").last().unwrap().to_string()
            }
        };

        self.content_length = length;
        self.filename = filename;

        Ok(())
    }

    pub async fn download(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !std::path::Path::new("temps").exists() {
            tokio::fs::create_dir("temps").await?;
        }

        let partial_range = self.range_headers().await?;
        let count = partial_range.len();

        let bodies = stream::iter(partial_range)
            .map(|r| async move {
                let mut resp = self
                    .client
                    .get(&self.url)
                    .header(header::RANGE, r.range)
                    .send()
                    .await
                    .unwrap()
                    .bytes_stream();

                let mut file = tokio::fs::File::create(format!("temps/{}.tmp", r.index))
                    .await
                    .unwrap();
                while let Some(b) = resp.next().await {
                    file.write(&b.unwrap()).await.unwrap();
                }
            })
            .buffer_unordered(10)
            .for_each(|_| async {})
            .await;

        let mut file = std::fs::File::create(self.filename.clone()).unwrap();

        for i in 0..count {
            let mut buf: Vec<u8> = Vec::new();
            let mut temp_file = std::io::BufReader::new(std::fs::File::open(format!("temps/{}.tmp", i)).unwrap());
            temp_file.read_to_end(&mut buf).unwrap();

            file.write(&buf).unwrap();
        }

        tokio::fs::remove_dir_all("temps").await?;

        Ok(())
    }
}
