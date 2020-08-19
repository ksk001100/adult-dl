mod downloader;
mod extractor;

use bytesize::ByteSize;
use downloader::Downloader;
use extractor::select_extractor;
use seahorse::{color, App, Context, Flag, FlagType};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let app = App::new(env!("CARGO_PKG_NAME"))
        .description(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .usage("adult-dl [url] [option]")
        .action(action)
        .flag(
            Flag::new("output", FlagType::String)
                .usage("--output, -o [name]: Specifying the output filename")
                .alias("o"),
        );

    app.run(args);
}

fn action(c: &Context) {
    match tokio::runtime::Runtime::new() {
        Ok(mut rt) => rt.block_on(async {
            let url = if c.args.len() >= 1 {
                &c.args[0]
            } else {
                eprintln!("{}\n", color::red("Specify URL..."));
                c.help();
                std::process::exit(1);
            };

            let ext = select_extractor(url).await.unwrap();
            let videoinfo = ext.extract(url).await.unwrap();

            println!("\n[URL] : {}", url);
            println!("[TITLE] : {}", videoinfo.title);
            println!("[EXTRACT URL] : {}", videoinfo.url);
            println!("[FILE SIZE] : {}\n", ByteSize::b(videoinfo.size as u64));

            let downloader = Downloader::new(videoinfo);
            downloader.download().await.unwrap();
        }),
        Err(e) => eprintln!("{}", color::red(e)),
    }
}
