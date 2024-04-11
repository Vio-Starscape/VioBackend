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

#[derive(Serialize, Deserialize, Debug)]
pub struct RawMarket {
    #[serde(rename = "_id")]
    id: u32,
    time_scanned: DateTime,
    location: String,
    items: HashMap<String, RawItem>,
}