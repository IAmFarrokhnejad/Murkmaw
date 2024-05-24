use std::process;
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



fn crawl_element(elem: Element) -> Result<Vec<String>>
{
    if elem.name =="a"
    {
        let text = elem.children.iter().filter(|c| is_text(c)).last().map(|n| n.text()).ok_or_else(|| anyhow!("Failed to fetch text!")); //PROCEED HERE (THIS LINE NEEDS FIXING)
    }
    let link_elements = elem.children.iter().filter(|c| is_node(c));
    for Child in elem.children
    {
        match Child 
        {

            Node::Element(elem)=>{
                log::info!("Element found!: {} ", elem.name);
            },
            _ =>{},

        }
    }

    Ok(Vec::new())
}

async fn crawl_url(url: &str) -> Result<Vec<String>> 
{
    let html = reqwest::get(url)
    .await?
    .text()
    .await;

    let dom = Dom::parse(&html);

    for Child in dom.children
    {
        match Child 
        {
            Node::Text(text)=>{
                log::info!("Node found!: {} ", text);
            },
            Node::Element(elem)=>{
                log::info!("Element found!: {} ", elem.name);
            },
            Node::Comment(comment)=>{
                log::info!("omment found!: {} ", comment);
            }

        }
    }



    let res: Vec<String> = Vec::new();
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