mod downloader;
mod extractor;

use downloader::Downloader;
use extractor::select_extractor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://www.xvideos.com/video57088597/_7_";
    let ext = select_extractor(url).await?;
    let video_url = ext.extract(url).await?;

    println!("{}", video_url);

    let downloader = Downloader::new(video_url).await?;
    downloader.download().await?;
    Ok(())
}
