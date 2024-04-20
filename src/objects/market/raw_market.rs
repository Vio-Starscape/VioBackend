use bson::Bson;
use mongodb::bson::doc;
use rust_decimal::Decimal;
use std::collections::BTreeMap;
use std::fmt;
use chrono::{DateTime, Utc};
use chrono::serde::ts_milliseconds::serialize as chrono_serialize;
use schemars::JsonSchema;
use serde::de::{self, SeqAccess, Visitor};
use serde::{Deserialize, Serialize, Serializer, ser::SerializeSeq};

#[derive(Debug)]
pub struct RawListing {
    pub user: u64,
    pub amount: u32,
    pub price: Decimal,
}

impl Serialize for RawListing {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(3))?;
        seq.serialize_element(&self.price)?;
        let price = Bson::Decimal128(self.price.into());
        seq.serialize_element(&self.amount)?;
        seq.serialize_element(&self.user)?;
        seq.end()
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

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct RawItem {
    pub name: String,
    pub buy: Vec<RawListing>,
    pub sell: Vec<RawListing>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct RawMarket {
    #[serde(serialize_with = "chrono_serialize")]
    pub time_scanned: DateTime<Utc>,
    pub location: String,
    pub items: BTreeMap<String, RawItem>,
}