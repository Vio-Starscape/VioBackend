
use rocket::{
    State,
    http::{Status, RawStr},
    get,
    routes,
    Rocket,
    Build,
    serde::json::Json
};
use crate::objects::{database::VioDB, market::market::MixedMarket};
use crate::objects::market::market::Market;

#[get("/v1/market/latest?<items>")]
pub async fn latest_market(db: &State<VioDB>, items: Option<&str>) -> Option<Json<MixedMarket>> {

    match items {
        Some(items) => {
            let item = items.split(",").map(|name| name.to_string()).collect();
            let market = db.get_latest_instance(&item).await.ok().unwrap();
            match market {
                Some(market) => Some(Json(market)),
                None => None
            }
        },
        None => None
    }
}

#[get("/v1/market/recent?<items>")]
pub async fn recent_market(db: &State<VioDB>, items: Option<&str>) -> Option<Json<Market>> {

    match items {
        Some(items) => {
            let item = items.split(",").map(|name| name.to_string()).collect();
            let market = db.get_recent_instance_filter(&item).await.ok().unwrap();
            match market {
                Some(market) => Some(Json(market)),
                None => None
            }
        },
        None => {
            let market = db.get_recent_instance().await.ok().unwrap();
            match market {
                Some(market) => Some(Json(market)),
                None => None
            }
        }
    }
}