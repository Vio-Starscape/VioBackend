use mongodb::bson::{doc, Document};
use mongodb::bson;
use chrono::{DateTime, Utc};

use std::collections::HashMap;
use std::fmt;

use serde::de::{self, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RobloxUser {
    #[serde(rename = "_id")]
    pub id: u64,
    pub name: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
}

impl RobloxUser{
    pub fn new(id: u64, name: &String, display_name: &String) -> RobloxUser {
        RobloxUser {
            id,
            name: name.to_string(),
            display_name: display_name.to_string(),
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn display_name(&self) -> &String {
        &self.display_name
    }

    pub fn url(&self) -> String {
        format!("https://rblx.name/{}", self.id)
    }

    pub fn display(&self) {
        println!("User ID: {}", self.id);
        println!("Name: {}", self.name);
        println!("Display Name: {}", self.display_name);
        println!("URL: {}", self.url());
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Listing {
    pub price: f32,
    pub amount: u32,
    pub vendor: RobloxUser,
}

impl Listing {
    pub fn new(price: f32, amount: u32, vendor: RobloxUser) -> Listing {
        Listing {
            price,
            amount,
            vendor,
        }
    }

    pub fn price(&self) -> f32 {
        self.price
    }

    pub fn amount(&self) -> u32 {
        self.amount
    }

    pub fn vendor(&self) -> &RobloxUser {
        &self.vendor
    }
}

struct ListingVisitor;

impl <'de> Visitor<'de> for ListingVisitor {
    type Value = Listing;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("struct Listing")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let price = seq.next_element()?
            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
        let amount = seq.next_element()?
            .ok_or_else(|| de::Error::invalid_length(1, &self))?;
        let vendor = seq.next_element()?
            .ok_or_else(|| de::Error::invalid_length(2, &self))?;
        Ok(Listing { vendor, amount, price })
    }
}

impl <'de> Deserialize<'de> for Listing {
    fn deserialize<D>(deserializer: D) -> Result<Listing, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_seq(ListingVisitor)
    }
}

// fn datetime_to_bson<S>(val: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
// where
//     S: Serializer,
// {
//     let bson_datetime = bson::DateTime::from_chrono(val);
//     bson_datetime.serialize(serializer)
// }

fn datetime_from_bson<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let bson_datetime = bson::DateTime::deserialize(deserializer)?;
    Ok(bson_datetime.to_chrono())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Item {
    name: String,
    #[serde(rename = "_id")]
    id: u32,
    #[serde(deserialize_with = "datetime_from_bson")]
    time_scanned: DateTime<Utc>,
    buy: Vec<Listing>,
    sell: Vec<Listing>,
}

impl Item {
    pub fn new() -> Item {
        Item {
            name: String::from(""),
            id: 0,
            time_scanned: Utc::now(),
            buy: Vec::new(),
            sell: Vec::new(),
        }
    }
    /// Return the name of the Item
    ///
    /// # Returns
    ///
    /// `String` - The name of the item
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn buy(&self) -> &Vec<Listing> {
        &self.buy
    }

    pub fn sell(&self) -> &Vec<Listing> {
        &self.sell
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Market {
    #[serde(rename = "_id")]
    id: u32,
    #[serde(deserialize_with = "datetime_from_bson")]
    time_scanned: DateTime<Utc>,
    items: HashMap<String, Item>,
}


impl Market {
    pub fn new() -> Market {
        Market {
            id: 0,
            time_scanned: Utc::now(),
            items: HashMap::new(),
        }
    }

    pub fn display(&self) {
        println!("ID: {}", self.id);
        println!("Time Scanned: {}", self.time_scanned);
        // println!("Items: {:?}", self.items);
    }

    pub fn time_scanned(&self) -> &DateTime<Utc>{
        &self.time_scanned
    }

    pub fn items(&self) -> &HashMap<String, Item> {
        &self.items
    }

    pub fn get_item(&self, name: &String) -> Option<Item> {
        Some(self.items.get(name)?.clone())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MixedMarket{
    pub items: HashMap<String, Item>
}