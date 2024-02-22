use mongodb::bson::{doc, DateTime};

use std::collections::HashMap;
use std::fmt;

use serde::de::{self, SeqAccess, Visitor};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Debug)]
pub struct RawListing {
    user: u64,
    amount: u32,
    price: f32,
}

impl RawListing {
    pub fn new(user: u64, amount: u32, price: f32) -> RawListing {
        RawListing {
            user,
            amount,
            price,
        }
    }

    pub fn user(&self) -> u64 {
        self.user
    }

    pub fn amount(&self) -> u32 {
        self.amount
    }

    pub fn price(&self) -> f32 {
        self.price
    }
}
struct RawListingVisitor;

impl <'de> Visitor<'de> for RawListingVisitor {
    type Value = RawListing;

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
        let user = seq.next_element()?
            .ok_or_else(|| de::Error::invalid_length(2, &self))?;
        Ok(RawListing { user, amount, price })
    }
}

impl <'de> Deserialize<'de> for RawListing {
    fn deserialize<D>(deserializer: D) -> Result<RawListing, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_seq(RawListingVisitor)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RawItem {
    name: String,
    buy: Vec<RawListing>,
    sell: Vec<RawListing>,
}

impl RawItem {
    pub fn new() -> RawItem {
        RawItem {
            name: String::from(""),
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

    pub fn buy(&self) -> &Vec<RawListing> {
        &self.buy
    }

    pub fn sell(&self) -> &Vec<RawListing> {
        &self.sell
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RawMarket {
    #[serde(rename = "_id")]
    id: u32,
    time_scanned: DateTime,
    location: String,
    items: HashMap<String, RawItem>,
}


impl RawMarket {
    pub fn new() -> RawMarket {
        RawMarket {
            id: 0,
            time_scanned: DateTime::now(),
            location: String::from(""),
            items: HashMap::new(),
        }
    }

    pub fn display(&self) {
        println!("ID: {}", self.id);
        println!("Time Scanned: {}", self.time_scanned);
        // println!("Items: {:?}", self.items);
    }

    pub fn time_scanned(&self) -> &DateTime {
        &self.time_scanned
    }

    pub fn items(&self) -> &HashMap<String, RawItem> {
        &self.items
    }
}