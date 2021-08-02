use rocket::http::Status;
use rocket::outcome::Outcome::{Failure, Success};
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

async fn validate_request_api_token(req: &Request<'_>) -> Result<(), ApiTokenError> {
    let auth_header = req
        .headers()
        .get_one("Authorization")
        .ok_or(ApiTokenError::NoHeader)?;

    let token = parse_auth_header(auth_header)
        .ok_or(ApiTokenError::MalformedHeader)?
        .to_string();

    let db = req
        .guard::<DBConn>()
        .await
        .success_or(ApiTokenError::DatabaseError)?;

    let token_exists = db
        .run(move |conn| db_queries::token_exists(conn, &token))
        .await
        .map_err(|_| ApiTokenError::DatabaseError)?;

    if token_exists {
        Ok(())
    } else {
        Err(ApiTokenError::InvalidToken)
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiToken {
    type Error = ApiTokenError;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        match validate_request_api_token(req).await {
            Ok(()) => Success(ApiToken),
            Err(ApiTokenError::DatabaseError) => {
                Failure((Status::InternalServerError, ApiTokenError::DatabaseError))
            }
            Err(ApiTokenError::InvalidToken) => {
                Failure((Status::Unauthorized, ApiTokenError::InvalidToken))
            }
            Err(ApiTokenError::MalformedHeader) => {
                Failure((Status::BadRequest, ApiTokenError::MalformedHeader))
            }
            Err(ApiTokenError::NoHeader) => {
                Failure((Status::Unauthorized, ApiTokenError::MalformedHeader))
            }
        }
    }
}
