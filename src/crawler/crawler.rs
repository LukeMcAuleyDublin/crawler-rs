use crate::crawler::services::AppServices;
use crate::parser::links::LinkCollection;

#[derive(Debug)]
pub struct Crawler {
    pub services: AppServices,
    pub link_collection: LinkCollection,
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
        })
    }
    pub async fn crawl(&mut self) {
        match self
            .link_collection
            .crawl(&self.services.http, &self.services.db)
            .await
        {
            Ok(_) => {
                println!("Successfully crawled.");
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}
