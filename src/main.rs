use clap::Parser;
use scraper::{Html, Selector};
use std::time::{Duration, Instant};
use url::Url;

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

    println!("Starting crawler at: {}", &args.url);

    let client = reqwest::Client::new();
    let html_content = fetch_html(&client, &args.url).await?;
    let mut visited_links: Vec<String> = Vec::new();
    let initial_links = extract_links(&html_content, &args.url, &visited_links)?;
    let mut unvisited_links: Vec<String> = initial_links;
    let mut unsuccessful_links: Vec<String> = Vec::new();

    let start_time = Instant::now();
    let timeout_duration = Duration::from_secs(args.seconds);

    while let Some(link) = unvisited_links.pop() {
        if start_time.elapsed() >= timeout_duration {
            println!("\nTime limit of {} seconds reached.", args.seconds);
            unvisited_links.push(link);
            break;
        }
        if !visited_links.contains(&link) {
            match fetch_html(&client, &link).await {
                Ok(new_html) => {
                    let new_links = extract_links(&new_html, &link, &visited_links)?;
                    unvisited_links.extend(new_links);
                    visited_links.push(link);
                }
                Err(e) => {
                    println!("Failed to fetch {}: {}", link, e);
                    unsuccessful_links.push(link);
                }
            }
        }
    }

    println!("\nUnsuccessful links: {}", unsuccessful_links.len());
    println!("\nUnvisited links: {}", unvisited_links.len());
    println!("\nVisited links: {}", visited_links.len());

    println!("\nvisited urls:");
    for (i, url) in visited_links.iter().enumerate() {
        println!("{}. {}", i + 1, url);
    }

    Ok(())
}

async fn fetch_html(
    client: &reqwest::Client,
    url: &str,
) -> std::result::Result<String, Box<dyn std::error::Error>> {
    println!("Fetching {}", url);
    let response = client.get(url).send().await?.text().await?;

    Ok(response)
}

fn extract_links(
    html_content: &str,
    base_url_str: &str,
    visited_links: &Vec<String>,
) -> std::result::Result<Vec<String>, Box<dyn std::error::Error>> {
    let document = Html::parse_document(html_content);
    let link_selector = Selector::parse("a[href]").unwrap();
    let mut links: Vec<String> = Vec::new();
    let base_url = Url::parse(base_url_str)?;

    for element in document.select(&link_selector) {
        if let Some(href) = element.value().attr("href") {
            match base_url.join(href) {
                Ok(absolute_url) => {
                    let url_string = absolute_url.to_string();
                    if !links.contains(&url_string) && !visited_links.contains(&url_string) {
                        links.push(url_string);
                    }
                }
                Err(_) => {
                    println!("Skipping invalid URL: {}", href);
                }
            }
        }
    }

    Ok(links)
}
