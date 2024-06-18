use rocket::{
    catch,
    request::Request,
    response::{self, Responder},
    Response,
};

use serde_json;
use rocket_okapi::{gen::OpenApiGenerator, OpenApiError};
use rocket_okapi::response::OpenApiResponderInner;
use rocket_okapi::okapi::openapi3::{Responses, MediaType};
use rocket_okapi::okapi::openapi3::RefOr;
use rocket_okapi::okapi;


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