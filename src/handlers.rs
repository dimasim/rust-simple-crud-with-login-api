// src/handlers.rs

use actix_web::{web, post, HttpResponse, Responder,web::ReqData};
use sqlx::PgPool;
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, Header, EncodingKey};
use serde::{Serialize, Deserialize};
use crate::models::{User, RegisterUser, LoginUser};
use std::env;
use crate::models::{Todo, CreateTodo};


#[derive(Serialize)]
struct RegisterResponse {
    message: String,
    user: User,
}

#[derive(Serialize)]
struct LoginResponse {
    token: String,
}

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: i32, // Subject (user_id)
    exp: usize, // Expiration time
}

// Endpoint: POST /api/register
#[post("/register")]
pub async fn register(
    pool: web::Data<PgPool>,
    body: web::Json<RegisterUser>,
) -> impl Responder {
    let hashed_password = match hash(&body.password, DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => return HttpResponse::InternalServerError().json("Gagal hash password"),
    };

    let result = sqlx::query_as!(
        User,
        "INSERT INTO users (name, email, password) VALUES ($1, $2, $3)
         RETURNING id, name, email, created_at, updated_at, deleted_at",
        body.name,
        body.email,
        hashed_password
    )
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(user) => HttpResponse::Created().json(RegisterResponse {
            message: "Registrasi berhasil".to_string(),
            user,
        }),
        Err(e) => {
            // Error jika email sudah ada
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Gagal membuat user",
                "details": e.to_string()
            }))
        }
    }
}

// Endpoint: POST /api/login
#[post("/login")]
pub async fn login(
    pool: web::Data<PgPool>,
    body: web::Json<LoginUser>,
) -> impl Responder {
    let user = match sqlx::query!("SELECT id, password FROM users WHERE email = $1", body.email)
        .fetch_optional(pool.get_ref())
        .await
    {
        Ok(Some(user)) => user,
        _ => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Email atau password salah"})),
    };

    if !verify(&body.password, &user.password).unwrap_or(false) {
        return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Email atau password salah"}));
    }

    // Buat token JWT
    let exp = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user.id,
        exp: exp as usize,
    };
    
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
        .unwrap();

    HttpResponse::Ok().json(LoginResponse { token })
}
#[get("/todos")]
pub async fn get_todos(
    pool: web::Data<PgPool>,
    user_id: ReqData<i32>, // Ambil user_id dari middleware
) -> impl Responder {
    let id = user_id.into_inner();
    let result = sqlx::query_as!(
        Todo,
        "SELECT id, title, description, is_done, image_url, user_id, created_at FROM todos WHERE user_id = $1",
        id
    )
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
    let result = sqlx::query_as!(
        Todo,
        "INSERT INTO todos (title, description, user_id) VALUES ($1, $2, $3)
         RETURNING id, title, description, is_done, image_url, user_id, created_at",
        body.title,
        body.description,
        id
    )
    .fetch_one(pool.get_ref())
    .await;

    match result {
        Ok(todo) => HttpResponse::Created().json(serde_json::json!({"data": todo})),
        Err(_) => HttpResponse::InternalServerError().json("Gagal membuat todo"),
    }
}



// Tambahkan ini di src/handlers.rs
use actix_multipart::Multipart;
use futures_util::TryStreamExt;
use std::io::Write;
use std::fs::File;
use uuid::Uuid;
use actix_web::patch; // Kita gunakan PATCH atau POST, POST lebih cocok di sini

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
    let todo_check = sqlx::query!("SELECT user_id FROM todos WHERE id = $1", todo_id)
        .fetch_optional(pool.get_ref())
        .await;

    match todo_check {
        Ok(Some(record)) => {
            if record.user_id != current_user_id {
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
            let result = sqlx::query_as!(
                Todo,
                "UPDATE todos SET image_url = $1, updated_at = NOW() WHERE id = $2
                 RETURNING id, title, description, is_done, image_url, user_id, created_at",
                file_url,
                todo_id
            )
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