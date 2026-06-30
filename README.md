# Rust Todo API Service with Actix-Web, SQLx, and PostgreSQL

Project ini adalah RESTful API untuk aplikasi Todo yang dilengkapi dengan fitur autentikasi (JWT), manajemen Todo, serta upload gambar. Project ini telah diperbaiki, dikonfigurasi menggunakan PostgreSQL, dan dibungkus menggunakan Docker Compose dengan migrasi database otomatis.

## Fitur Utama
1. **Autentikasi Pengguna**: Registrasi (`/api/register`) & Login (`/api/login`) menggunakan hash password (bcrypt) dan JSON Web Token (JWT).
2. **Manajemen Todo**: Membuat, mengambil daftar Todo yang spesifik untuk setiap pengguna.
3. **Upload Gambar Todo**: Upload file gambar/foto pendukung Todo dan disimpan secara statis di direktori `./public/uploads`.
4. **Auto-Migration**: Migrasi database PostgreSQL berjalan otomatis saat aplikasi web dijalankan.
5. **Dockerized Setup**: Seluruh sistem siap dijalankan menggunakan satu command `docker compose`.

---

## Prasyarat
Pastikan Anda sudah menginstal:
* [Docker & Docker Desktop](https://www.docker.com/)

---

## Cara Menjalankan Aplikasi

### 1. Menjalankan Menggunakan Docker (Rekomendasi)
Anda hanya perlu menjalankan satu perintah untuk membangun dan menjalankan aplikasi beserta database PostgreSQL:

```bash
docker compose up --build
```

Setelah perintah di atas berjalan, sistem akan melakukan langkah berikut secara otomatis:
1. Menjalankan container PostgreSQL (`todo_postgres`) di port internal dan mengekspos port `5434` ke mesin lokal Anda.
2. Membangun aplikasi Rust menggunakan Dockerfile multi-stage.
3. Menunggu database PostgreSQL siap menerima koneksi.
4. Menjalankan migrasi database otomatis untuk membuat tabel `users` dan `todos`.
5. Menjalankan server Actix-Web di port `8080`.

Aplikasi siap diakses di: **`http://localhost:8080`**

### 2. Menjalankan Secara Lokal (Tanpa Docker)
Jika ingin menjalankan secara lokal tanpa Docker, Anda memerlukan instansi PostgreSQL yang berjalan (misal pada port `5434`), kemudian sesuaikan file `.env`:

```env
DATABASE_URL=postgres://postgres:postgres@localhost:5434/rust_app
JWT_SECRET=rahasia-sekali-jangan-disebar
```

Jalankan perintah berikut:
```bash
# Menjalankan migrasi (jika sqlx-cli terinstal)
sqlx migrate run

# Menjalankan aplikasi
cargo run
```

---

## Dokumentasi API Endpoint

Semua endpoint API diawali dengan prefix `/api`.

### 1. Autentikasi Pengguna

#### **Registrasi Pengguna Baru**
* **Endpoint:** `POST /api/register`
* **Request Body (JSON):**
  ```json
  {
    "name": "Dimas",
    "email": "dimas@example.com",
    "password": "password123"
  }
  ```
* **Response (201 Created):**
  ```json
  {
    "message": "Registrasi berhasil",
    "user": {
      "id": 1,
      "name": "Dimas",
      "email": "dimas@example.com",
      "created_at": "2026-06-30T17:30:00Z",
      "updated_at": "2026-06-30T17:30:00Z",
      "deleted_at": null
    }
  }
  ```

#### **Login Pengguna**
* **Endpoint:** `POST /api/login`
* **Request Body (JSON):**
  ```json
  {
    "email": "dimas@example.com",
    "password": "password123"
  }
  ```
* **Response (200 OK):**
  ```json
  {
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
  }
  ```
  *(Simpan token ini untuk digunakan pada request selanjutnya sebagai header Authorization)*

---

### 2. Manajemen Todo (Memerlukan Autentikasi)
Setiap request ke endpoint di bawah ini wajib menyertakan header:
`Authorization: Bearer <TOKEN_ANDA>`

#### **Mengambil Daftar Todo**
* **Endpoint:** `GET /api/todos`
* **Response (200 OK):**
  ```json
  {
    "data": [
      {
        "id": 1,
        "title": "Belajar Rust",
        "description": "Belajar actix-web dan sqlx",
        "is_done": false,
        "image_url": null,
        "user_id": 1,
        "created_at": "2026-06-30T17:35:00Z"
      }
    ]
  }
  ```

#### **Membuat Todo Baru**
* **Endpoint:** `POST /api/todos`
* **Request Body (JSON):**
  ```json
  {
    "title": "Belajar Docker",
    "description": "Membungkus aplikasi Rust ke Docker container"
  }
  ```
* **Response (201 Created):**
  ```json
  {
    "data": {
      "id": 2,
      "title": "Belajar Docker",
      "description": "Membungkus aplikasi Rust ke Docker container",
      "is_done": false,
      "image_url": null,
      "user_id": 1,
      "created_at": "2026-06-30T17:40:00Z"
    }
  }
  ```

#### **Upload Gambar untuk Todo**
* **Endpoint:** `POST /api/todos/{id}/upload`
* **Request Type:** `multipart/form-data`
* **Form Field:** `image` (pilih file gambar JPEG/PNG)
* **Response (200 OK):**
  ```json
  {
    "data": {
      "id": 2,
      "title": "Belajar Docker",
      "description": "Membungkus aplikasi Rust ke Docker container",
      "is_done": false,
      "image_url": "public/uploads/xxxx-xxxx-xxxx.jpg",
      "user_id": 1,
      "created_at": "2026-06-30T17:40:00Z"
    }
  }
  ```
  *(Gambar dapat diakses secara statis melalui browser pada alamat: `http://localhost:8080/public/uploads/xxxx-xxxx-xxxx.jpg`)*

---

## Struktur Direktori Penting
* `migrations/`: Menyimpan file SQL DDL untuk skema database.
* `src/auth_middleware.rs`: Middleware JWT untuk memvalidasi request token.
* `src/handlers.rs`: Berisi semua logika penanganan endpoint HTTP.
* `src/routes.rs`: Mendaftarkan dan memetakan rute endpoint ke handler yang sesuai.
* `src/main.rs`: Entry point aplikasi yang menginisialisasi database pool dan menjalankan server HTTP Actix-web.
* `Dockerfile` & `docker-compose.yml`: Berkas konfigurasi Dockerisasi.
