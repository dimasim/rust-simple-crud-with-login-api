use actix_web::web;
use crate::handlers::{
    auth::{register, login},
    todos::{get_todos, create_todo, upload_todo_image},
};
use crate::middleware::Auth;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(register)
            .service(login)
            // Grup endpoint yang membutuhkan autentikasi
            .service(
                web::scope("")
                    .wrap(Auth) // Terapkan middleware di sini
                    .service(get_todos)
                    .service(create_todo)
                    .service(upload_todo_image),
            )
    );
}