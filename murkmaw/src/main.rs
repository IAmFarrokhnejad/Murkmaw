use std::{any, collections::{HashSet, VecDeque}, process, sync::{Arc, RwLock}};
use anyhow::{anyhow, Result, bail};
use clap::Parser;
use html_parser::{Dom, Element, Node};
use tokio::sync::RwLock;


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

fn crawl_element(elem: &Element, root_url: Url) -> Result<Vec<String>> 
{


    let mut links: Vec<String> = Vec::new();

    if elem.name == "a" 
    {
        let href_attrib = elem.attributes().get("href").ok_or_else(||anyhow!("Failed to find href from the link!"))?.as_ref().ok_or_else(||"Href does not have a value!")?.clone();

        links.push(get_url(&href_attrib, root_url.clone())?.to_string());
    }


    for node in elem.children().iter().filter(|c| is_node(c)) {
        match node {
            Node::Element(elem) => {
                let mut children_links = crawl_element(elem, root_url.clone());
                links.append(&mut children_links);
            },
            _ =>{}
        }
    }

    Ok(links)
}

async fn find_links(url: String) -> Result<Vec<String>> 
{   


    //Pare HTML into a DOM object
    let html = reqwest::get(url.clone())
    .await?
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

async fn crawl(crawler_state: crawlerStateRef) -> Result<()> {
    
    'crawler: loop {

        let mut link_queue = crawler_state.link_queue.write().await;
        let already_visited = crawler_state.already_visited.read().await;

        if link_queue.is_empty() || (already_visited.len() > crawler_state.max_links) {
            break 'crawler;
        }
        drop(already_visited);

        let url_str = link_queue.pop_back().ok_or_else(|| anyhow!("Queue is empty!"))?;
        drop(link_queue);

        let url = Url::parse(&url_str)?;
        let links = find_links(url.clone()).await?;

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

async fn try_main(args: programArgs) -> Result<()> 
{
    let mut crawler_state = crawlerState {
        link_queue: RwLock::new(VecDeque::from([args.starting_url])),
        already_visited: RwLock::new(Default::default()),
        max_links: 1000,
    };

    let crawler_state = Arc::new(crawlerState);

    crawl(crawler_state.clone()).await?;

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