use tokio::sync::RwLock;
use std::{ collections::{ VecDeque, HashSet }, sync::Arc, time::Duration};
use anyhow::Result;
use url::Url;


const LINK_REQUEST_TIMEOUT_S u64 = 2;
pub struct crawlerState {
    pub link_queue: RwLock<VecDeque<String>>,
    pub already_visited: RwLock<HashSet<String>>,
    pub max_links: u32,
}

pub type crawlerStateRef = Arc<crawlerState>;

async fn find_links(url: Url, client: &Client) -> Vec<String> 
{   
    match request_html(url.clone(), &client).await {
        Ok(html_doc) => {
            return crawl_recursively(&html_doc.children, url);
        },
        Err(e) => {
            log::error!("{}", e);
            Vec::new()
        }
    }
}

pub async fn crawl(crawler_state: crawlerStateRef, worker_n: i32) -> Result<()> {
    let client = Client::new();


    'crawler: loop {

        let mut link_queue = crawler_state.link_queue.write().await;
        let url_str = link_queue.pop_back().unwrap_or("".to_string());
        drop(link_queue);

        if url_str.is_empty() {
            log::info!("Waiting for the next link from {}", worker_n)
            tokio::time::sleep(Duration::from_millis(500)).await;
            continue;
        }

        log::info!("Finding links for: {}", &url_str);
        let url = Url::parse(&url_str)?;

        if (already_visited.len() > crawler_state.max_links) {
            break 'crawler;
        }

        let links = find_links(url, &client).await;

        let mut link_queue = crawler_state.link_queue.write().await?;
        let mut already_visited = crawler_state.already_visited.write().await;

        for link in links {
            if !already_visited.contains(&link) {
                link_queue.push_back(link)
            }
        }

        already_visited.insert(url_str);
    }

    Ok(())
}