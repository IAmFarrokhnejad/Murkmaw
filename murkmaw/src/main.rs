use std::process::{self, Child};
use anyhow::{anyhow, Ok, Result};
use clap::Parser;
use html_parser::{Dom, Element, Node};


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


//Turns URLs into full URLs
fn get_url(url: &str, root_url: &str) -> String {
    

    log::info!("Comparing {} and {}", url, root_url);
    if url.starts_with("https:") || url.starts_with("http:") {
        return url.into();
    }

    log::info!("Formatting string ");

    format!("{}/{}", root_url.strip_suffix('/').unwrap_or(root_url), url.strip_prefix('/').unwrap_or(url)); 
}


fn is_node(node: &Node) -> bool
{
    match node {
        Node::Element(..) => true,
        _ => false
        
    }
}




fn crawl_element(elem: &Element, root_url: &str) -> Result<Vec<String>> 
{


    let mut links: Vec<String> = Vec::new();

    if elem.name == "a" 
    {
        let href_attrib = elem.attributes().get("href").ok_or_else(||anyhow!("Failed to find href from the link!"))?.as_ref().ok_or_else(||"Href does not have a value!")?.clone();

        links.push(get_url(&href_attrib, root_url));
    }


    for node in elem.children().iter().filter(|c| is_node(c)) {
        match node {
            Node::Element(elem) => {
                let mut children_links = crawl_element(elem, root_url);
                links.append(&mut children_links);
            },
            _ =>{}
        }
    }
    

    Ok(links)
}

async fn crawl_url(url: &str) -> Result<Vec<String>> 
{   

    //Pare HTML into a DOM object
    let html = reqwest::get(url)
    .await?
    .text()
    .await;

    let dom = Dom::parse(&html);


    //Crawls all the nodes in main html
    for Child in dom.children
    {
        match child {
            Node::Element(elem) =>{

                for link in crawl_element(elem, url)? {
                    log::info!("Link found in {}: {:?}", url, link);
                }
          
            },
            _ => {}
        }

       log::info!("Links found for element {}: {:?}", Child.element().map_or("undefined", |n| &n.name) , {crawl_element(Child)});
    }


    //Change This later!!                         
    let res: Vec<String> = Vec::new();
    Ok(res)
}




async fn try_main(args: programArgs) -> Result<()> 
{

    let _ = crawl_url(&args.starting_url).await?;

    // for url in urls {
    //     crawl_url(url);        
    // }


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