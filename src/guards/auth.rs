use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};
use rocket::outcome::{Outcome};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use crate::environment::Env;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
}

#[derive(Debug)]
pub enum AuthError {
    Missing,
    Invalid,
}

pub struct AuthGuard {
    #[allow(dead_code)]
    claims: Claims,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthGuard {
    type Error = AuthError;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let token = request.headers().get_one("Authorization");

        match token {
            Some(token) => {
                let token = token.trim_start_matches("Bearer ");

                let decoding_key = DecodingKey::from_secret(Env::jwt_secret().as_ref());
                let validation = Validation::default();

                match decode::<Claims>(token, &decoding_key, &validation) {
                    Ok(decoded) => {
                        Outcome::Success(AuthGuard { claims: decoded.claims })
                    }
                    Err(_) => Outcome::Error((Status::Unauthorized, AuthError::Invalid)),
                }
            }
            None => Outcome::Error((Status::Unauthorized, AuthError::Missing)),
        }
    }
}
