use actix_web::{get, web, App, HttpServer, Responder};
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::env;

// Handler sederhana untuk menguji server
#[get("/")]
async fn hello() -> impl Responder {
    "Selamat Datang di Backend Rust!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 1. Memuat variabel dari file .env
    dotenv().ok();
    println!("🚀 Server berhasil dijalankan!");

    // 2. Mengambil URL database dari environment
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // 3. Membuat koneksi pool ke PostgreSQL
    // Pool ini akan dibagikan ke semua thread server
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool.");

    // 4. Menjalankan server HTTP
    HttpServer::new(move || {
        App::new()
            // Menyimpan state (koneksi pool) agar bisa diakses oleh handler
            .app_data(web::Data::new(pool.clone()))
            .service(hello)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}