use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use futures::stream::StreamExt;
use mongodb::{
    bson::{doc, Document},
    Cursor,
    Database,
    Client,
    Collection
};

use std::sync::Arc;
use super::raw_market::Market;
use super::raw_market::RobloxUser;


#[derive(Debug, Serialize, Deserialize)]
pub struct Count{
    _id: u32,
    pub count: u32
}
pub struct VioDB {
    uri: String,
    client: Client,
    pub db: Database,
}

impl VioDB {
    pub async fn new(uri: &str, database: &str) -> mongodb::error::Result<VioDB> {
        let client: Client = Client::with_uri_str(uri).await?;
        let db: Database = client.database(database);
        Ok(VioDB {
            uri: uri.to_string(),
            client,
            db,
        })
    }

    pub async fn get_roblox_users(&self) -> mongodb::error::Result<HashMap<u64, RobloxUser>> {
        let roblox: Collection<RobloxUser> = self.db.collection("Roblox");
        let mut cursor: Cursor<RobloxUser> = roblox.find(doc! {}, None).await?;
        let mut users: HashMap<u64, RobloxUser> = HashMap::new();
        while let Some(user) = cursor.next().await {
            let current_user: RobloxUser = user.unwrap();
            users.insert(current_user.id, current_user);
        }
        Ok(users)
    }

    pub async fn insert_users_into_market_data(&self, market: &mut Document, roblox_users: Option<&HashMap<u64, RobloxUser>>) -> mongodb::error::Result<()> {
        

        // If roblox_users is none then get from db
        let roblox_users: HashMap<u64, RobloxUser> = match roblox_users {
            Some(u) => u.clone(),
            None => self.get_roblox_users().await?
        };

        if let Some(items) = market.get_document_mut("items").ok() {
            for (_, item) in items.iter_mut(){
                let item_doc = item.as_document_mut().unwrap();
                if let Ok(buy) = item_doc.get_array_mut("buy") {
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

        Ok(())
    }

    /// **These** Functions are too dangerous and take too long. Will not use

    // pub async fn get_raw_market(&self) -> mongodb::error::Result<Vec<Market>> {
    //     let collection: Collection<Document> = self.db.collection("Market");
    //     let mut cursor: Cursor<Document> = collection.find(doc! {"_id": {"$gt": 0}}, None).await?;
    //     let mut markets: Vec<Market> = Vec::new();

    //     while let Some(docu) = cursor.next().await {
    //         let market: Market = mongodb::bson::from_document(docu?)?;
    //         markets.push(market);
    //     }
        
    //     Ok(markets)
    // }

    // pub async fn get_market(&self) -> mongodb::error::Result<Vec<Document>> {
    //     let collection: Collection<Document> = self.db.collection("Market");
    //     let mut cursor: Cursor<Document> = collection.find(doc! {"_id": {"$gt": 0}}, None).await?;
    //     let mut markets: Vec<Document> = Vec::new();

    //     let roblox_users = self.get_roblox_users().await?;

    //     let mut documents: Vec<Document> = Vec::new();
    //     while let Some(docu) = cursor.next().await {
    //         documents.push(docu?);
    //     }

    //     for mut market in documents {
    //         self.insert_users_into_market_data(&mut market, Some(&roblox_users)).await?;
    //         markets.push(market);
    //     }

    //     Ok(markets)
    // }

    pub async fn get_market_count(&self) -> mongodb::error::Result<Option<Count>> {
        let collection: Collection<Document> = self.db.collection("Market");
        let market_count: Option<Document> = collection.find_one(doc! {"_id": 0}, None).await?;
        if let Some(docu) = market_count {
            let count: Count = mongodb::bson::from_document(docu)?;
            Ok(Some(count))
        } else {
            Ok(None)
        }
    }

    pub async fn get_latest_raw_instance(&self) -> mongodb::error::Result<Option<Market>> {
        let count: Option<Count> = self.get_market_count().await?;
        if let Some(c) = count {
            let last_document: Option<Document> = self.db.collection("Market").find_one(doc! {"_id": c.count}, None).await?;
            let latest_instance: Market = mongodb::bson::from_document(last_document.unwrap())?;
            Ok(Some(latest_instance))
        } else {
            Ok(None)
        }
    }

    pub async fn get_latest_instance(&self) -> mongodb::error::Result<Option<Document>> {
        let count: Option<Count> = self.get_market_count().await?;
        if let Some(c) = count {
            let last_document = self.db.collection("Market").find_one(doc! {"_id": c.count}, None).await?;
            let mut market = last_document.unwrap();
            self.insert_users_into_market_data(&mut market,None).await?;
            Ok(Some(market.clone()))
        } else {
            Ok(None)
        }
    }
}