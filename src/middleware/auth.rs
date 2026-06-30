use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures_util::future::LocalBoxFuture;
use jsonwebtoken::{decode, Validation, DecodingKey};
use std::env;
use std::future::{ready, Ready};
use crate::handlers::auth::Claims; // Gunakan struct Claims dari handlers/auth

pub struct Auth;

impl<S, B> Transform<S, ServiceRequest> for Auth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware { service }))
    }
}

pub struct AuthMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let auth_header = req.headers().get("Authorization");
        let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

        if let Some(auth_val) = auth_header {
            if let Ok(auth_str) = auth_val.to_str() {
                if auth_str.starts_with("Bearer ") {
                    let token = &auth_str[7..];
                    let token_data = decode::<Claims>(
                        token,
                        &DecodingKey::from_secret(secret.as_ref()),
                        &Validation::default(),
                    );
                    
                    if let Ok(data) = token_data {
                        // Sisipkan user_id ke request extensions agar bisa diakses handler
                        req.extensions_mut().insert(data.claims.sub);
                        return Box::pin(self.service.call(req));
                    }
                }
            }
        }
        
        Box::pin(async {
            Err(actix_web::error::ErrorUnauthorized("Token tidak valid atau tidak ada"))
        })
    }
}
