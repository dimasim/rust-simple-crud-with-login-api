use actix_web::web;
use crate::api::{auth, todos, middleware::Auth};

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(auth::register)
            .service(auth::login)
            // Grup endpoint yang membutuhkan autentikasi
            .service(
                web::scope("")
                    .wrap(Auth) // Terapkan middleware di sini
                    .service(todos::get_todos)
                    .service(todos::create_todo)
                    .service(todos::upload_todo_image),
            )
    );
}