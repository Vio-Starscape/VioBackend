use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::sync::{Arc, Mutex};
use futures::stream::StreamExt;
use mongodb::{
    bson::{doc, Document},
    options::FindOptions,
    options::FindOneOptions,
    Cursor,
    Database,
    Client,
    Collection
};

use super::market::market::{RobloxUser, Market, MixedMarket, Item};
use super::market::helper::insert_users_into_market_data;

#[derive(Debug, Serialize, Deserialize)]
pub struct Count{
    _id: u32,
    pub count: u32
}

#[derive(Clone)]
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

    /// **These** Functions are too dangerous and take too long. Will not use
    /// but if I can get the working efficiently than that would be great

    /// 1st Attempt: **Speed**: 32.6s;
    /// 2nd Attempt: **Speed**: 17.2s;
    /// 3rd Attempt with `--release`: **Speed**: 12.5s;
    // pub async fn get_market(&self) -> mongodb::error::Result<Vec<Market>> { 
    //     let collection: Collection<Document> = self.db.collection("Market");

    //     // Collect Query
    //     let mut cursor: Cursor<Document> = collection.find(doc! {"_id": {"$gt": 0}}, None).await?;
    //     let all_items: Vec<Result<Document, mongodb::error::Error>> = cursor.collect().await;
    //     let roblox_users = Arc::new(self.get_roblox_users().await?);

    //     // Define item_instances
    //     let item_instances: Arc<Mutex<HashMap<u32, Document>>> = Arc::new(Mutex::new(HashMap::new()));

    //     // Define tasks
    //     let mut tasks: Vec<tokio::task::JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>>> = Vec::new();

    //     // Loop through all_items and insert into item_instances
    //     for docu in all_items {
    //         let docu = docu?;
    //         let roblox_users = Arc::clone(&roblox_users);
    //         let item_instances = Arc::clone(&item_instances);

    //         tasks.push(
    //             tokio::spawn(
    //                 async move {
    //                     let market_data = insert_users_into_market_data(docu, &roblox_users).await?;
    //                     let id = market_data.get_i32("_id").unwrap() as u32;
    //                     let mut item_instances = item_instances.lock().unwrap();
    //                     item_instances.insert(id, market_data);
    //                     Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
    //                 }
    //             )
    //         );
    //     }

    //     // Join all tasks
    //     let _ = futures::future::try_join_all(tasks).await;


    //     // Collect all items from item_instances
    //     let data = item_instances.lock().unwrap();
    //     let vec: Vec<Document> = data.values().cloned().collect();

    //     let mut markets: Vec<Market> = Vec::new();

    //     for doc in vec {
    //         let market: Market = mongodb::bson::from_document(doc).unwrap();
    //         markets.push(market);
    //     }

    //     Ok(markets)
    // }

    pub async fn get_market_for_item(&self, item: String) -> mongodb::error::Result<Vec<Document>> { 
        let collection: Collection<Document> = self.db.collection("Market");

        let projection = doc! {
            "_id": 1,
            "time_scanned": 1,
            format!("items.{}", item): 1
        };
        let find_options = FindOptions::builder().projection(projection).build();
        let cursor: Cursor<Document> = collection.find(doc! {"_id": {"$gt": 0}}, find_options).await?;
        let all_items: Vec<Result<Document, mongodb::error::Error>> = cursor.collect().await;
        let roblox_users = Arc::new(self.get_roblox_users().await?);

        let item_instances: Arc<Mutex<HashMap<u32, Document>>> = Arc::new(Mutex::new(HashMap::new()));

        let mut tasks: Vec<tokio::task::JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>>> = Vec::new();

        for docu in all_items {
            let docu = docu?;
            let roblox_users = Arc::clone(&roblox_users);
            let item_instances = Arc::clone(&item_instances);

            tasks.push(
                tokio::spawn(
                    async move {
                        let market_data = insert_users_into_market_data(docu, &roblox_users).await?;
                        let id = market_data.get_i32("_id").unwrap() as u32;
                        {
                            let mut item_instances = item_instances.lock().unwrap();
                            item_instances.insert(id, market_data);
                        }
                        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
                    }
                )
            );
        }

        let _ = futures::future::try_join_all(tasks).await;

        let data = item_instances.lock().unwrap();
        let vec: Vec<Document> = data.values().cloned().collect();

        Ok(vec)
    }

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

    // pub async fn get_latest_raw_instance(&self) -> mongodb::error::Result<Option<RawMarket>> {
    //     let count: Option<Count> = self.get_market_count().await?;
    //     if let Some(c) = count {
    //         let last_document: Option<Document> = self.db.collection("Market").find_one(doc! {"_id": c.count}, None).await?;
    //         let latest_instance: RawMarket = mongodb::bson::from_document(last_document.unwrap())?;
    //         Ok(Some(latest_instance))
    //     } else {
    //         Ok(None)
    //     }
    // }

    pub async fn get_latest_instance(&self, items: &Vec<String>) -> mongodb::error::Result<Option<MixedMarket>> {
        let mut queryMap: HashMap<String, Item> = HashMap::new();
        let col = self.db.collection("Market");
        let mut options = FindOneOptions::builder().sort(doc! {"_id": -1});

        let roblox_users = self.get_roblox_users().await?;

        for item in items {
            let built_option = options.clone().projection(doc! {"_id" : 1, "time_scanned": 1, format!("items.{}", item): 1}).build();
            let docu: Option<Document> = col.find_one(doc!{"_id": {"$gt": 0}, format!("items.{}", item): {"$exists": true}}, built_option).await?;
            if let Some(docu) = docu {
                let fixed_doc = insert_users_into_market_data(docu, &roblox_users).await?;
                let processed_doc: Market = mongodb::bson::from_document(fixed_doc)?;
                queryMap.insert(item.clone(), processed_doc.get_item(item).unwrap());
            }
        }


        Ok(Some(MixedMarket{
            items: queryMap
        }))
    }

    pub async fn get_recent_instance(&self) -> mongodb::error::Result<Option<Market>> {
        let count: Option<Count> = self.get_market_count().await?;
        if let Some(c) = count {
            let last_document = self.db.collection("Market").find_one(doc! {"_id": c.count}, None).await?;
            let market = last_document.unwrap();
            let roblox_users = self.get_roblox_users().await?;
            let fixed_doc = insert_users_into_market_data(market, &roblox_users).await?;
            let processed_doc: Market = mongodb::bson::from_document(fixed_doc)?;
            Ok(Some(processed_doc))
        } else {
            Ok(None)
        }
    }

    pub async fn get_recent_instance_filter(&self, items: &Vec<String>) -> mongodb::error::Result<Option<Market>> {
        let count: Option<Count> = self.get_market_count().await?;
        let mut filter = doc! {"_id": 1, "time_scanned": 1};
        for item in items {
            filter.insert(format!("items.{}", item), 1);
        }
        if let Some(c) = count {
            let options = FindOneOptions::builder().projection(filter).build();
            let last_document = self.db.collection("Market").find_one(doc! {"_id": c.count}, Some(options)).await?;
            let market = last_document.unwrap();
            let roblox_users = self.get_roblox_users().await?;
            let fixed_doc = insert_users_into_market_data(market, &roblox_users).await?;
            let processed_doc = mongodb::bson::from_document::<Market>(fixed_doc)?;
            Ok(Some(processed_doc))
        } else {
            Ok(None)
        }
    }
}