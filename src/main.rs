mod objects {
    pub mod raw_market;
    pub mod database;
}

use objects::database::*;
use objects::raw_market::Market;

use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() -> mongodb::error::Result<()> {
    dotenv().ok();

    let db = VioDB::new(env::var("MONGO_URI").unwrap().as_str(), "Vio").await?;

    println!("Started");

    let market = db.get_market().await?;

    println!("Stopped");

    // println!("Latest Market Instance: {:?}", market);

    Ok(())
}