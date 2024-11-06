use std::{any, collections::{HashSet, VecDeque}, process, sync::{Arc, RwLock}, time::Duration};
use anyhow::{anyhow, Result, bail};
use clap::Parser;
use html_parser::{Dom, Element, Node};
use tokio::sync::RwLock;
use futures::{stream::FutureUnordered, Future, StreamExt};
use url::Url;
use html_parser::{Dom, Node, Element};


/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct programArgs {
    /// Name of the person to greet
    #[arg(short, long)]
    starting_url: String,
}

struct crawlerState {
    link_queue: RwLock<VecDeque<String>>,
    already_visited: RwLock<HashSet<String>>,
    max_links: u32,
}

type crawlerStateRef = Arc<crawlerState>;

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

//CREATE THIS FUNCTION LATER
//fn request_html(url: Url) ->Dom

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
    let elements = children.iter().filter_map(|e| crawl_element(e, root_url.clone()).ok());

    links.flatten().collect()
}


fn crawl_element(elem: &Element, root_url: Url) -> <Vec<String>> 
{

    let mut link: Option<String> = None;

    if elem.name == "a" 
    {
        if let Ok(href_attrib) = get_href(&elem) {
            link = get_url(&href_attrib, root_url.clone()).ok().map(|url| url.to_sting());   
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

async fn find_links(url: Url, client: &Client) -> Vec<String> 
{   

    log::info!("Finding links in: {}", url.as_str());


    let response = client.get(url.clone()).timeout(Duration::from_millis(500)).send().await?;

    if response.status() != StatusCode::OK {
        log::error!("Invalid response from the page");
        return Vec::new();
    }
    //Pare HTML into a DOM object
    let html = response
    .text()
    .await;

    let dom = Dom::parse(&html);
    let mut res: Vec<String> = Vec::new();


    //Crawls all the nodes in main html
    for Child in dom.children
    {
        match child {
            Node::Element(elem) =>{
                let links = match crawl_element(&elem, url.clone()) {
                    Ok(links) => links,
                    Err(e) => {
                        log::error!("Error: {}", e);
                         Vec::new() 
                    }
                }
                for link in links {
                    res.push(link.clone());
                    log::info!("Link found in {}: {:?}", url, link);
                }
          
            },
            _ => {}
        }

       log::info!("Links found for element {}: {:?}", Child.element().map_or("undefined", |n| &n.name) , {crawl_element(Child)});
    }
    //Change This later!!                         

    Ok(res)
}

async fn crawl(crawler_state: crawlerStateRef, worker_n: i32) -> Result<()> {
    let client = Client::new();


    'crawler: loop {

        let mut link_queue = crawler_state.link_queue.write().await;
        let already_visited = crawler_state.already_visited.read().await;

        if link_queue.is_empty() {
            log::info!("Waiting for the next link from {}", worker_n)
            tokio::time::sleep(Duration::from_millis(500)).await;
            continue;
        }

        if (already_visited.len() > crawler_state.max_links) {
            break 'crawler;
        }
        

        let url_str = link_queue.pop_back().ok_or_else(|| anyhow!("Queue is empty!"))?;
        drop(link_queue);
        drop(already_visited);

        let url = Url::parse(&url_str)?;
        let links = find_links(url, &client).await?;

        let mut link_queue = crawler_state.link_queue.write().await;
        let mut already_visited = crawler_state.already_visited.write().await;

        for link in links {
            if already_visited.contains(&link) {
                link_queue.push_back(link);
            }

        }

        //Store all the visited links
        
        already_visited.insert(url_str);

        Ok(())

    }

    println!("{:?}", already_visited);
    Ok(())
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