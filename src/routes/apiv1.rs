use rocket::{
    catch, get, http::Status, post, request::{FromRequest, Outcome, Request}, response::{self, Responder}, serde::json::Json, Response, State
};
use rocket_ws::{WebSocket, Channel};

use log::{info, debug};
use std::env;
use serde_json;
use rocket_okapi::{openapi, request::{OpenApiFromRequest, RequestHeaderInput}, gen::OpenApiGenerator, OpenApiError};
use rocket_okapi::response::OpenApiResponderInner;
use rocket_okapi::okapi::openapi3::{Object, Responses, SecurityRequirement, SecurityScheme, SecuritySchemeData, MediaType};
use rocket_okapi::okapi::openapi3::RefOr;
use rocket_okapi::okapi;
use crate::objects::{database::VioDB, market::{market::{Item, MixedMarket}, raw_market::RawMarket}};
use crate::objects::market::market::Market;

#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
pub struct MyError {
    pub err: String,
    pub msg: Option<String>,
    #[serde(skip)]
    pub http_status_code: u16
}

impl<'r> Responder<'r, 'static> for MyError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        // Convert object to json
        let body = serde_json::to_string(&self).unwrap();
        Response::build()
            .sized_body(body.len(), std::io::Cursor::new(body))
            .header(rocket::http::ContentType::JSON)
            .status(rocket::http::Status::new(self.http_status_code))
            .ok()
    }
}

impl OpenApiResponderInner for MyError {
    fn responses(gen: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        Ok(Responses {
            responses: okapi::map! {
                "400".to_owned() => RefOr::Object(bad_request_response(gen)),
                "401".to_owned() => RefOr::Object(unauthorized_response(gen)),
                "404".to_owned() => RefOr::Object(not_found_response(gen)),
                "422".to_owned() => RefOr::Object(unprocessable_entity_response(gen)),
            },
            ..Default::default()
        })
    }
}

#[catch(422)]
pub fn unprocessable_entity() -> MyError {
    MyError {
        err: "Unprocessable Entity".to_owned(),
        msg: Some("The request was well-formed but was unable to be followed due to semantic errors.".to_owned()),
        http_status_code: 422,
    }
}

#[catch(401)]
pub fn unauthorized() -> MyError {
    MyError {
        err: "Unauthorized".to_owned(),
        msg: Some("The authentication given was incorrect or insufficient.".to_owned()),
        http_status_code: 401,
    }
}

#[catch(404)]
pub fn not_found() -> MyError {
    MyError {
        err: "Not Found".to_owned(),
        msg: Some("The requested resource could not be found.".to_owned()),
        http_status_code: 404,
    }
}

#[catch(400)]
pub fn bad_request() -> MyError {
    MyError {
        err: "Bad Request".to_owned(),
        msg: Some("The request was malformed.".to_owned()),
        http_status_code: 400,
    }
}


pub fn unauthorized_response(gen: &mut OpenApiGenerator) -> okapi::openapi3::Response  {
    let schema = gen.json_schema::<MyError>();
    okapi::openapi3::Response {
        description: "\
        # 401 Unauthorized\n\
        The authentication given was incorrect or insufficient. \
        "
        .to_owned(),
        content: okapi::map! {
            "application/json".to_owned() => MediaType {
                schema: Some(schema),
                ..Default::default()
            }
        },
        ..Default::default()
    }
}

pub fn not_found_response(gen: &mut OpenApiGenerator) -> okapi::openapi3::Response  {
    let schema = gen.json_schema::<MyError>();
    okapi::openapi3::Response {
        description: "\
        # 404 Not Found\n\
        The requested resource could not be found. \
        "
        .to_owned(),
        content: okapi::map! {
            "application/json".to_owned() => MediaType {
                schema: Some(schema),
                ..Default::default()
            }
        },
        ..Default::default()
    }
}

pub fn bad_request_response(gen: &mut OpenApiGenerator) -> okapi::openapi3::Response  {
    let schema = gen.json_schema::<MyError>();
    okapi::openapi3::Response {
        description: "\
        # 400 Bad Request\n\
        The request was malformed. \
        "
        .to_owned(),
        content: okapi::map! {
            "application/json".to_owned() => MediaType {
                schema: Some(schema),
                ..Default::default()
            }
        },
        ..Default::default()
    }
}

pub fn unprocessable_entity_response(gen: &mut OpenApiGenerator) -> okapi::openapi3::Response  {
    let schema = gen.json_schema::<MyError>();
    okapi::openapi3::Response {
        description: "\
        # 422 Unprocessable Entity\n\
        The request was well-formed but was unable to be followed due to semantic errors. \
        "
        .to_owned(),
        content: okapi::map! {
            "application/json".to_owned() => MediaType {
                schema: Some(schema),
                ..Default::default()
            }
        },
        ..Default::default()
    }
}

pub struct ApiKey(String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKey {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let key_header = request.headers().get_one("x-api-key");
        let db = request.guard::<&State<VioDB>>().await.unwrap();
        match key_header {
            Some(key) if db.confirm_api_key(key).await.unwrap() => {
                debug!("API Key: {}", key);
                Outcome::Success(ApiKey(key.to_string()))
            },
            _ => Outcome::Error((Status::Unauthorized, ()))
        }
    }
}

impl<'a> OpenApiFromRequest<'a> for ApiKey {
    fn from_request_input(
        _gen: &mut OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> rocket_okapi::Result<RequestHeaderInput> {
        // Setup global requirement for Security scheme
        let security_scheme = SecurityScheme {
            description: Some("Requires an API key to access.".to_owned()),
            data: SecuritySchemeData::ApiKey {
                name: "x-api-key".to_owned(),
                location: "header".to_owned(),
            },
            extensions: Object::default(),
        };

        let mut security_req = SecurityRequirement::new();

        security_req.insert("ApiKeyAuth".to_owned(), Vec::new());

        Ok(RequestHeaderInput::Security(
            "ApiKeyAuth".to_owned(),
            security_scheme,
            security_req,
        ))
    }

    fn get_responses(gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {

        Ok(Responses {
            responses: okapi::map! {
                "400".to_owned() => RefOr::Object(bad_request_response(gen)),
                "401".to_owned() => RefOr::Object(unauthorized_response(gen)),
                "404".to_owned() => RefOr::Object(not_found_response(gen)),
            },
            ..Default::default()
        })
    }
}


/// # Latest Market (Keyword Only)
///
/// Get the latest market data for a list of items.
/// This difference between this and the recent market is that this will look for the last instance of an Item.
/// It should always return something if the item is in the database.
///
/// If no items are provided, will 404.
#[openapi]
#[get("/market/latest?<items>")]
pub async fn latest_market(_key: ApiKey, db: &State<VioDB>, items: Option<&str>) -> Result<Json<MixedMarket>, Status> {
    info!("Getting latest market data for items: {:?} | by: {}", &items, _key.0);
    match items {
        Some(items) => {
            let item = items.split(",").map(|name| name.to_string()).collect();
            let market = db.get_latest_instance(&item).await;
            match market.ok() {
                Some(market) => Ok(Json(market)),
                None => Err(Status::NotFound)
            }
        },
        None => Err(Status::BadRequest)
    }
}

/// # Recent Market
///
/// Get the most recent market data.
///
/// If no items are provided, will return the entire most recent scan.
/// else will return only the requested items in the most recent scan.
/// 
#[openapi]
#[get("/market/recent?<items>")]
pub async fn recent_market(_key:ApiKey, db: &State<VioDB>, items: Option<&str>) -> Result<Json<Market>, Status> {
    info!("Getting recent market data for items: {:?} | by: {}", &items, _key.0);
    match items {
        Some(items) => {
            let item = items.split(",").map(|name| name.to_string()).collect();
            let market = db.get_recent_instance_filter(&item).await.ok();
            match market.unwrap() {
                Some(market) => Ok(Json(market)),
                None => Err(Status::NotFound)
            }
        },
        None => {
            let market = db.get_recent_instance().await.ok().unwrap();
            match market {
                Some(market) => Ok(Json(market)),
                None => Err(Status::NotFound)
            }
        }
    }
}

/// # Item List
///
/// Get a list of all items that can be requested.
#[openapi]
#[get("/market/items")]
pub async fn item_list(_key: ApiKey, db: &State<VioDB>) -> Json<Vec<String>> {
    info!("Getting item list | by: {}", _key.0);
    let items = db.get_item_list().await.ok().unwrap();
    Json(items)
}

/// # Item History
/// 
/// Get the history of an item.
/// This function returns a lot of data, and is not recommended to be used for large amounts of items.
/// If you call this function, you should most likely cache the data as it won't ever change other than new items.
/// 
#[openapi]
#[get("/market/history/<item>")]
pub async fn item_history(_key: ApiKey, db: &State<VioDB>, item: &str) -> Result<Json<Vec<Item>>, Status> {
    info!("Getting market data history for item: {:?} | by: {}", &item, _key.0);
    let market = db.get_market_for_item(item.to_string()).await.ok().unwrap();
    Ok(Json(market))
}


// Websocket and Dataendpoint

// type SharedState = std::sync::Arc<tokio::sync::Mutex<HashMap<String, WebSocket>>>;


#[openapi(skip)]
#[post("/market/insert/data", format="json", data="<data>")]
pub async fn insert_data(_key: ApiKey, db: &State<VioDB>, data: Json<RawMarket>) -> Result<Json<String>, Status> {
    info!("Inserting data | by: {}", _key.0);
    if _key.0 != env::var("ADMIN_KEY").unwrap() {
        return Err(Status::Unauthorized);
    }
    db.add_market_to_database(data.0).await.ok().unwrap();
    info!("Data Inserted");
    Ok(Json("Data Inserted".to_string()))
}


// #[openapi()]
// #[get("/market/ws")]
// pub async fn ws_endpoint(_key: ApiKey, ws: WebSocket) -> Channel<'static> {
//     info!("Getting websocket endpoint | by: {}", _key.0);
//     use rocket::futures::{SinkExt, StreamExt};

//     ws.channel(move |mut stream| Box::pin(async move{
//         while let Some(msg) = stream.next().await {
//             let _ = stream.send(msg?).await;
//         }

//         Ok(())
//     }))
// }