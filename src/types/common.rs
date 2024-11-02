use rocket::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: Option<i32>,
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct Todo {
    pub id: Option<i32>,
    pub title: String,
    pub description: String,
    pub completed: bool,
    pub user_id: i32,
}

#[derive(Serialize, Deserialize)]
pub struct OptionalTodo {
    pub id: Option<i32>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub completed: Option<bool>,
    pub user_id: Option<i32>,
}