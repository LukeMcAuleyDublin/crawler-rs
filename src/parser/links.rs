use scraper::{Html, Selector};
use sqlx::PgPool;
use url::Url;

#[derive(Debug)]
pub struct LinkCollection {
    pub visited_links: Vec<Link>,
    pub unvisited_links: Vec<Link>,
}

impl LinkCollection {
    pub fn new(start_point_url: String) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            visited_links: Vec::new(),
            unvisited_links: vec![Link::new(start_point_url)],
        })
    }
    pub async fn crawl(
        &mut self,
        client: &reqwest::Client,
        db_conn: &PgPool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        while let Some(mut link) = self.unvisited_links.pop() {
            println!("Crawling {}", &link.address);
            match link.extract_links(client, &self.visited_links).await {
                Ok(extracted_links) => {
                    for (_i, url) in extracted_links.iter().enumerate() {
                        self.add_to_unvisited_links(Link {
                            address: String::from(url),
                            visited: false,
                        });
                    }
                    self.add_to_visited_links(link.clone());
                    self.save_link(link, db_conn).await;
                }
                Err(e) => {
                    println!("Error while extracting links: {:?}", e);
                }
            }
        }
        Ok(())
    }
    pub fn add_to_visited_links(&mut self, mut link: Link) {
        link.visited = true;
        self.visited_links.push(link);
    }

    pub fn add_to_unvisited_links(&mut self, link: Link) {
        self.unvisited_links.push(link)
    }

    pub async fn save_link(&self, link: Link, db_conn: &PgPool) {
        match link.save(db_conn).await {
            Ok(_) => {
                println!("Successfully saved: {}", &link.address);
            }
            Err(e) => {
                println!("Error while saving: {}\nError: {}", &link.address, e);
            }
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct Link {
    pub address: String,
    pub visited: bool,
}

impl Link {
    pub fn new(address: String) -> Self {
        Self {
            address: address,
            visited: false,
        }
    }
    async fn fetch_html(
        &self,
        client: &reqwest::Client,
    ) -> std::result::Result<String, Box<dyn std::error::Error>> {
        println!("Fetching {}", &self.address);
        let response = client.get(&self.address).send().await?.text().await?;

        Ok(response)
    }

    pub async fn extract_links(
        &mut self,
        client: &reqwest::Client,
        visited_links: &Vec<Link>,
    ) -> std::result::Result<Vec<String>, Box<dyn std::error::Error>> {
        let html_content: String = self.fetch_html(client).await?;
        let document = Html::parse_document(&html_content);
        let link_selector = Selector::parse("a[href]").unwrap();
        let mut links: Vec<String> = Vec::new();
        let base_url = Url::parse(&self.address)?;

        for element in document.select(&link_selector) {
            if let Some(href) = element.value().attr("href") {
                match base_url.join(href) {
                    Ok(absolute_url) => {
                        let url_string = absolute_url.to_string();
                        if !links.contains(&url_string)
                            && !visited_links.iter().any(|link| link.address == url_string)
                        {
                            if self.valid_https(&url_string) {
                                links.push(url_string);
                            }
                        }
                    }
                    Err(_) => {
                        println!("Skipping invalid URL: {}", href);
                    }
                }
            }
        }

        self.visited = true;

        Ok(links)
    }

    pub fn valid_https(&self, url_string: &str) -> bool {
        match Url::parse(url_string) {
            Ok(url) => {
                if url.scheme() != "https" {
                    return false;
                }

                match url.host_str() {
                    Some(host) => host.contains('.'),
                    None => false,
                }
            }
            Err(_) => false,
        }
    }

    pub async fn save(&self, db_conn: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO urls (address) VALUES ($1)")
            .bind(&self.address)
            .execute(db_conn)
            .await?;

        Ok(())
    }
}
