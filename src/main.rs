#[macro_use] extern crate rocket;

mod objects {
    pub mod market{
        pub mod raw_market;
        pub mod market;
        pub mod helper;
    }
    pub mod database;
}

mod routes {
    pub mod apiv1;
}

use objects::database::*;
use objects::market::market::{Market, Item};

use dotenv::dotenv;
use rocket::{State, http::Status, get, routes, Rocket, Build, serde::json::Json};
use std::env;

use routes::apiv1::{
    latest_market
};

#[get("/")]
async fn index() -> &'static str {
    "<h1>Hello, world!</h1>"
}

#[launch]
async fn rocket() -> Rocket<Build> {
    dotenv().ok();
    let db = match VioDB::new(env::var("MONGO_URI").unwrap().as_str(), "Vio").await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Failed to create VioDB: {}", e);
            std::process::exit(1);
        }
    };

    rocket::build()
        .manage(db)
        .mount("/", routes![index, latest_market])
}