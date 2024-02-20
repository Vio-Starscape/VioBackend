use mongodb::bson::{doc, Document, DateTime};

use std::collections::HashMap;
use std::fmt;

use serde::de::{self, SeqAccess, Visitor};
use serde::{Serialize, Deserialize};

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

#[derive(Serialize, Debug)]
pub struct Listing {
    vendor: u64,
    amount: u32,
    price: f32,
}

impl Listing {
    pub fn new(vendor: u64, amount: u32, price: f32) -> Listing {
        Listing {
            vendor,
            amount,
            price,
        }
    }

    pub fn vendor(&self) -> u64 {
        self.vendor
    }

    pub fn amount(&self) -> u32 {
        self.amount
    }

    pub fn price(&self) -> f32 {
        self.price
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Item {
    name: String,
    buy: Vec<Listing>,
    sell: Vec<Listing>,
}

impl Item {
    pub fn new() -> Item {
        Item {
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

    pub fn buy(&self) -> &Vec<Listing> {
        &self.buy
    }

    pub fn sell(&self) -> &Vec<Listing> {
        &self.sell
    }

    pub fn add_buy(&mut self, vendor: u64, amount: u32, price: f32) {
        self.buy.push(Listing::new(vendor, amount, price));
    }

    pub fn add_sell(&mut self, vendor: u64, amount: u32, price: f32) {
        self.sell.push(Listing::new(vendor, amount, price));
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Market {
    #[serde(rename = "_id")]
    id: u32,
    time_scanned: DateTime,
    location: String,
    items: HashMap<String, Item>,
}


impl Market {
    pub fn new() -> Market {
        Market {
            id: 0,
            time_scanned: DateTime::now(),
            location: String::from(""),
            // time_scanned: Map::new(),
            items: HashMap::new(),
        }
    }

    pub fn display(&self) {
        println!("ID: {}", self.id);
        println!("Time Scanned: {}", self.time_scanned);
        // println!("Items: {:?}", self.items);
    }

    // pub fn time_scanned(&self) -> &String {
    //     &self.time_scanned
    // }

    // pub fn items(&self) -> &HashMap<String, Item> {
    //     &self.items
    // }
}