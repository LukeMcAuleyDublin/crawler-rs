use scraper::{Html, Selector};
use sqlx::PgPool;
use url::Url;

use crate::logger::logger::{Logger, Message};
use crossterm::style::Color;

#[derive(Debug)]
pub struct LinkCollection {
    pub visited_links: Vec<Link>,
    pub unvisited_links: Vec<Link>,
    pub restrict_domain: bool,
    pub logger: Logger,
}

impl LinkCollection {
    pub fn new(
        start_point_url: String,
        restrict_domain: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            visited_links: Vec::new(),
            unvisited_links: vec![Link::new(start_point_url)],
            restrict_domain,
            logger: Logger::new(String::from("crawler.link_collection"), Color::Blue),
        })
    }
    pub async fn crawl(
        &mut self,
        client: &reqwest::Client,
        db_conn: &PgPool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        while let Some(mut link) = self.unvisited_links.pop() {
            self.logger.log(Message {
                text: format!("Crawling {}", &link.address),
                color: Color::DarkGreen,
            })?;
            match link.extract_links(client, &self.visited_links).await {
                Ok(extracted_links) => {
                    for (_i, url) in extracted_links.iter().enumerate() {
                        match self.restrict_domain {
                            true => {
                                if self.url_in_domain(&link.address, url).unwrap() {
                                    self.add_to_unvisited_links(Link::new(String::from(url)));
                                }
                            }
                            _ => {
                                self.add_to_unvisited_links(Link::new(String::from(url)));
                            }
                        }
                    }
                    self.add_to_visited_links(link.clone());
                    self.save_link(link, db_conn).await?;
                }
                Err(e) => {
                    self.logger.log(Message {
                        text: format!("Error while extracting links: {:?}", e),
                        color: Color::Red,
                    })?;
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

    pub async fn save_link(
        &self,
        link: Link,
        db_conn: &PgPool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match link.save(db_conn).await {
            Ok(_) => {
                self.logger.log(Message {
                    text: format!("Successfully saved: {}", &link.address),
                    color: Color::Green,
                })?;
            }
            Err(e) => {
                self.logger.log(Message {
                    text: format!("Error while saving: {}\nError: {}", &link.address, e),
                    color: Color::DarkRed,
                })?;
            }
        }

        Ok(())
    }

    pub fn url_in_domain(&self, link: &str, new_link: &str) -> Result<bool, url::ParseError> {
        let parsed_link = Url::parse(link).unwrap();
        let parsed_new_link = Url::parse(new_link).unwrap();

        Ok(parsed_link.domain() == parsed_new_link.domain())
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct Link {
    pub address: String,
    pub visited: bool,
    pub logger: Logger,
}

impl Link {
    pub fn new(address: String) -> Self {
        Self {
            address,
            visited: false,
            logger: Logger::new(String::from("crawler.link"), Color::DarkBlue),
        }
    }
    async fn fetch_html(
        &self,
        client: &reqwest::Client,
    ) -> std::result::Result<String, Box<dyn std::error::Error>> {
        self.logger.log(Message {
            text: format!("Fetching {}", &self.address),
            color: Color::DarkCyan,
        })?;
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
                    Err(_) => self.logger.log(Message {
                        text: format!("Skipping invalid URL: {}", href),
                        color: Color::DarkRed,
                    })?,
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
        match sqlx::query("INSERT INTO urls (address) VALUES ($1)")
            .bind(&self.address)
            .execute(db_conn)
            .await
        {
            Ok(_) => self.logger.log(Message {
                text: String::from("Successfully added row to DB"),
                color: Color::DarkGreen,
            })?,
            Err(e) => self.logger.log(Message {
                text: format!("Error when writing to DB: {}", e),
                color: Color::DarkRed,
            })?,
        }

        Ok(())
    }
}
