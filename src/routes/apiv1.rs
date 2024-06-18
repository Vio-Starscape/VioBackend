use rocket::{
    get,
    http::Status,
    post,
    serde::json::Json,
    State
};

use log::info;
use std::env;
use rocket_okapi::openapi;
use crate::objects::{database::VioDB, market::{market::{Item, MixedMarket}, raw_market::RawMarket}};
use crate::objects::market::market::Market;

use crate::routes::key::ApiKey;


/// # Latest Market (Keyword Only)
///
/// Get the latest market data for a list of items.
/// This difference between this and the recent market is that this will look for the last instance of an Item.
/// It should always return something if the item is in the database.
/// 
/// Provide a list of items separated by commas.
///
/// If no items are provided, will 404.
#[openapi]
#[get("/market/latest?<items>")]
pub async fn latest_market(_key: ApiKey, db: &State<VioDB>, items: Option<&str>) -> Result<Json<MixedMarket>, Status> {
    info!("Getting latest market data for items: {:?} | by: {}", &items, _key.value());
    match items {
        Some(items) => {
            let item = items.split(",").map(|name| name.to_string()).collect();
            let market = db.get_latest_instance(&item).await;
            match market.ok() {
                Some(market) => Ok(Json(market)),
                None => Err(Status::NotFound)
            }
        },
        None => Err(Status::BadRequest)
    }
}

/// # Recent Market
///
/// Get the most recent market data.
///
/// If no items are provided, will return the entire most recent scan.
/// else will return only the requested items in the most recent scan.
/// 
/// Provide a list of items separated by commas.
/// 
/// Sometimes the scans miss a bunch of items, so only use this if you plan on storing the data yourself aswell.
/// 
#[openapi]
#[get("/market/recent?<items>")]
pub async fn recent_market(_key:ApiKey, db: &State<VioDB>, items: Option<&str>) -> Result<Json<Market>, Status> {
    info!("Getting recent market data for items: {:?} | by: {}", &items, _key.value());
    match items {
        Some(items) => {
            let item = items.split(",").map(|name| name.to_string()).collect();
            let market = db.get_recent_instance_filter(&item).await.ok();
            match market.unwrap() {
                Some(market) => Ok(Json(market)),
                None => Err(Status::NotFound)
            }
        },
        None => {
            let market = db.get_recent_instance().await.ok().unwrap();
            match market {
                Some(market) => Ok(Json(market)),
                None => Err(Status::NotFound)
            }
        }
    }
}

/// # Item List
///
/// Get a list of all items that can be requested.
#[openapi]
#[get("/market/items")]
pub async fn item_list(_key: ApiKey, db: &State<VioDB>) -> Json<Vec<String>> {
    info!("Getting item list | by: {}", _key.value());
    let items = db.get_item_list().await.ok().unwrap();
    Json(items)
}

/// # Item History
/// 
/// Get the history of an item.
/// This function returns a lot of data, and is not recommended to be used for large amounts of items.
/// If you call this function, you should most likely cache the data as it won't ever change other than new items.
/// 
#[openapi]
#[get("/market/history/<item>")]
pub async fn item_history(_key: ApiKey, db: &State<VioDB>, item: &str) -> Result<Json<Vec<Item>>, Status> {
    info!("Getting market data history for item: {:?} | by: {}", &item, _key.value());
    let market = db.get_market_for_item(item.to_string()).await.ok().unwrap();
    Ok(Json(market))
}


// Websocket and Dataendpoint

// type SharedState = std::sync::Arc<tokio::sync::Mutex<HashMap<String, WebSocket>>>;


#[openapi(skip)]
#[post("/market/insert/data", format="json", data="<data>")]
pub async fn insert_data(_key: ApiKey, db: &State<VioDB>, data: Json<RawMarket>) -> Result<Json<String>, Status> {
    info!("Inserting data | by: {}", _key.value());
    if _key.value() != env::var("ADMIN_KEY").unwrap() {
        return Err(Status::Unauthorized);
    }
    db.add_market_to_database(data.0).await.ok().unwrap();
    info!("Data Inserted");
    Ok(Json("Data Inserted".to_string()))
}


// #[openapi()]
// #[get("/market/ws")]
// pub async fn ws_endpoint(_key: ApiKey, ws: WebSocket) -> Channel<'static> {
//     info!("Getting websocket endpoint | by: {}", _key.0);
//     use rocket::futures::{SinkExt, StreamExt};

//     ws.channel(move |mut stream| Box::pin(async move{
//         while let Some(msg) = stream.next().await {
//             let _ = stream.send(msg?).await;
//         }

//         Ok(())
//     }))
// }