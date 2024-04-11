use mongodb::bson::doc;
use mongodb::bson;
use chrono::{DateTime, Utc};

use std::collections::HashMap;
use std::fmt;

use serde::de::{self, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use schemars::JsonSchema;

fn deserialize_id<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let f = f64::deserialize(deserializer)?;
    Ok(f as u64)
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct RobloxUser {
    #[serde(rename = "_id", deserialize_with = "deserialize_id")]
    pub id: u64,
    pub name: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
}

#[derive(Serialize, Debug, Clone, JsonSchema)]
pub struct Listing {
    pub price: f64,
    pub amount: u32,
    pub vendor: RobloxUser,
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

fn datetime_from_bson<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let bson_datetime = bson::DateTime::deserialize(deserializer)?;
    Ok(bson_datetime.to_chrono())
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct Item {
    name: String,
    #[serde(rename = "_id")]
    pub id: u32,
    #[serde(deserialize_with = "datetime_from_bson")]
    time_scanned: DateTime<Utc>,
    buy: Vec<Listing>,
    sell: Vec<Listing>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct Market {
    #[serde(rename = "_id")]
    d: u32,
    #[serde(deserialize_with = "datetime_from_bson")]
    time_scanned: DateTime<Utc>,
    items: HashMap<String, Item>,
}


impl Market {
    pub fn get_item(&self, name: &String) -> Option<Item> {
        Some(self.items.get(name)?.clone())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct MixedMarket{
    pub items: HashMap<String, Item>
}