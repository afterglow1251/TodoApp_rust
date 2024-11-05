use sqlx::PgPool;
use jsonwebtoken::{encode, Header, EncodingKey};
use bcrypt::{hash, verify};
use rocket::State;
use rocket::http::Status;
use rocket::serde::json::Json;
use crate::environment::Env;
use crate::utils::time::{get_current_timestamp};
use crate::utils::validators::{is_email};
use crate::constants::common::{BCRYPT_COST, JWT_EXPIRATION_SECONDS};
use crate::types::common::{User};
use crate::types::responses::{LoginResponse, MessageOnlyResponse, Claims};


#[post("/register", format = "json", data = "<user>")]
pub async fn register(user: Json<User>, pool: &State<PgPool>) -> Result<Json<MessageOnlyResponse>, (Status, Json<MessageOnlyResponse>)> {
    if !is_email(&user.email) {
        return Err((
            Status::BadRequest,
            Json(MessageOnlyResponse {
                message: "Invalid email address!".to_string(),
            }),
        ));
    }

    if let Ok(Some(_)) = sqlx::query!("SELECT id FROM users WHERE email = $1", user.email)
        .fetch_optional(pool.inner())
        .await
    {
        return Err((
            Status::Conflict,
            Json(MessageOnlyResponse {
                message: "User with this email already exists!".to_string(),
            }),
        ));
    }

    let hashed_password = match hash(&user.password, BCRYPT_COST) {
        Ok(h) => h,
        Err(_) => {
            return Err((
                Status::InternalServerError,
                Json(MessageOnlyResponse {
                    message: "Failed to hash password!".to_string(),
                }),
            ));
        }
    };

    let result = sqlx::query!(
        "INSERT INTO users (email, password) VALUES ($1, $2)",
        user.email,
        hashed_password
    )
        .execute(pool.inner())
        .await;

    match result {
        Ok(_) => Ok(Json(MessageOnlyResponse {
            message: "User registered successfully!".to_string(),
        })),
        Err(_) => Err((
            Status::InternalServerError,
            Json(MessageOnlyResponse {
                message: "Failed to register user!".to_string(),
            }),
        )),
    }
}

#[post("/login", format = "json", data = "<user>")]
pub async fn login(user: Json<User>, pool: &State<PgPool>) -> Result<Json<LoginResponse>, (Status, Json<LoginResponse>)> {
    match sqlx::query!("SELECT id, password FROM users WHERE email = $1", user.email)
        .fetch_one(pool.inner())
        .await
    {
        Ok(record) => {
            match verify(&user.password, &record.password) {
                Ok(valid) if valid => {
                    let claims = Claims {
                        sub: user.email.clone(),
                        exp: get_current_timestamp() + JWT_EXPIRATION_SECONDS,
                    };

                    let jwt_secret = Env::jwt_secret();
                    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(jwt_secret.as_bytes())).unwrap();

                    Ok(Json(LoginResponse {
                        message: format!("User {} logged in successfully!", user.email),
                        token: Some(token),
                    }))
                }
                _ => {
                    Err((
                        Status::Unauthorized,
                        Json(LoginResponse {
                            message: "Invalid email or password!".to_string(),
                            token: None,
                        }),
                    ))
                }
            }
        },
        Err(_) => {
            Err((
                Status::Unauthorized,
                Json(LoginResponse {
                    message: "Invalid email or password!".to_string(),
                    token: None,
                }),
            ))
        },
    }
}

#[post("/logout", format = "json")]
pub fn logout() -> Json<MessageOnlyResponse> {
    Json(MessageOnlyResponse {
        message: "User logged out successfully!".to_string(),
    })
}