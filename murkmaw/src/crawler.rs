use std::sync::Arc;
use tokio::sync::Mutex;
use reqwest;
use std::collections::HashSet;

#[derive(Debug)]
pub struct Crawler {
    visited_links: Arc<Mutex<HashSet<String>>>,
}

impl Crawler {
    /// Creates a new instance of the crawler with an empty set of visited links.
    pub fn new() -> Self {
        Crawler {
            visited_links: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Crawls a given URL with a maximum number of retries for network requests.
    pub async fn crawl(&self, url: &str, max_retries: usize) -> Result<(), Box<dyn std::error::Error>> {
        if !self.is_already_visited(url).await {
            self.add_to_visited(url).await;

            let mut retries = 0;
            loop {
                match reqwest::get(url).await {
                    Ok(response) if response.status().is_success() => {
                        let body = response.text().await?;
                        println!("Fetched URL: {}", url);
                        println!("Content: {}", &body[0..body.len().min(100)]); // Print the first 100 characters
                        break;
                    }
                    Err(e) if retries < max_retries => {
                        retries += 1;
                        eprintln!("Retrying {} (attempt {}/{}) due to error: {}", url, retries, max_retries, e);
                    }
                    Err(e) => {
                        eprintln!("Failed to fetch {} after {} retries: {}", url, retries, e);
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    /// Checks if a URL has already been visited.
    async fn is_already_visited(&self, url: &str) -> bool {
        let visited = self.visited_links.lock().await;
        visited.contains(url)
    }

    /// Marks a URL as visited by adding it to the set of visited links.
    async fn add_to_visited(&self, url: &str) {
        let mut visited = self.visited_links.lock().await;
        visited.insert(url.to_string());
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let crawler = Crawler::new();
    let urls = vec![
        "https://example.com",
        "https://www.rust-lang.org",
        "https://crates.io",
    ];

    let tasks: Vec<_> = urls.iter()
        .map(|&url| {
            let crawler = &crawler;
            tokio::spawn(async move {
                crawler.crawl(url, 3).await.unwrap_or_else(|err| {
                    eprintln!("Error crawling {}: {}", url, err);
                });
            })
        })
        .collect();

    for task in tasks {
        task.await?;
    }

    Ok(())
}
//Author: Morteza Farrokhnejad