use anyhow::{anyhow, Context, Result};
use clap::Parser;
use log2::*;
use logger::spinner::Colour;
use model::LinkGraph;
use rayon::prelude::*;
use reqwest::Client;
use std::{collections::VecDeque, process, sync::Arc, time::Duration};
use tokio::{fs, sync::RwLock, task::JoinSet};
use url::Url;

mod crawler;
mod image_utils;
mod logger;
mod model;
use crate::{
    crawler::{scrape_page, CrawlerState, LinkPath, ScrapeOption},
    image_utils::{convert_links_to_images, download_images},
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct ProgramArgs {
    #[arg(short, long)]
    starting_url: String,
    #[arg(long, default_value_t = 100)]
    max_links: u64,
    #[arg(long, default_value_t = 100)]
    max_images: u64,
    #[arg(short, long, default_value_t = 4)]
    n_worker_threads: u64,
    #[arg(short, long, default_value_t = false)]
    log_status: bool,
    #[arg(short, long, default_value_t = String::from("images/"))]
    img_save_dir: String,
    #[arg(long, default_value_t = String::from("links.json"))]
    links_json: String,
}

async fn output_status(crawler_state: &CrawlerState, total_links: u64) -> Result<()> {
    let progress_bar = logger::progress_bar::ProgressBar::new(total_links);
    progress_bar.message("Finding links");
    'output: loop {
        if crawler_state.link_graph.read().await.len() > crawler_state.max_links {
            info!("All links found");
            break 'output;
        }

        progress_bar.set_step(crawler_state.link_graph.read().await.len() as u64);
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    Ok(())
}
//Author: Morteza Farrokhnejad

async fn crawl(crawler_state: &CrawlerState) -> Result<()> {
    let client = Client::new();

    'crawler: loop {
        if crawler_state.link_graph.read().await.len() > crawler_state.max_links {
            break 'crawler;
        }

        let LinkPath { parent, child } = crawler_state
            .link_queue
            .write()
            .await
            .pop_back()
            .unwrap_or(Default::default());
        let scrape_options = vec![ScrapeOption::Images, ScrapeOption::Titles];
        let scrape_output = scrape_page(Url::parse(&child)?, &client, &scrape_options).await;

        crawler_state
            .link_graph
            .write()
            .await
            .update(&child, &parent, &scrape_output.links, &scrape_output.images, &scrape_output.titles)
            .context("failed to update link graph")?;
    }

    Ok(())
}

async fn serialize_links(links: &LinkGraph, destination: &str) -> Result<()> {
    let json = serde_json::to_string(links)?;
    fs::write(destination, json).await?;
    Ok(())
}

fn new_crawler_state(starting_url: String, max_links: u64) -> Arc<CrawlerState> {
    Arc::new(CrawlerState {
        link_queue: RwLock::new(VecDeque::from([LinkPath {
            child: starting_url,
            ..Default::default()
        }])),
        link_graph: RwLock::new(Default::default()),
        max_links: max_links as usize,
    })
}

async fn try_main(args: ProgramArgs) -> Result<()> {
    let crawler_state = new_crawler_state(args.starting_url, args.max_links);

    let mut tasks = JoinSet::new();
    for _ in 0..args.n_worker_threads {
        let crawler_state = crawler_state.clone();
        tasks.spawn(tokio::spawn(async move { crawl(&crawler_state).await }));
    }

    if args.log_status {
        let crawler_state = crawler_state.clone();
        tasks.spawn(tokio::spawn(async move {
            output_status(&crawler_state, args.max_links).await
        }));
    }

    while let Some(result) = tasks.join_next().await {
        if let Err(e) = result {
            error!("Error: {:?}", e);
        }
    }

    let link_graph = crawler_state.link_graph.read().await;
    let spinner = logger::spinner::Spinner::new();

    spinner.status("[1/4] converting image links");
    let image_metadata = convert_links_to_images(&link_graph);
    spinner.print_above("  [1/4] converted image links", Colour::Green);

    spinner.status("[2/4] downloading image metadata");
    download_images(&image_metadata, &args.img_save_dir, args.max_images).await?;
    spinner.print_above("  [2/4] downloaded image metadata", Colour::Green);

    spinner.status("[3/4] creating image database");
    let image_database = serde_json::to_string(&image_metadata)?;
    fs::write(args.img_save_dir + "database.json", image_database).await?;
    spinner.print_above("  [3/4] created image database", Colour::Green);

    spinner.status(format!("[4/4] serializing links to {}", args.links_json));
    serialize_links(&link_graph, &args.links_json).await?;
    spinner.print_above(
        format!("  [4/4] serializing links to {}", args.links_json),
        Colour::Green,
    );

    Ok(())
}

fn main() {
    let _log2 = log2::open("log.txt");
    let args = ProgramArgs::parse();
    pretty_print_args(&args);

    match try_main(args).await {
        Ok(_) => {
            info!("Finished!");
        }
        Err(e) => {
            error!("Error: {:?}", e);
            process::exit(-1);
        }
    }
}

fn pretty_print_args(args: &ProgramArgs) {
    println!("{}", console::style("CRAWLER INPUT ARGUMENTS").white().on_black());
    println!("  {} {}", console::Emoji("🌐", ""), console::style(&args.starting_url).bold().cyan());
    println!("  {} {}", console::Emoji("🔗", ""), console::style(&args.max_links).bold().cyan());
    println!("  {} {}", console::Emoji("🖼️", ""), console::style(&args.max_images).bold().cyan());
    println!("  {} {}", console::Emoji("⚒️", ""), console::style(&args.n_worker_threads).bold().cyan());
    println!("  {} {}", console::Emoji("❔", ""), console::style(args.log_status).bold().cyan());
    println!("  {} {}", console::Emoji("📁", ""), console::style(&args.img_save_dir).bold().cyan());
    println!("  {} {}", console::Emoji("📁", ""), console::style(&args.links_json).bold().cyan());
    println!();
}