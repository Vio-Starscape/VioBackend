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
    pub mod errors;
    pub mod ratelimiter;
    pub mod key;
}
use objects::database::*;

use log::info;

use dotenv::dotenv;
use rocket::catchers;
use rocket::{launch, Rocket, Build, config::Config, options, routes};
use rocket_okapi::{rapidoc::*, openapi_get_routes};
use rocket_cors::{AllowedOrigins, AllowedHeaders, CorsOptions};
use rocket_okapi::settings::UrlObject;
use std::env;
use std::sync::Mutex;

use routes::{apiv1, ratelimiter};
use routes::errors;

#[options("/v1/<_..>")]
fn options_v1_all() -> rocket::http::Status {
    println!("OPTIONS /v1/* preflight handled.");
    rocket::http::Status::Ok
}

#[launch]
async fn rocket() -> Rocket<Build> {
    dotenv().ok();
    env_logger::init();
    let db = match VioDB::new(
        env::var("MONGO_URI").unwrap().as_str(),
        env::var("DATABASE_NAME").unwrap().as_str(),
    ).await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Failed to create VioDB: {}", e);
            std::process::exit(1);
        }
    };

    info!("Connected to database");

    let cors = CorsOptions::default() 
        .allowed_origins(AllowedOrigins::all()) // Customize allowed origins if needed
        .allowed_headers(AllowedHeaders::some(&["Authorization", "Accept", "x-api-key", "Content-Type"])) // Add more headers as required
        .allow_credentials(false)
        .to_cors().unwrap();

    let figment = Config::figment()
        .merge(("address", env::var("HOST").unwrap().as_str()))
        .merge(("port", env::var("PORT").unwrap().as_str().parse::<u16>().unwrap()))
        .merge(("secret_key", env::var("SECRET_KEY").unwrap().as_str()));

    let settings = rocket_okapi::settings::OpenApiSettings::new();
    let routes = openapi_get_routes!{
        settings: apiv1::latest_market, apiv1::recent_market, apiv1::item_list, apiv1::item_history, apiv1::insert_data};

    rocket::custom(figment)
        .manage(db)
        .manage(ratelimiter::RequestCount {
            counts: Mutex::new(std::collections::HashMap::new())
        })
        .mount("/v1", routes)
        .mount("/v1", routes![options_v1_all])
        .mount("/",
        make_rapidoc(&RapiDocConfig { 
            general: GeneralConfig {
                spec_urls: vec![UrlObject::new("V1 (Initial API)", "../v1/openapi.json")],
                heading_text: "Vio API".to_string(),
                persist_auth: true,
                ..Default::default()
            },
            title: Some("Vio API".to_string()),
            ui: UiConfig {
                theme: Theme::Dark,
                ..Default::default()
            },
            hide_show: HideShowConfig {
                show_header: false,
                ..Default::default()
            },
            ..Default::default()  
        }))
        .register("/v1", catchers![errors::unauthorized, errors::not_found, errors::bad_request, errors::unprocessable_entity])
        .attach(cors)
}