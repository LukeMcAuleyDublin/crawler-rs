use clap::{ArgAction, Parser};
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

    #[arg(short, long, action = ArgAction::SetTrue)]
    restrict_domain: bool,

    #[arg(short, long, default_value = "30")]
    seconds: u64,
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let initial_url = String::from(&args.url);
    let restrict_domain = args.restrict_domain;

    match Crawler::new(initial_url, restrict_domain).await {
        Ok(mut crawler) => {
            crawler.crawl().await;
        }
        Err(e) => {
            println!("ERROR! SAD! :( : {}", e);
        }
    }

    Ok(())
}
