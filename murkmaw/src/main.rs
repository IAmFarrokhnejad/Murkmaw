use std::process;

use anyhow::Result;






async fn try_main() -> Result<()> 
{


    let resp = reqwest::get("https://google.com")
    .await?;


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