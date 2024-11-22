use anyhow::{anyhow, Context, Result};
use clap::Parser;
use log2::*;
use logger::spinner::Colour;
use model::LinkGraph;
use rayon::prelude::*;
use reqwest::Client;
use std::{process, sync::Arc, time::Duration};
use tokio::{fs, task::JoinSet};
use url::Url;

mod crawler;
mod image_utils;
mod logger;
mod model;
use crate::{
    crawler::Crawler,
    image_utils::{convert_links_to_images, download_images},
};
//Author: Morteza Farrokhnejad

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
    #[arg(long, default_value_t = 3)]
    max_retries: usize,
}

async fn output_status(crawler: &Crawler, total_links: u64) -> Result<()> {
    let progress_bar = logger::progress_bar::ProgressBar::new(total_links);
    progress_bar.message("Finding links");
    'output: loop {
        let visited_links = crawler.get_visited_count().await;
        if visited_links >= total_links {
            info!("All links found");
            break 'output;
        }

        progress_bar.set_step(visited_links as u64);
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    Ok(())
}

async fn try_main(args: ProgramArgs) -> Result<()> {
    let crawler = Crawler::new();
    crawler.add_to_queue(args.starting_url.clone()).await;

    let mut tasks = JoinSet::new();
    for _ in 0..args.n_worker_threads {
        let crawler = crawler.clone();
        tasks.spawn(async move {
            crawler.run(args.max_retries).await.unwrap_or_else(|err| {
                error!("Crawler task error: {:?}", err);
            });
        });
    }

    if args.log_status {
        let crawler = crawler.clone();
        tasks.spawn(async move {
            output_status(&crawler, args.max_links).await.unwrap_or_else(|err| {
                error!("Status tracking error: {:?}", err);
            });
        });
    }

    while let Some(result) = tasks.join_next().await {
        if let Err(e) = result {
            error!("Error: {:?}", e);
        }
    }

    let spinner = logger::spinner::Spinner::new();
    spinner.status("[1/4] converting image links");
    let image_metadata = convert_links_to_images(&crawler.get_graph().await);
    spinner.print_above("  [1/4] converted image links", Colour::Green);

    spinner.status("[2/4] downloading image metadata");
    download_images(&image_metadata, &args.img_save_dir, args.max_images).await?;
    spinner.print_above("  [2/4] downloaded image metadata", Colour::Green);

    spinner.status("[3/4] creating image database");
    let image_database = serde_json::to_string(&image_metadata)?;
    fs::write(args.img_save_dir + "database.json", image_database).await?;
    spinner.print_above("  [3/4] created image database", Colour::Green);

    spinner.status(format!("[4/4] serializing links to {}", args.links_json));
    let links = crawler.get_graph().await;
    let json = serde_json::to_string(&links)?;
    fs::write(&args.links_json, json).await?;
    spinner.print_above(
        format!("  [4/4] serialized links to {}", args.links_json),
        Colour::Green,
    );

    Ok(())
}

fn main() {
    let _log2 = log2::open("log.txt");
    let args = ProgramArgs::parse();
    pretty_print_args(&args);

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(args.n_worker_threads as usize)
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(async {
        if let Err(e) = try_main(args).await {
            error!("Error: {:?}", e);
            process::exit(-1);
        }
    });

    info!("Finished!");
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
    println!("  {} {}", console::Emoji("🔄", ""), console::style(&args.max_retries).bold().cyan());
    println!();
}
