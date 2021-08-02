use rocket::http::Status;
use rocket::outcome::Outcome;
use rocket::request::{self, FromRequest};
use rocket::Request;

use crate::db::DBConn;
use crate::db_queries;

pub struct ApiToken;

#[derive(Debug)]
pub enum ApiTokenError {
    NoHeader,
    MalformedHeader,
    InvalidToken,
    DatabaseError,
}

fn parse_auth_header(header: &str) -> Option<&str> {
    let split_header = header.split(' ').collect::<Vec<_>>();

    if split_header.len() != 2 || split_header[0] != "Bearer" {
        return None;
    }

    Some(split_header[1])
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiToken {
    type Error = ApiTokenError;

    // TODO rewrite
    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let auth_header = req.headers().get_one("Authorization");

        if auth_header.is_none() {
            return Outcome::Failure((Status::Unauthorized, ApiTokenError::NoHeader));
        }

        let parsed_header = parse_auth_header(auth_header.unwrap());
        if parsed_header.is_none() {
            return Outcome::Failure((Status::BadRequest, ApiTokenError::MalformedHeader));
        }

        let token = parsed_header.unwrap().to_string();

        if let Outcome::Success(db) = req.guard::<DBConn>().await {
            let token_row = db
                .run(move |conn| db_queries::token_exists(conn, &token))
                .await;

            match token_row {
                Ok(true) => Outcome::Success(ApiToken),
                Ok(false) => Outcome::Failure((Status::Unauthorized, ApiTokenError::InvalidToken)),
                Err(_) => {
                    Outcome::Failure((Status::InternalServerError, ApiTokenError::DatabaseError))
                }
            }
        } else {
            Outcome::Failure((Status::InternalServerError, ApiTokenError::DatabaseError))
        }
    }
}
