use crate::routes::ratelimiter;
use crate::routes::errors::{bad_request_response, unauthorized_response, not_found_response};
use std::env;
use rocket::{
    http::Status,
    request::{FromRequest, Outcome, Request},
    State
};

use rocket_okapi::{
    request::{OpenApiFromRequest, RequestHeaderInput},
    gen::OpenApiGenerator
};
use rocket_okapi::okapi::openapi3::{Object, RefOr, Responses, SecurityRequirement, SecurityScheme, SecuritySchemeData};
use rocket_okapi::okapi;
use crate::objects::database::VioDB;

pub struct ApiKey(String);

impl ApiKey {
    pub fn value(&self) -> &str {
        &self.0
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKey {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let key_header = request.headers().get_one("x-api-key");
        let db = request.guard::<&State<VioDB>>().await.unwrap();
        let count = request.guard::<&State<ratelimiter::RequestCount>>().await.unwrap();

        match key_header {
            Some(key) if db.confirm_api_key(key).await.unwrap() => {

                if key == &env::var("ADMIN_KEY").unwrap() {
                    // Bypass rate limiting for admin keys
                    return Outcome::Success(ApiKey(key.to_string()));
                }

                let counter = count.increment(key);
                if counter > 60 {
                    Outcome::Error((Status::TooManyRequests, ()))
                } else {
                    Outcome::Success(ApiKey(key.to_string()))
                }
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