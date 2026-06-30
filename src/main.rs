use actix_web::{web, App, HttpServer};
use sqlx::postgres::PgPoolOptions;

mod auth_middleware;
mod config;
mod errors;
mod handlers;
mod models;
mod routes;

use crate::config::Config;
use crate::routes::configure_routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 1. Inisialisasi Konfigurasi
    let cfg = Config::from_env();

    // 2. Inisialisasi Koneksi Database Pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&cfg.database_url)
        .await
        .expect("Gagal membuat koneksi pool.");

    // Jalankan migrasi database otomatis
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Gagal menjalankan migrasi database");

    println!("🚀 Server berhasil dijalankan pada http://127.0.0.1:8080");

    // 3. Menjalankan Server HTTP
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(cfg.clone())) // Bagikan config ke handler
            .configure(configure_routes) // Daftarkan semua rute dari routes.rs
            // Service untuk file statis
            .service(actix_files::Files::new("/public", "./public"))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}