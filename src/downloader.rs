use crate::extractor::VideoInfo;
use futures::{stream, StreamExt};
use num_cpus;
use reqwest::header;
use reqwest::Client;
use seahorse::color;
use std::io::Read;
use std::io::Write;
use std::sync::Arc;
use tokio::prelude::*;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct Downloader {
    client: Client,
    url: String,
    filename: String,
    temp_size: usize,
    content_length: usize,
    downloaded_count: Arc<Mutex<usize>>,
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
    pub async fn new(videoinfo: VideoInfo) -> Result<Self, Box<dyn std::error::Error>> {
        let s = Self {
            client: Client::new(),
            url: videoinfo.url.to_owned(),
            filename: videoinfo.filename.to_owned(),
            temp_size: 300000,
            content_length: videoinfo.size,
            downloaded_count: Arc::new(Mutex::new(1)),
        };

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

    pub async fn download(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !std::path::Path::new("temps").exists() {
            tokio::fs::create_dir("temps").await?;
        }

        let partial_range = self.range_headers().await?;
        let count = partial_range.len();

        stream::iter(partial_range)
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

                let mut lock = self.downloaded_count.lock().await;
                while let Some(b) = resp.next().await {
                    file.write(&b.unwrap()).await.unwrap();

                    let per = (*lock as f64 / count as f64) * 100.0;
                    let progress = "=".repeat(per as usize);
                    let whitespace = " ".repeat(100 - (per as usize));
                    print!("\r[{}>{}] : {:.1}%", progress, whitespace, per);
                }
                *lock += 1;
            })
            .buffer_unordered(num_cpus::get())
            .for_each(|_| async {})
            .await;

        let mut file = std::fs::File::create(&self.filename).unwrap();

        for i in 0..count {
            let mut buf: Vec<u8> = Vec::new();
            let mut temp_file =
                std::io::BufReader::new(std::fs::File::open(format!("temps/{}.tmp", i)).unwrap());
            temp_file.read_to_end(&mut buf).unwrap();

            file.write(&buf).unwrap();
        }

        tokio::fs::remove_dir_all("temps").await?;

        println!("\n\n\t{}", color::green("==========================="));
        println!(
            "\t{}  {}  {}",
            color::green("||"),
            color::yellow("Download Complete!!"),
            color::green("||")
        );
        println!("\t{}\n", color::green("==========================="));

        // println!("[File size] : {}", std::fs::metadata(&self.filename).unwrap().len());
        Ok(())
    }
}
