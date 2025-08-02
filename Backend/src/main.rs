use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use actix_cors::Cors;
use bcrypt::verify;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;
use dotenv::dotenv;

use serde::Deserialize;

#[derive(Deserialize)]
struct LoginParams {
    roll_no: String,
    password: String,
}


#[get("/login")]
async fn login_handler(
    db: web::Data<PgPool>,
    query: web::Query<LoginParams>,
) -> impl Responder {
    let roll_no = &query.roll_no;
    let password_input = &query.password;

    // Query DB for stored hash
    let result = sqlx::query!(
        "SELECT password_hash FROM users WHERE roll_no = $1",
        roll_no
    )
    .fetch_optional(db.get_ref())
    .await;

    println!("Roll No: {}", roll_no);
println!("Password: {}", password_input);

    match result {
        Ok(Some(row)) => {
            println!("Got hash from DB: {:?}", row.password_hash);
            let is_valid = verify(password_input, &row.password_hash).unwrap_or(false);

            if is_valid {
                HttpResponse::Ok().json(serde_json::json!({
                    "success": true,
                    "message": "Login successful"
                }))
            } else {
                HttpResponse::Unauthorized().json(serde_json::json!({
                    "success": false,
                    "message": "Invalid password"
                }))
            }
        }
        Ok(None) => {
            HttpResponse::NotFound().json(serde_json::json!({
                "success": false,
                "message": "Roll number not found"
            }))
        }
        Err(e) => {
    eprintln!("🔥 Login error: {:?}", e);
    HttpResponse::InternalServerError().json(serde_json::json!({
        "success": false,
        "message": "Server error"
    }))
}
    }
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables from .env file
    dotenv().ok();

    // Read the database URL
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env");

    // Create the PostgreSQL connection pool
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to the database");

    println!("Connected to DB ✅");

    // Start the Actix Web server
    HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .app_data(web::Data::new(db_pool.clone())) // share DB pool
            .wrap(cors)
            .service(login_handler)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
