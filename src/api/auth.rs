use actix_web::{web, post, HttpResponse, Responder};
use sqlx::PgPool;
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, Header, EncodingKey};
use serde::{Serialize, Deserialize};
use crate::models::{User, RegisterUser, LoginUser};
use crate::config::Config;
use crate::errors::ApiError;

// ... (Struct LoginResponse & Claims tetap sama seperti sebelumnya)

#[post("/register")]
pub async fn register(
    pool: web::Data<PgPool>,
    body: web::Json<RegisterUser>,
) -> Result<HttpResponse, ApiError> {
    let hashed_password = hash(&body.password, DEFAULT_COST)
        .map_err(|_| ApiError::InternalServerError("Gagal hash password".to_string()))?;

    let user = sqlx::query_as!(
        User,
        "INSERT INTO users (name, email, password) VALUES ($1, $2, $3)
         RETURNING id, name, email, created_at, updated_at, deleted_at",
        body.name, body.email, hashed_password
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| ApiError::InternalServerError(format!("Gagal membuat user: {}", e)))?;
    
    // ... (Struct RegisterResponse tetap sama)
    Ok(HttpResponse::Created().json(RegisterResponse {
        message: "Registrasi berhasil".to_string(),
        user,
    }))
}

// ... (Kode untuk handler `login` juga dipindahkan ke sini, diubah agar me-return Result<HttpResponse, ApiError>)