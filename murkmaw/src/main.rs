use std::{any, collections::{HashSet, VecDeque}, process, sync::{Arc, RwLock}, time::Duration};
use anyhow::{anyhow, Result, bail};
use clap::Parser;
use html_parser::{Dom, Element, Node};
use tokio::sync::RwLock;
use futures::{stream::FutureUnordered, Future, StreamExt};
use url::Url;
use html_parser::{Dom, Node, Element};
mod crawler;


/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct programArgs {
    /// Name of the person to greet
    #[arg(short, long)]
    starting_url: String,
}

fn main() {
    let args = Args::parse();

    for _ in 0..args.count {
        println!("Hello {}!", args.name);
    }
}

//Author: Morteza Farrokhnejad


fn get_href(elem: &Element) -> Result<String> {
    elem.attributes().get("href").ok_or_else(||anyhow!("Failed to find href from the link!"))?.as_ref().ok_or_else(||"Href does not have a value!").cloned()
}


async fn request_html(url: Url, client: &Client) ->Result<Dom> {
    let response = client.get(url.clone()).timeout(Duration::from_secs(LINK_REQUEST_TIMEOUT_S)).send().await?;

    if response.status() != StatusCode::OK {
        bail!("Page returned invalid response!");
    }

    let html = response.text().await?;
    Ok(Dom::parse(&html)?);
}



//Turns URLs into full URLs
fn get_url(path: &str, root_url: Url) -> Result <Url> {

    if Ok(url) = Url::parse(&path) {
        return Ok(url);
    }

    root_url.join(&path).ok().ok_or(anyhow!("Failed to join the relative path!"))

    match Url::parse(&path) {
        Ok(url) => Ok(url),
        _ => {
            match root_url.join(path) {
                Ok(url) => Ok(url),
                _ => bail!("Failed to join the relative path!")
            }
        }
    },
}

fn is_node(node: &Node) -> bool
{
    match node {
        Node::Element(..) => true,
        _ => false
        
    }
}


fn crawl_recursively(children: &[Node], root_url: Url) -> Result<Vec<String>> {
    let elements = children.iter().filter_map(|e| crawl_element(e, root_url.clone()));

    let links = elements.map(|e| crawl_element(e, root_url.clone()));

    links.flatten().collect()
}


fn crawl_element(elem: &Element, root_url: Url) -> <Vec<String>> 
{

    let mut link: Option<String> = None;

    if elem.name == "a" 
    {
        if let Ok(href_attrib) = get_href(&elem) {
            link = get_url(&href_attrib, root_url.clone()).ok().map(|url| url.to_string());   
        } else {
            log::error!("Failed to locate the 'href' in the HTML tag!")
        }

       
    }

    let mut children_links = crawl_recursively(&elem.children, root_url);

    if let Some(link) = link {
        children_links.push(link);
    }

    children_links
}



async fn output_status(crawlerState: crawlerStateRef) -> Result<()> {
    loop {
        let already_visited = crawler_state.already_visited.read().await;
        log::info!("Number of links visited: {}", already_visited.len());

        for link in already_visited.iter() {
            log::info!("Already Visited: {}", link);
        }

        drop(already_visited);

        tokio::time::sleep(Duration::from_secs(3)).await;
    }

    Ok(())
}

async fn try_main(args: programArgs) -> Result<()> 
{
    let mut crawler_state = crawlerState {
        link_queue: RwLock::new(VecDeque::from([args.starting_url])),
        already_visited: RwLock::new(Default::default()),
        max_links: 1000,
    };

    let crawler_state = Arc::new(crawlerState);


    let mut tasks = FutureUnordered::<Pin<Box<dyn Future<Output = Result<()>>>>>::new();
    tasks.push(crawl(Box::pin(crawler_state.clone(), 1)));
    tasks.push(crawl(Box::pin(crawler_state.clone(), 2)));
    tasks.push(crawl(Box::pin(crawler_state.clone(), 3)));
    tasks.push(crawl(Box::pin(crawler_state.clone(), 4)));
    tasks.push(crawl(Box::pin(crawler_state.clone())));

    crawl(crawler_state.clone()).await?;

    while let Some(result) = tasks.next().await? {
        match result {
            Err(e) => {
                log::error!("Error: {:?}", e);
            },

            _ => ()
        }     
    }

    let already_visited = crawler_state.already_visited.read().await?;
    println!("{:?}", already_visited);
    Ok(())
}

#[tokio::main]
async fn main()
{
    env_logger::init();

    let args = programArgs::parse();

    match try_main(args).await
    {
        Ok(_) => {
            log::info!("Done!");
        },
        Err(e) =>{
            log::error!("An error occured: {:?}", e);
            process::exit(-1);
        }     
    }
}