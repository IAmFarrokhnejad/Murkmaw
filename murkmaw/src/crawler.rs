use anyhow::{anyhow, bail, Result};
use log2::*;
use reqwest::{Client, StatusCode};
use scraper::{Html, Selector};
use std::{collections::VecDeque, sync::Arc, time::Duration};
use tokio::sync::RwLock;
use url::Url;

use crate::model::Image;
use crate::model::LinkGraph;

const LINK_REQUEST_TIMEOUT_S: u64 = 2;

/// Enum to represent data to scrape from
/// each link
pub enum ScrapeOption {
    Images,
    Titles, 
}


#[derive(Default)]
pub struct LinkPath {
    pub parent: String,
    pub child: String,
}

pub struct ScrapeOutput {
    pub links: Vec<String>,
    pub images: Vec<Image>,
    pub titles: Vec<String>,
}

pub struct CrawlerState {
    pub link_queue: RwLock<VecDeque<LinkPath>>,
    pub link_graph: RwLock<LinkGraph>,
    pub max_links: usize,
}

pub type CrawlerStateRef = Arc<CrawlerState>;

fn get_url(path: &str, root_url: Url) -> Result<Url> {
    if let Ok(url) = Url::parse(path) {
        return Ok(url);
    }

    root_url
        .join(path)
        .ok()
        .ok_or(anyhow!("could not join relative path"))
}

fn get_images(html_dom: &Html, root_url: &Url) -> Vec<Image> {
    let img_selector = Selector::parse("img[src]").unwrap();

    let image_links = html_dom
        .select(&img_selector)
        .filter(|e| e.value().attr("src").is_some())
        .map(|e| {
            (
                e.value().attr("src").unwrap(),
                e.value().attr("alt").unwrap_or(""),
            )
        })
        .map(|(link, alt)| Image {
            link: link.to_string(),
            alt: alt.to_string(),
        });

    let mut result: Vec<Image> = Default::default();
    for image in image_links {
        if let Ok(absolute_url) = get_url(&image.link, root_url.clone()) {
            result.push(Image {
                link: absolute_url.to_string(),
                ..image
            });
            continue;
        }

        error!("failed to join url");
    }

    result
}

fn get_titles(html_dom: &Html) -> Vec<String> {
    let mut titles: Vec<String> = Default::default();

    for tag in ["h1", "h2", "title"] {
        let title_selector = Selector::parse(tag).unwrap();

        titles.extend(
            html_dom
                .select(&title_selector)
                .map(|e| e.text().collect::<String>()),
        );
    }

    titles
}

async fn scrape_page_helper(
    url: Url,
    client: &Client,
    options: &[ScrapeOption],
) -> Result<ScrapeOutput> {
    let response = client
        .get(url.clone())
        .timeout(Duration::from_secs(LINK_REQUEST_TIMEOUT_S))
        .send()
        .await?;

    if response.status() != StatusCode::OK {
        bail!("page returned invalid response");
    }

    let html = response.text().await?;

    let html_dom = scraper::Html::parse_document(&html);

    let link_selector = Selector::parse("a").unwrap();
    let links: Vec<String> = html_dom
        .select(&link_selector)
        .filter_map(|e| e.value().attr("href"))
        .map(|href| href.to_string())
        .collect();

    let mut images: Vec<Image> = Vec::new();
    let mut titles: Vec<String> = Vec::new();
    for option in options {
        match option {
            ScrapeOption::Images => {
                images = get_images(&html_dom, &url);
            }
            ScrapeOption::Titles => {
                titles = get_titles(&html_dom);
            }
        }
    }

    Ok(ScrapeOutput {
        links,
        images,
        titles,
    })
}


pub async fn scrape_page(url: Url, client: &Client, options: &[ScrapeOption]) -> ScrapeOutput {
    let mut scrape_output = match scrape_page_helper(url.clone(), client, options).await {
        Ok(output) => output,
        Err(e) => {
            error!("Could not find links: {}", e);
            ScrapeOutput {
                images: Default::default(),
                links: Default::default(),
                titles: Default::default(),
            }
        }
    };

    scrape_output.links = scrape_output
        .links
        .iter()
        .filter_map(|l| get_url(l, url.clone()).ok())
        .map(|url| url.to_string())
        .collect();

    scrape_output
}
//Author: Morteza Farrokhnejad