use std::process::{self, Child};
use anyhow::{anyhow, Ok, Result};
use html_parser::{Dom, Element, Node};





fn is_node(node: &Node) -> bool
{
    match node {
        Node::Element(..) => true,
        _ => false
        
    }
}


fn is_text(node: &Node) -> bool
{
    match node {
        Node::Text(_)=>true,
        _ =>false
        
    }
}


fn crawl_element(elem: Element) -> Result<Vec<String>> {


    let mut links: Vec<String> = Vec::new();

    if elem.name == "a" {
        let text = elem.children.iter()
            .filter(|c| is_text(c))
            .last()
            .map(|n| n.text())
            .ok_or_else(|| anyhow!("Failed to fetch the text!"))?
            .text()
            .ok_or_else(|| anyhow!("Failed to fetch the value!"))?;

        links.push(text.to_string());
    }


    for nodes in elem.children.iter().filter(|c| is_node(c))
    {

        match node {
            Node::Element(elem) =>
            {
                //add any link from this element to our vector
                let mut children_links = crawl_element(elem.clone())?;
                links.append(&mut children_links);
            },
            _=>{}
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
                log::info!("Links found for element {}: {:?}", elem.name, crawl_element(elem));
            },
            _ => {}
        }

       log::info!("Links found for element {}: {:?}", Child.element().map_or("undefined", |n| &n.name) , {crawl_element(Child)});
    }


    //Change This later!!                         
    let res: Vec<String> = Vec::new();                         //PROCEED HERE!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
    Ok(res)
}




async fn try_main() -> Result<()> 
{

    let _ = crawl_url("https://google.com").await?;

    println!("{:?}", resp.text().await);


    Ok(())
}


#[tokio::main]
async fn main()
{
    env_logger::init();

    match try_main().await
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