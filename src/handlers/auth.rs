use actix_web::{web, post, HttpResponse, Responder};
use sqlx::{PgPool, Row};
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, Header, EncodingKey};
use serde::{Serialize, Deserialize};
use crate::models::{User, RegisterUser, LoginUser};
use std::env;

#[derive(Serialize)]
pub struct RegisterResponse {
    pub message: String,
    pub user: User,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32, // Subject (user_id)
    pub exp: usize, // Expiration time
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

    let result = sqlx::query_as::<_, User>(
        "INSERT INTO users (name, email, password) VALUES ($1, $2, $3)
         RETURNING id, name, email, created_at, updated_at, deleted_at"
    )
    .bind(&body.name)
    .bind(&body.email)
    .bind(&hashed_password)
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
    let user_row = match sqlx::query("SELECT id, password FROM users WHERE email = $1")
        .bind(&body.email)
        .fetch_optional(pool.get_ref())
        .await
    {
        Ok(Some(row)) => row,
        _ => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Email atau password salah"})),
    };

    let user_id: i32 = user_row.get("id");
    let user_password_hash: String = user_row.get("password");

    if !verify(&body.password, &user_password_hash).unwrap_or(false) {
        return HttpResponse::Unauthorized().json(serde_json::json!({"error": "Email atau password salah"}));
    }

    // Buat token JWT
    let exp = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user_id,
        exp: exp as usize,
    };
    
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
        .unwrap();

    HttpResponse::Ok().json(LoginResponse { token })
}
