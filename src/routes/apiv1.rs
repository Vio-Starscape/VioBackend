use rocket::{
    catch, get, http::Status, request::{FromRequest, Outcome, Request}, response::Responder, serde::json::Json, State,
    response, Response
};

use log::{info, debug};

use serde_json;
use rocket_okapi::{openapi, request::{OpenApiFromRequest, RequestHeaderInput}, gen::OpenApiGenerator, OpenApiError};
use rocket_okapi::response::OpenApiResponderInner;
use rocket_okapi::okapi::openapi3::{Object, Responses, SecurityRequirement, SecurityScheme, SecuritySchemeData, MediaType};
use rocket_okapi::okapi::openapi3::RefOr;
use rocket_okapi::okapi;
use crate::objects::{database::VioDB, market::market::MixedMarket, market::market::Item};
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
            },
            ..Default::default()
        })
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
///
/// If no items are provided, will not return anything / 404.
#[openapi]
#[get("/market/latest?<items>")]
pub async fn latest_market(_key: ApiKey, db: &State<VioDB>, items: Option<&str>) -> Result<Json<MixedMarket>, Status> {
    info!("Getting latest market data for items: {:?} | by: {}", &items, _key.0);
    match items {
        Some(items) => {
            let item = items.split(",").map(|name| name.to_string()).collect();
            let market = db.get_latest_instance(&item).await.ok().unwrap();
            match market {
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
#[openapi]
#[get("/market/recent?<items>")]
pub async fn recent_market(_key:ApiKey, db: &State<VioDB>, items: Option<&str>) -> Result<Json<Market>, Status> {
    info!("Getting recent market data for items: {:?} | by: {}", &items, _key.0);
    match items {
        Some(items) => {
            let item = items.split(",").map(|name| name.to_string()).collect();
            println!("{:?}", item);
            let market = db.get_recent_instance_filter(&item).await.ok();
            println!("{:?}", market);
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