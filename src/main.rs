#[macro_use]
extern crate rocket;

mod routes;
mod types;
mod constants;
mod environment;
mod utils;
mod guards;

use dotenv::dotenv;
use crate::environment::{Env};
use sqlx::PgPool;
use routes::auth::{register, login, logout};
use routes::todos::{create_todo, get_todo, get_todos, update_todo, delete_todo, save_todo, upload_todo};

#[launch]
async fn rocket() -> _ {
    dotenv().ok();

    let database_url = Env::database_url();
    let pool = PgPool::connect(&database_url).await.expect("Failed to connect to the database");

    rocket::build()
        .manage(pool)
        .mount("/api/auth", routes![register, login, logout])
        .mount("/api/todos", routes![create_todo, get_todo, delete_todo, update_todo, get_todos, save_todo, upload_todo])
}
