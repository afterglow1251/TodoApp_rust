use rocket::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

#[derive(Serialize, Deserialize)]
pub struct MessageOnlyResponse {
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    pub message: String,
    pub token: Option<String>,
}