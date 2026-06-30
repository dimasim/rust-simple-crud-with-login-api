use actix_web::{web, get, post, HttpResponse, Responder, web::ReqData};
use sqlx::{PgPool, Row};
use crate::models::{Todo, CreateTodo};
use actix_multipart::Multipart;
use futures_util::TryStreamExt;
use std::io::Write;
use std::fs::File;
use uuid::Uuid;

#[get("/todos")]
pub async fn get_todos(
    pool: web::Data<PgPool>,
    user_id: ReqData<i32>, // Ambil user_id dari middleware
) -> impl Responder {
    let id = user_id.into_inner();
    let result = sqlx::query_as::<_, Todo>(
        "SELECT id, title, description, is_done, image_url, user_id, created_at FROM todos WHERE user_id = $1"
    )
    .bind(id)
    .fetch_all(pool.get_ref())
    .await;

    match result {
        Ok(todos) => HttpResponse::Ok().json(serde_json::json!({"data": todos})),
        Err(_) => HttpResponse::InternalServerError().json("Gagal mengambil data todos"),
    }
}

#[post("/todos")]
pub async fn create_todo(
    pool: web::Data<PgPool>,
    user_id: ReqData<i32>,
    body: web::Json<CreateTodo>,
) -> impl Responder {
    let id = user_id.into_inner();
    let result = sqlx::query_as::<_, Todo>(
        "INSERT INTO todos (title, description, user_id) VALUES ($1, $2, $3)
         RETURNING id, title, description, is_done, image_url, user_id, created_at"
    )
    .bind(&body.title)
    .bind(&body.description)
    .bind(id)
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(todo) => HttpResponse::Created().json(serde_json::json!({"data": todo})),
        Err(_) => HttpResponse::InternalServerError().json("Gagal membuat todo"),
    }
}

#[post("/todos/{id}/upload")]
pub async fn upload_todo_image(
    pool: web::Data<PgPool>,
    user_id: ReqData<i32>,
    path: web::Path<i32>,
    mut payload: Multipart,
) -> impl Responder {
    let current_user_id = user_id.into_inner();
    let todo_id = path.into_inner();
    
    // Verifikasi apakah todo ini milik user yang sedang login
    let todo_check = sqlx::query("SELECT user_id FROM todos WHERE id = $1")
        .bind(todo_id)
        .fetch_optional(pool.get_ref())
        .await;

    match todo_check {
        Ok(Some(row)) => {
            let todo_user_id: i32 = row.get("user_id");
            if todo_user_id != current_user_id {
                return HttpResponse::NotFound().json("Todo tidak ditemukan atau bukan milik anda");
            }
        },
        _ => return HttpResponse::NotFound().json("Todo tidak ditemukan atau bukan milik anda"),
    }

    // Proses upload
    while let Ok(Some(mut field)) = payload.try_next().await {
        if field.name() == "image" {
            // Buat nama file unik
            let file_extension = field
                .content_disposition()
                .get_filename()
                .and_then(|f| f.split('.').last())
                .unwrap_or("jpg");

            let file_name = format!("{}.{}", Uuid::new_v4(), file_extension);
            let file_path = format!("./public/uploads/{}", file_name);
            let file_url = format!("public/uploads/{}", file_name);

            // Buat direktori jika belum ada
            std::fs::create_dir_all("./public/uploads").unwrap();

            let mut f = File::create(&file_path).unwrap();

            // Tulis file chunk by chunk
            while let Some(chunk) = field.try_next().await.unwrap() {
                f.write_all(&chunk).unwrap();
            }

            // Update database
            let result = sqlx::query_as::<_, Todo>(
                "UPDATE todos SET image_url = $1, updated_at = NOW() WHERE id = $2
                 RETURNING id, title, description, is_done, image_url, user_id, created_at"
            )
            .bind(&file_url)
            .bind(todo_id)
            .fetch_one(pool.get_ref())
            .await;

            return match result {
                Ok(todo) => HttpResponse::Ok().json(serde_json::json!({"data": todo})),
                Err(_) => HttpResponse::InternalServerError().json("Gagal update database"),
            };
        }
    }

    HttpResponse::BadRequest().json("Field 'image' tidak ditemukan")
}
