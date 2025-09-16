// src/models.rs

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use sqlx::FromRow;

// Untuk response registrasi
#[derive(Serialize, FromRow)]
pub struct User {
    #[serde(rename = "ID")]
    pub id: i32,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Email")]
    pub email: String,
    #[serde(rename = "CreatedAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "UpdatedAt")]
    pub updated_at: DateTime<Utc>,
    #[serde(rename = "DeletedAt")]
    pub deleted_at: Option<DateTime<Utc>>,
}

// Untuk request registrasi
#[derive(Deserialize)]
pub struct RegisterUser {
    pub name: String,
    pub email: String,
    pub password: String,
}

// Untuk request login
#[derive(Deserialize)]
pub struct LoginUser {
    pub email: String,
    pub password: String,
}

// Untuk data Todo
#[derive(Serialize, FromRow, Debug)]
pub struct Todo {
    #[serde(rename = "ID")]
    pub id: i32,
    #[serde(rename = "Title")]
    pub title: String,
    #[serde(rename = "Description")]
    pub description: Option<String>,
    #[serde(rename = "IsDone")]
    pub is_done: bool,
    #[serde(rename = "ImageURL")]
    pub image_url: Option<String>,
    #[serde(rename = "UserID")]
    pub user_id: i32,
    #[serde(rename = "CreatedAt")]
    pub created_at: DateTime<Utc>,
}

// Untuk request membuat Todo baru
#[derive(Deserialize)]
pub struct CreateTodo {
    pub title: String,
    pub description: Option<String>,
}