use clap::Parser;
use std::time::{Duration, Instant};

mod parser;
mod pg;

use parser::links::LinkCollection;

use pg::conn::establish_connection;

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
    let initial_url: String = String::from(&args.url);
    let timeout_duration = Duration::from_secs(args.seconds);
    let start_time = Instant::now();

    let pool = establish_connection().await?;
    let client = reqwest::Client::new();
    let mut link_collection = LinkCollection {
        db_conn: pool,
        http_client: client,
        visited_links: vec![],
        unvisited_links: vec![],
        start_point_url: initial_url,
    };

    match link_collection.crawl().await {
        Ok(_) => return Ok(()),
        Err(e) => {
            println!("Error! {}", e);
        }
    }

    Ok(())
}
