use std::collections::HashMap;
use futures::stream::StreamExt;

use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};
use rocket::State;

use mongodb::{
    bson::{doc, Document, DateTime},
    Database,
    Cursor,
    Collection
};

use super::market::RobloxUser;

pub struct ApiKey(String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKey {
    type Error = rocket::http::Status;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<'r, Self> {
        let db: &State<Database> = request.guard::<State<Database>>().await.expect("Database connection not found");
        match request.headers().get_one("x-api-key") {
            Some(key) => {
                // Query your database to check if the API key exists
                let api_key_exists = db.collection("api_keys").find_one(doc! { "key": key }, None).await.is_ok();
                if api_key_exists {
                    Outcome::Success(ApiKey(key.to_string()))
                } else {
                    Err(Status::Forbidden)
                }
            },
            None => Err(Status::BadRequest),
        }
    }
}

pub async fn insert_users_into_market_data(mut market: Document, roblox_users: &HashMap<u64, RobloxUser>) -> mongodb::error::Result<Document> {
    // If roblox_users is none then get from db

    let id= match market.get_i32("_id") {
        Ok(id) => id as u32,
        Err(_) => match market.get_i64("_id") {
            Ok(id) => id as u32,
            Err(_) => 0
        }
    };

    let time = match market.get_datetime("time_scanned"){
        Ok(time) => *time,
        Err(_) => DateTime::now(),
    };

    if let Some(items) = market.get_document_mut("items").ok() {
        for (_, item) in items.iter_mut(){
            let item_doc = item.as_document_mut().unwrap();
            item_doc.insert("_id", id);
            item_doc.insert("time_scanned", time.clone());
            if let Ok(buy) = item_doc.get_array_mut("buy") {
                for listing in buy {
                    let listing_array = listing.as_array_mut().unwrap();
                    
                    // The vendor_id is a 32 bit integer in some cases and 64 bit in others
                    let vendor_id = match listing_array[2].as_i64() {
                        Some(id) => id as u64,
                        None => match listing_array[2].as_i32() {
                            Some(id) => id as u64,
                            None => 0
                        }
                    };

                    // If the user is in the roblox_users hashmap then replace the id with the user
                    if let Some(user) = roblox_users.get(&vendor_id) {
                        listing_array[2] = mongodb::bson::to_bson(&user)?.into();
                    }
                }
            }
            if let Ok(buy) = item_doc.get_array_mut("sell") {
                for listing in buy {
                    let listing_array = listing.as_array_mut().unwrap();
                    // let vendor_id = listing_array[2].as_i64().unwrap() as u64;
                    let vendor_id = match listing_array[2].as_i64() {
                        Some(id) => id as u64,
                        None => match listing_array[2].as_i32() {
                            Some(id) => id as u64,
                            None => 0
                        }
                    };
                    if let Some(user) = roblox_users.get(&vendor_id) {
                        listing_array[2] = mongodb::bson::to_bson(&user)?.into();
                    }
                }
            }
        }
    }

    Ok(market)
}