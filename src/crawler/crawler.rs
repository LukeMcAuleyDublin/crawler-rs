use crate::crawler::services::AppServices;
use crate::logger::logger::{Logger, Message};
use crate::parser::links::LinkCollection;

use crossterm::style::Color;

#[derive(Debug)]
pub struct Crawler {
    pub services: AppServices,
    pub link_collection: LinkCollection,
    pub logger: Logger,
}

impl Crawler {
    pub async fn new(
        initial_url: String,
        stay_on_domain: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let collection = LinkCollection::new(initial_url, stay_on_domain).unwrap();
        Ok(Self {
            services: AppServices::new().await?,
            link_collection: collection,
            logger: Logger::new(
                String::from("crawler.crawler"),
                Color::Rgb {
                    r: 0,
                    g: 47,
                    b: 167,
                },
            ),
        })
    }
    pub async fn crawl(&mut self) -> std::io::Result<()> {
        match self
            .link_collection
            .crawl(&self.services.http, &self.services.db)
            .await
        {
            Ok(_) => self.logger.log(Message {
                text: String::from("Successfully Executed"),
                color: Color::Cyan,
            }),
            Err(e) => self.logger.log(Message {
                text: format!("Unsuccessfully Executed: {}", e),
                color: Color::Red,
            }),
        }
    }
}
