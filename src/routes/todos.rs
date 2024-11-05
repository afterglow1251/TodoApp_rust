use std::fs::File;
use std::io::Read;
use std::path::Path;
use sqlx::PgPool;
use tokio::fs::File as AsyncFile;
use tokio::io::AsyncWriteExt;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{State, Data};
use rocket_multipart_form_data::{MultipartFormData, MultipartFormDataOptions, MultipartFormDataField, mime};
use crate::types::common::{Todo, OptionalTodo};
use crate::types::responses::{MessageOnlyResponse};
use crate::guards::auth::AuthGuard;


#[post("/upload", data = "<data>")]
pub async fn upload_todo(
    __auth: AuthGuard,
    content_type: &rocket::http::ContentType,
    data: Data<'_>,
    pool: &State<PgPool>,
) -> Result<&'static str, &'static str> {
    let options = MultipartFormDataOptions::with_multipart_form_data_fields(vec![
        MultipartFormDataField::file("file").content_type_by_string(Some(mime::TEXT_PLAIN)).unwrap(),
        MultipartFormDataField::text("title"),
        MultipartFormDataField::text("completed"),
        MultipartFormDataField::text("user_id"),
    ]);

    let multipart_form_data = MultipartFormData::parse(content_type, data, options)
        .await
        .map_err(|_| "Failed to parse multipart form data")?;

    let mut description = String::new();
    if let Some(file_fields) = multipart_form_data.files.get("file") {
        let file_field = &file_fields[0];

        let mut file = File::open(&file_field.path).map_err(|_| "Failed to open file")?;
        file.read_to_string(&mut description).map_err(|_| "Failed to read file")?;
        println!("File content:\n{}", description);
    } else {
        return Err("No file found in the request");
    }

    let title = multipart_form_data.texts.get("title")
        .and_then(|fields| fields.get(0))
        .map(|field| field.text.clone())
        .unwrap_or_else(|| "Untitled".to_string());

    let completed = multipart_form_data.texts.get("completed")
        .and_then(|fields| fields.get(0))
        .map(|field| field.text.parse::<bool>().unwrap_or(false))
        .unwrap_or(false);

    let user_id_str = multipart_form_data.texts.get("user_id")
        .and_then(|fields| fields.get(0))
        .map(|field| field.text.clone())
        .unwrap_or_default();

    let user_id: i32 = user_id_str.parse().map_err(|_| "Invalid user_id")?;

    let result = sqlx::query!(
        "INSERT INTO todos (user_id, title, description, completed) VALUES ($1, $2, $3, $4)",
        user_id,
        title,
        description,
        completed,
    )
        .execute(pool.inner())
        .await;

    match result {
        Ok(_) => Ok("Todo uploaded successfully"),
        Err(_) => Err("Failed to save todo to the database"),
    }
}

#[get("/save/<id>", format = "json")]
pub async fn save_todo(
    _auth: AuthGuard,
    id: i32,
    pool: &State<PgPool>,
) -> Result<Json<MessageOnlyResponse>, (Status, Json<MessageOnlyResponse>)> {
    let todo = sqlx::query!(
        "SELECT title, description FROM todos WHERE id = $1",
        id
    )
        .fetch_one(pool.inner())
        .await;

    let todo = match todo {
        Ok(t) => t,
        Err(_) => {
            let error_message = format!("Todo with id {} not found", id);
            return Err((Status::NotFound, Json(MessageOnlyResponse { message: error_message })));
        }
    };

    let content = todo.description;

    let file_path = format!("todo_{}.txt", id);
    let path = Path::new(&file_path);

    let mut file = AsyncFile::create(&path)
        .await
        .map_err(|_| (Status::InternalServerError, Json(MessageOnlyResponse { message: "Failed to create file".to_string() })))?;

    file.write_all(content.as_bytes())
        .await
        .map_err(|_| (Status::InternalServerError, Json(MessageOnlyResponse { message: "Failed to write to file".to_string() })))?;

    Ok(Json(MessageOnlyResponse {
        message: format!("Todo with id {} saved to {}", id, file_path),
    }))
}

#[post("/", format = "json", data = "<todo>")]
pub async fn create_todo(
    _auth: AuthGuard,
    todo: Json<Todo>,
    pool: &State<PgPool>,
) -> Result<Json<Todo>, String> {
    let new_todo = sqlx::query!(
        "INSERT INTO todos (user_id, title, description, completed) VALUES ($1, $2, $3, $4) RETURNING id",
        todo.user_id,
        todo.title,
        todo.description,
        todo.completed,
    )
        .fetch_one(pool.inner())
        .await
        .map_err(|_| "Failed to create todo".to_string())?;

    Ok(Json(Todo {
        id: Some(new_todo.id),
        title: todo.title.clone(),
        description: todo.description.clone(),
        completed: todo.completed.clone(),
        user_id: todo.user_id.clone(),
    }))
}

#[get("/<id>", format = "json")]
pub async fn get_todo(
    _auth: AuthGuard,
    id: i32,
    pool: &State<PgPool>,
) -> Result<Json<Todo>, (Status, Json<MessageOnlyResponse>)> {
    let todo = sqlx::query!(
        "SELECT id, title, description, completed, user_id FROM todos WHERE id = $1",
        id
    )
        .fetch_one(pool.inner())
        .await;

    match todo {
        Ok(todo) => {
            let todo_response = Todo {
                id: Some(todo.id),
                title: todo.title,
                description: todo.description,
                completed: todo.completed.unwrap_or(false),
                user_id: todo.user_id.ok_or("User ID is missing").unwrap(),
            };
            Ok(Json(todo_response))
        }
        Err(_) => {
            let error_message = format!("Todo with id {} not found", id);
            Err((Status::NotFound, Json(MessageOnlyResponse { message: error_message })))
        }
    }
}

#[delete("/<id>", format = "json")]
pub async fn delete_todo(
    _auth: AuthGuard,
    id: i32,
    pool: &State<PgPool>,
) -> Result<Json<MessageOnlyResponse>, (Status, Json<MessageOnlyResponse>)> {
    let result = sqlx::query!("DELETE FROM todos WHERE id = $1", id)
        .execute(pool.inner())
        .await;

    match result {
        Ok(query_result) => {
            let rows_affected = query_result.rows_affected();

            if rows_affected > 0 {
                Ok(Json(MessageOnlyResponse {
                    message: format!("Todo with id {} deleted successfully!", id),
                }))
            } else {
                let error_message = format!("Todo with id {} not found", id);
                Err((Status::NotFound, Json(MessageOnlyResponse { message: error_message })))
            }
        }
        Err(_) => {
            let error_message = "Failed to delete todo".to_string();
            Err((Status::InternalServerError, Json(MessageOnlyResponse { message: error_message })))
        }
    }
}

#[patch("/<id>", format = "json", data = "<updated_todo>")]
pub async fn update_todo(
    _auth: AuthGuard,
    id: i32,
    updated_todo: Json<OptionalTodo>,
    pool: &State<PgPool>,
) -> Result<Json<Todo>, (Status, Json<MessageOnlyResponse>)> {
    let current_todo = sqlx::query!(
        "SELECT id, title, description, completed, user_id FROM todos WHERE id = $1",
        id
    )
        .fetch_one(pool.inner())
        .await;

    let current_todo = match current_todo {
        Ok(todo) => todo,
        Err(_) => {
            let error_message = format!("Todo with id {} not found", id);
            return Err((Status::NotFound, Json(MessageOnlyResponse { message: error_message })));
        }
    };

    let user_id = match current_todo.user_id {
        Some(uid) => uid,
        None => {
            let error_message = "User ID is missing".to_string();
            return Err((Status::BadRequest, Json(MessageOnlyResponse { message: error_message })));
        }
    };

    let updated_response = Todo {
        id: Some(current_todo.id),
        title: updated_todo
            .title
            .clone()
            .unwrap_or(current_todo.title),
        description: updated_todo
            .description
            .clone()
            .unwrap_or(current_todo.description),
        completed: updated_todo
            .completed
            .unwrap_or(current_todo.completed.unwrap_or(false)),
        user_id,
    };

    sqlx::query!(
        "UPDATE todos SET title = $1, description = $2, completed = $3 WHERE id = $4",
        updated_response.title,
        updated_response.description,
        updated_response.completed,
        id
    )
        .execute(pool.inner())
        .await
        .map_err(|_| {
            let error_message = "Failed to update todo".to_string();
            (Status::InternalServerError, Json(MessageOnlyResponse { message: error_message }))
        })?;

    Ok(Json(updated_response))
}

#[get("/", format = "json")]
pub async fn get_todos(_auth: AuthGuard, pool: &State<PgPool>) -> Result<Json<Vec<Todo>>, String> {
    let todos = sqlx::query!(
        "SELECT id, title, description, completed, user_id FROM todos"
    )
        .fetch_all(pool.inner())
        .await
        .map_err(|_| "Failed to fetch todos".to_string())?;

    let mut todos_response: Vec<Todo> = Vec::new();

    for todo in todos {
        let user_id = todo.user_id.ok_or("User ID is missing")?;
        todos_response.push(Todo {
            id: Some(todo.id),
            title: todo.title,
            description: todo.description,
            completed: todo.completed.unwrap_or(false),
            user_id,
        });
    }

    Ok(Json(todos_response))
}
