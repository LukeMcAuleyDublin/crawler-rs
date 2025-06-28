use clap::Parser;
// use std::time::{Duration, Instant};

mod crawler;
mod parser;
mod pg;
use crawler::crawler::Crawler;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    url: String,

    #[arg(short, long, default_value = "30")]
    seconds: u64,
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let initial_url = String::from(&args.url);

    match Crawler::new(initial_url, false).await {
        Ok(mut crawler) => {
            crawler.crawl().await;
        }
        Err(e) => {
            println!("ERROR! SAD! :( : {}", e);
        }
    }

    Ok(())
}
