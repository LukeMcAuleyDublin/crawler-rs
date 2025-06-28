use crate::crawler::services::AppServices;
use crate::parser::links::LinkCollection;

#[derive(Debug)]
pub struct Crawler {
    pub services: AppServices,
    pub link_collection: LinkCollection,
    pub single_site: bool,
}

impl Crawler {
    pub async fn new(
        initial_url: String,
        stay_on_domain: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let collection = LinkCollection::new(initial_url).unwrap();
        Ok(Self {
            services: AppServices::new().await?,
            link_collection: collection,
            single_site: stay_on_domain,
        })
    }
    pub async fn crawl(&mut self) {
        if self.single_site == true {
            self.crawl_single_site();
        } else {
            self.crawl_multiple_sites().await;
        }
    }

    async fn crawl_single_site(&self) {
        println!("Crawling single domain...")
    }

    async fn crawl_multiple_sites(&mut self) {
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
